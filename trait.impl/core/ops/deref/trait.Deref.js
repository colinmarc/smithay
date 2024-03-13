(function() {var implementors = {
"calloop":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"calloop/generic/struct.NoIoDrop.html\" title=\"struct calloop::generic::NoIoDrop\">NoIoDrop</a>&lt;T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/os/fd/raw/trait.AsRawFd.html\" title=\"trait std::os::fd::raw::AsRawFd\">AsRawFd</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"calloop/generic/struct.FdWrapper.html\" title=\"struct calloop::generic::FdWrapper\">FdWrapper</a>&lt;T&gt;"]],
"drm":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"drm/control/dumbbuffer/struct.DumbMapping.html\" title=\"struct drm::control::dumbbuffer::DumbMapping\">DumbMapping</a>&lt;'_&gt;"]],
"gbm":[["impl&lt;'a, T: 'static&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"gbm/struct.MappedBufferObject.html\" title=\"struct gbm::MappedBufferObject\">MappedBufferObject</a>&lt;'a, T&gt;"],["impl&lt;T: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/std/os/fd/owned/trait.AsFd.html\" title=\"trait std::os::fd::owned::AsFd\">AsFd</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"gbm/struct.Device.html\" title=\"struct gbm::Device\">Device</a>&lt;T&gt;"]],
"smithay":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/backend/egl/display/struct.EGLDisplayHandle.html\" title=\"struct smithay::backend::egl::display::EGLDisplayHandle\">EGLDisplayHandle</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/backend/egl/ffi/egl/struct.LIB.html\" title=\"struct smithay::backend::egl::ffi::egl::LIB\">LIB</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/backend/renderer/element/memory/struct.MemoryBuffer.html\" title=\"struct smithay::backend::renderer::element::memory::MemoryBuffer\">MemoryBuffer</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/backend/renderer/utils/struct.Buffer.html\" title=\"struct smithay::backend::renderer::utils::Buffer\">Buffer</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/wayland/xdg_activation/struct.XdgActivationToken.html\" title=\"struct smithay::wayland::xdg_activation::XdgActivationToken\">XdgActivationToken</a>"],["impl&lt;'a, D: <a class=\"trait\" href=\"smithay/input/trait.SeatHandler.html\" title=\"trait smithay::input::SeatHandler\">SeatHandler</a> + 'static&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/wayland/pointer_constraints/struct.PointerConstraintRef.html\" title=\"struct smithay::wayland::pointer_constraints::PointerConstraintRef\">PointerConstraintRef</a>&lt;'a, D&gt;"],["impl&lt;B: <a class=\"trait\" href=\"smithay/backend/allocator/trait.Buffer.html\" title=\"trait smithay::backend::allocator::Buffer\">Buffer</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"smithay/backend/allocator/struct.Slot.html\" title=\"struct smithay::backend::allocator::Slot\">Slot</a>&lt;B&gt;"]],
"tracing":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"tracing/span/struct.EnteredSpan.html\" title=\"struct tracing::span::EnteredSpan\">EnteredSpan</a>"]],
"udev":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"udev/struct.Event.html\" title=\"struct udev::Event\">Event</a>"]],
"winit":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"winit/event_loop/struct.EventLoop.html\" title=\"struct winit::event_loop::EventLoop\">EventLoop</a>&lt;T&gt;"]],
"x11rb":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"x11rb/utils/struct.CSlice.html\" title=\"struct x11rb::utils::CSlice\">CSlice</a>"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()