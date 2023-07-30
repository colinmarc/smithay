//! Utilities for manipulating the data devices
//!
//! The data device is wayland's abstraction to represent both selection (copy/paste) and
//! drag'n'drop actions. This module provides logic to handle this part of the protocol.
//! Selection and drag'n'drop are per-seat notions.
//!
//! This module provides the freestanding [`set_data_device_focus`] function:
//!   This function sets the data device focus for a given seat; you'd typically call it
//!   whenever the keyboard focus changes, to follow it (for example in the focus hook of your keyboards).
//!
//! Using these two functions is enough for your clients to be able to interact with each other using
//! the data devices.
//!
//! The module also provides additional mechanisms allowing your compositor to see and interact with
//! the contents of the data device:
//!
//! - the freestanding function [`set_data_device_selection`]
//!   allows you to set the contents of the selection for your clients
//! - the freestanding function [`start_dnd`] allows you to initiate a drag'n'drop event from the compositor
//!   itself and receive interactions of clients with it via an other dedicated callback.
//!
//! The module defines the role `"dnd_icon"` that is assigned to surfaces used as drag'n'drop icons.
//!
//! ## Initialization
//!
//! To initialize this implementation, create the [`DataDeviceState`], store it inside your `State` struct
//! and implement the [`DataDeviceHandler`], as shown in this example:
//!
//! ```
//! # extern crate wayland_server;
//! # #[macro_use] extern crate smithay;
//! use smithay::delegate_data_device;
//! use smithay::wayland::data_device::{ClientDndGrabHandler, DataDeviceState, DataDeviceHandler, ServerDndGrabHandler};
//! # use smithay::input::{Seat, SeatState, SeatHandler, pointer::CursorImageStatus};
//! # use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
//!
//! # struct State { data_device_state: DataDeviceState }
//! # let mut display = wayland_server::Display::<State>::new().unwrap();
//! // Create the data_device state
//! let data_device_state = DataDeviceState::new::<State>(
//!     &display.handle(),
//! );
//!
//! // insert the DataDeviceState into your state
//! // ..
//!
//! // implement the necessary traits
//! # impl SeatHandler for State {
//! #     type KeyboardFocus = WlSurface;
//! #     type PointerFocus = WlSurface;
//! #     fn seat_state(&mut self) -> &mut SeatState<Self> { unimplemented!() }
//! #     fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&WlSurface>) { unimplemented!() }
//! #     fn cursor_image(&mut self, seat: &Seat<Self>, image: CursorImageStatus) { unimplemented!() }
//! # }
//! impl ClientDndGrabHandler for State {}
//! impl ServerDndGrabHandler for State {}
//! impl DataDeviceHandler for State {
//!     type SelectionUserData = ();
//!     fn data_device_state(&self) -> &DataDeviceState { &self.data_device_state }
//!     // ... override default implementations here to customize handling ...
//! }
//! delegate_data_device!(State);
//!
//! // You're now ready to go!
//! ```

use std::{
    cell::{Ref, RefCell},
    os::unix::io::{AsRawFd, OwnedFd},
};

use tracing::instrument;
use wayland_server::{
    backend::GlobalId,
    protocol::{
        wl_data_device_manager::{DndAction, WlDataDeviceManager},
        wl_data_source::WlDataSource,
        wl_surface::WlSurface,
    },
    Client, DisplayHandle, GlobalDispatch, Resource,
};

use crate::{
    input::{
        pointer::{Focus, GrabStartData as PointerGrabStartData},
        Seat, SeatHandler,
    },
    utils::Serial,
    wayland::seat::WaylandFocus,
};

mod device;
mod dnd_grab;
mod seat_data;
mod server_dnd_grab;
mod source;

pub use device::{DataDeviceUserData, DND_ICON_ROLE};
pub use source::{with_source_metadata, DataSourceUserData, SourceMetadata};

use seat_data::{SeatData, Selection};

/// Events that are generated by interactions of the clients with the data device
#[allow(unused_variables)]
pub trait DataDeviceHandler: Sized + ClientDndGrabHandler + ServerDndGrabHandler {
    /// UserData attached to server-side selections
    type SelectionUserData: Clone + Send + Sync + 'static;

    /// [DataDeviceState] getter
    fn data_device_state(&self) -> &DataDeviceState;

    /// Action chooser for DnD negociation
    fn action_choice(&mut self, available: DndAction, preferred: DndAction) -> DndAction {
        default_action_chooser(available, preferred)
    }

    /// A client has set the selection
    fn new_selection(&mut self, source: Option<WlDataSource>, seat: Seat<Self>) {}

