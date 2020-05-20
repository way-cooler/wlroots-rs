//! Handler for pointers

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{
    wlr_event_pointer_axis, wlr_event_pointer_button, wlr_event_pointer_motion, wlr_input_device
};

use crate::{
    compositor,
    input::pointer::{self, Pointer},
    utils::Handleable
};

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that is triggered when the pointer moves.
    fn on_motion(
        &mut self,
        compositor_handle: compositor::Handle,
        pointer_handle: pointer::Handle,
        event: &pointer::event::Motion
    ) {
    }

    fn on_motion_absolute(
        &mut self,
        compositor_handle: compositor::Handle,
        pointer_handle: pointer::Handle,
        event: &pointer::event::AbsoluteMotion
    ) {
    }

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_button(
        &mut self,
        compositor_handle: compositor::Handle,
        pointer_handle: pointer::Handle,
        event: &pointer::event::Button
    ) {
    }

    /// Callback that is triggered when an axis event fires.
    fn on_axis(
        &mut self,
        compositor_handle: compositor::Handle,
        pointer_handle: pointer::Handle,
        event: &pointer::event::Axis
    ) {
    }

    /// Callback that is triggered when the pointer is destroyed.
    fn destroyed(&mut self, compositor_handle: compositor::Handle, pointer_handle: pointer::Handle) {}
}

wayland_listener!(pub(crate) PointerWrapper, (Pointer, Box<dyn Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut PointerWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut pointer, ref mut pointer_handler) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            pointer_handler.destroyed(compositor, pointer.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.button_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.motion_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.motion_absolute_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.axis_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut PointerWrapper);
    };
    button_listener => key_notify: |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = pointer::event::Button::from_ptr(data as *mut wlr_event_pointer_button);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_button(compositor, pointer.weak_reference(), &event);
    };
    motion_listener => motion_notify:  |this: &mut PointerWrapper, data: *mut libc::c_void,|
    unsafe {
        let pointer = &mut this.data.0;
        let event = pointer::event::Motion::from_ptr(data as *mut wlr_event_pointer_motion);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_motion(compositor, pointer.weak_reference(), &event);
    };
    motion_absolute_listener => motion_absolute_notify:
    |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = pointer::event::AbsoluteMotion::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_motion_absolute(compositor, pointer.weak_reference(), &event);
    };
    axis_listener => axis_notify:  |this: &mut PointerWrapper, data: *mut libc::c_void,| unsafe {
        let pointer = &mut this.data.0;
        let event = pointer::event::Axis::from_ptr(data as *mut wlr_event_pointer_axis);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        this.data.1.on_axis(compositor, pointer.weak_reference(), &event);
    };
]);