    /// A client requested to read the server-set selection
    ///
    /// * `mime_type` - the requested mime type
    /// * `fd` - the fd to write into
    fn send_selection(
        &mut self,
        mime_type: String,
        fd: OwnedFd,
        seat: Seat<Self>,
        user_data: &Self::SelectionUserData,
    ) {
    }
}

/// Events that are generated during client initiated drag'n'drop
#[allow(unused_variables)]
pub trait ClientDndGrabHandler: SeatHandler + Sized {
    /// A client started a drag'n'drop as response to a user pointer action
    ///
    /// * `source` - The data source provided by the client.
    ///              If it is `None`, this means the DnD is restricted to surfaces of the
    ///              same client and the client will manage data transfer by itself.
    /// * `icon` - The icon the client requested to be used to be associated with the cursor icon
    ///            during the drag'n'drop.
    /// * `seat` - The seat on which the DnD operation was started
    fn started(&mut self, source: Option<WlDataSource>, icon: Option<WlSurface>, seat: Seat<Self>) {}

    /// The drag'n'drop action was finished by the user releasing the buttons
    ///
    /// At this point, any pointer icon should be removed.
    ///
    /// Note that this event will only be generated for client-initiated drag'n'drop session.
    ///
    /// * `seat` - The seat on which the DnD action was finished.
    fn dropped(&mut self, seat: Seat<Self>) {}
}

/// Event generated by the interactions of clients with a server initiated drag'n'drop
#[allow(unused_variables)]
pub trait ServerDndGrabHandler: SeatHandler {
    /// The client can accept the given mime type.
    ///
    /// If `mime_type` is None, the client cannot accept any of the offered mime
    /// types. If the last accepted mime_type is None, the drag-and-drop
    /// operation will be cancelled. That is kept track of internally, this
    /// callback is informational only.
    /// * `mime_type` - The accepted mime type
    /// * `seat` - The seat on which the DnD action was chosen.
    fn accept(&mut self, mime_type: Option<String>, seat: Seat<Self>) {}

    /// The client chose an action
    /// * `action` - The chosen action
    /// * `seat` - The seat on which the DnD action was chosen.
    fn action(&mut self, action: DndAction, seat: Seat<Self>) {}

    /// The DnD resource was dropped by the user
    ///
    /// After that, the client can still interact with your resource
    /// * `seat` - The seat on which the DnD was dropped.
    fn dropped(&mut self, seat: Seat<Self>) {}

    /// The Dnd was cancelled
    ///
    /// The client can no longer interact
    /// * `seat` - The seat on which the DnD action was cancelled.
    fn cancelled(&mut self, seat: Seat<Self>) {}

    /// The client requested for data to be sent
    ///
    /// * `mime_type` - The requested mime type
    /// * `fd` - The FD to write into
    /// * `seat` - The seat on which the DnD data is to be sent.
    fn send(&mut self, mime_type: String, fd: OwnedFd, seat: Seat<Self>) {}

    /// The client has finished interacting with the resource
    ///
    /// This can only happen after the resource was dropped.
    /// * `seat` - The seat on which the DnD action was finished.
    fn finished(&mut self, seat: Seat<Self>) {}
}

/// State of data device
#[derive(Debug)]
pub struct DataDeviceState {
    manager_global: GlobalId,
}

impl DataDeviceState {
    /// Regiseter new [WlDataDeviceManager] global
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<WlDataDeviceManager, ()> + 'static,
        D: DataDeviceHandler,
    {
        let manager_global = display.create_global::<D, WlDataDeviceManager, _>(3, ());

        Self { manager_global }
    }

    /// [WlDataDeviceManager] GlobalId getter
    pub fn global(&self) -> GlobalId {
        self.manager_global.clone()
    }
}

/// A simple action chooser for DnD negociation
///
/// If the preferred action is available, it'll pick it. Otherwise, it'll pick the first
/// available in the following order: Ask, Copy, Move.
pub fn default_action_chooser(available: DndAction, preferred: DndAction) -> DndAction {
    // if the preferred action is valid (a single action) and in the available actions, use it
    // otherwise, follow a fallback stategy
    if [DndAction::Move, DndAction::Copy, DndAction::Ask].contains(&preferred)
        && available.contains(preferred)
    {
        preferred
    } else if available.contains(DndAction::Ask) {
        DndAction::Ask
    } else if available.contains(DndAction::Copy) {
        DndAction::Copy
    } else if available.contains(DndAction::Move) {
        DndAction::Move
    } else {
        DndAction::empty()
    }
}

/// Set the data device focus to a certain client for a given seat
#[instrument(name = "wayland_data_device", level = "debug", skip(dh, seat, client), fields(seat = seat.name(), client = ?client.as_ref().map(|c| c.id())))]
pub fn set_data_device_focus<D>(dh: &DisplayHandle, seat: &Seat<D>, client: Option<Client>)
where
    D: SeatHandler + DataDeviceHandler + 'static,
{
    seat.user_data()
        .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));
    let seat_data = seat
        .user_data()
        .get::<RefCell<SeatData<D::SelectionUserData>>>()
        .unwrap();
    seat_data.borrow_mut().set_focus::<D>(dh, client);
}

/// Set a compositor-provided selection for this seat
///
/// You need to provide the available mime types for this selection.
///
/// Whenever a client requests to read the selection, your callback will
/// receive a [`DataDeviceHandler::send_selection`] event.
#[instrument(name = "wayland_data_device", level = "debug", skip(dh, seat, user_data), fields(seat = seat.name()))]
pub fn set_data_device_selection<D>(
    dh: &DisplayHandle,
    seat: &Seat<D>,
    mime_types: Vec<String>,
    user_data: D::SelectionUserData,
) where
    D: SeatHandler + DataDeviceHandler + 'static,
{
    seat.user_data()
        .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));
    let seat_data = seat
        .user_data()
        .get::<RefCell<SeatData<D::SelectionUserData>>>()
        .unwrap();
    seat_data.borrow_mut().set_selection::<D>(
        dh,
        Selection::Compositor {
            metadata: SourceMetadata {
                mime_types,
                dnd_action: DndAction::empty(),
            },
            user_data,
        },
    );
}

/// Errors happening when requesting selection contents
#[derive(Debug, thiserror::Error)]
pub enum SelectionRequestError {
    /// Requested mime type is not available
    #[error("Requested mime type is not available")]
    InvalidMimetype,
    /// Requesting server side selection contents is not supported
    #[error("Current selection is server-side")]
    ServerSideSelection,
    /// There is no active selection
    #[error("No active selection to query")]
    NoSelection,
}

/// Request the current data_device selection of the given seat
/// to be written to the provided file descriptor in the given mime type.
pub fn request_data_device_client_selection<D>(
    seat: &Seat<D>,
    mime_type: String,
    fd: OwnedFd,
) -> Result<(), SelectionRequestError>
where
    D: SeatHandler + DataDeviceHandler + 'static,
{
    seat.user_data()
        .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));
    let seat_data = seat
        .user_data()
        .get::<RefCell<SeatData<D::SelectionUserData>>>()
        .unwrap();
    match seat_data.borrow().get_selection() {
        Selection::Client(source) => {
            if !source
                .data::<DataSourceUserData>()
                .unwrap()
                .inner
                .lock()
                .unwrap()
                .mime_types
                .contains(&mime_type)
            {
                Err(SelectionRequestError::InvalidMimetype)
            } else {
                source.send(mime_type, fd.as_raw_fd());
                Ok(())
            }
        }
        Selection::Compositor { metadata, .. } => {
            if !metadata.mime_types.contains(&mime_type) {
                Err(SelectionRequestError::InvalidMimetype)
            } else {
                Err(SelectionRequestError::ServerSideSelection)
            }
        }
        Selection::Empty => Err(SelectionRequestError::NoSelection),
    }
}

/// Gets the user_data for the currently active selection, if set by the compositor
#[instrument(name = "wayland_data_device", level = "debug", skip_all, fields(seat = seat.name()))]
pub fn current_data_device_selection_userdata<D>(seat: &Seat<D>) -> Option<Ref<'_, D::SelectionUserData>>
where
    D: SeatHandler + DataDeviceHandler + 'static,
{
    seat.user_data()
        .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));
    let seat_data = seat
        .user_data()
        .get::<RefCell<SeatData<D::SelectionUserData>>>()
        .unwrap();
    Ref::filter_map(seat_data.borrow(), |data| match data.get_selection() {
        Selection::Compositor { ref user_data, .. } => Some(user_data),
        _ => None,
    })
    .ok()
}

/// Clear the current selection for this seat
#[instrument(name = "wayland_data_device", level = "debug", skip_all, fields(seat = seat.name()))]
pub fn clear_data_device_selection<D>(dh: &DisplayHandle, seat: &Seat<D>)
where
    D: SeatHandler + DataDeviceHandler + 'static,
{
    seat.user_data()
        .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));
    let seat_data = seat
        .user_data()
        .get::<RefCell<SeatData<D::SelectionUserData>>>()
        .unwrap();
    seat_data.borrow_mut().set_selection::<D>(dh, Selection::Empty);
}

/// Start a drag'n'drop from a resource controlled by the compositor
///
/// You'll receive events generated by the interaction of clients with your
/// drag'n'drop in the provided callback. See [`ServerDndGrabHandler`] for details about
/// which events can be generated and what response is expected from you to them.
#[instrument(name = "wayland_data_device", level = "debug", skip(dh, seat, data), fields(seat = seat.name()))]
pub fn start_dnd<D>(
    dh: &DisplayHandle,
    seat: &Seat<D>,
    data: &mut D,
    serial: Serial,
    start_data: PointerGrabStartData<D>,
    metadata: SourceMetadata,
) where
    D: SeatHandler + DataDeviceHandler + 'static,
    <D as SeatHandler>::PointerFocus: WaylandFocus,
{
    seat.user_data()
        .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));
    if let Some(pointer) = seat.get_pointer() {
        pointer.set_grab(
            data,
            server_dnd_grab::ServerDnDGrab::new(dh, start_data, metadata, seat.clone()),
            serial,
            Focus::Keep,
        );
    }
}

mod handlers {
    use std::cell::RefCell;

    use tracing::error;
    use wayland_server::{
        protocol::{
            wl_data_device::WlDataDevice,
            wl_data_device_manager::{self, WlDataDeviceManager},
            wl_data_source::WlDataSource,
        },
        Dispatch, DisplayHandle, GlobalDispatch,
    };

    use crate::input::Seat;

    use super::{device::DataDeviceUserData, seat_data::SeatData, source::DataSourceUserData};
    use super::{DataDeviceHandler, DataDeviceState};

    impl<D> GlobalDispatch<WlDataDeviceManager, (), D> for DataDeviceState
    where
        D: GlobalDispatch<WlDataDeviceManager, ()>,
        D: Dispatch<WlDataDeviceManager, ()>,
        D: Dispatch<WlDataSource, DataSourceUserData>,
        D: Dispatch<WlDataDevice, DataDeviceUserData>,
        D: DataDeviceHandler,
        D: 'static,
    {
        fn bind(
            _state: &mut D,
            _handle: &DisplayHandle,
            _client: &wayland_server::Client,
            resource: wayland_server::New<WlDataDeviceManager>,
            _global_data: &(),
            data_init: &mut wayland_server::DataInit<'_, D>,
        ) {
            data_init.init(resource, ());
        }
    }

    impl<D> Dispatch<WlDataDeviceManager, (), D> for DataDeviceState
    where
        D: Dispatch<WlDataDeviceManager, ()>,
        D: Dispatch<WlDataSource, DataSourceUserData>,
        D: Dispatch<WlDataDevice, DataDeviceUserData>,
        D: DataDeviceHandler,
        D: 'static,
    {
        fn request(
            _state: &mut D,
            client: &wayland_server::Client,
            _resource: &WlDataDeviceManager,
            request: wl_data_device_manager::Request,
            _data: &(),
            _dhandle: &DisplayHandle,
            data_init: &mut wayland_server::DataInit<'_, D>,
        ) {
            match request {
                wl_data_device_manager::Request::CreateDataSource { id } => {
                    data_init.init(id, DataSourceUserData::new());
                }
                wl_data_device_manager::Request::GetDataDevice { id, seat: wl_seat } => {
                    match Seat::<D>::from_resource(&wl_seat) {
                        Some(seat) => {
                            seat.user_data()
                                .insert_if_missing(|| RefCell::new(SeatData::<D::SelectionUserData>::new()));

                            let data_device = data_init.init(id, DataDeviceUserData { wl_seat });

                            let seat_data = seat
                                .user_data()
                                .get::<RefCell<SeatData<D::SelectionUserData>>>()
                                .unwrap();
                            seat_data.borrow_mut().add_device(data_device);
                        }
                        None => {
                            error!(client = ?client, data_device = ?id, "Unmanaged seat given to a data device.");
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

#[allow(missing_docs)] // TODO
#[macro_export]
macro_rules! delegate_data_device {
    ($(@<$( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+>)? $ty: ty) => {
        $crate::reexports::wayland_server::delegate_global_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_device_manager::WlDataDeviceManager: ()
        ] => $crate::wayland::data_device::DataDeviceState);

        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_device_manager::WlDataDeviceManager: ()
        ] => $crate::wayland::data_device::DataDeviceState);
        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_device::WlDataDevice: $crate::wayland::data_device::DataDeviceUserData
        ] => $crate::wayland::data_device::DataDeviceState);
        $crate::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            $crate::reexports::wayland_server::protocol::wl_data_source::WlDataSource: $crate::wayland::data_device::DataSourceUserData
        ] => $crate::wayland::data_device::DataDeviceState);
    };
}
