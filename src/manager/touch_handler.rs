//! Handler for touch input

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::wlr_input_device;

use crate::{
    compositor,
    input::touch::{self, Touch},
    utils::Handleable
};

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that is triggered when the user starts touching the
    /// screen/input device.
    fn on_down(
        &mut self,
        compositor_handle: compositor::Handle,
        touch_handle: touch::Handle,
        event: &touch::event::Down
    ) {
    }

    /// Callback that is triggered when the user stops touching the
    /// screen/input device.
    fn on_up(
        &mut self,
        compositor_handle: compositor::Handle,
        touch_handle: touch::Handle,
        event: &touch::event::Up
    ) {
    }

    /// Callback that is triggered when the user moves his fingers along the
    /// screen/input device.
    fn on_motion(
        &mut self,
        compositor_handle: compositor::Handle,
        touch_handle: touch::Handle,
        event: &touch::event::Motion
    ) {
    }

    /// Callback triggered when the touch is canceled.
    fn on_cancel(
        &mut self,
        compositor_handle: compositor::Handle,
        touch_handle: touch::Handle,
        event: &touch::event::Cancel
    ) {
    }

    /// Callback that is triggered when the touch is destroyed.
    fn destroyed(&mut self, compositor_handle: compositor::Handle, touch_handle: touch::Handle) {}
}

wayland_listener!(pub(crate) TouchWrapper, (Touch, Box<dyn Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut touch, ref mut touch_handler) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            touch_handler.destroyed(compositor, touch.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.down_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.up_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.motion_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.cancel_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut TouchWrapper);
    };
    down_listener => down_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = touch::event::Down::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_down(compositor,
                        touch.weak_reference(),
                        &event);
    };
    up_listener => up_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = touch::event::Up::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_up(compositor,
                      touch.weak_reference(),
                      &event);
    };
    motion_listener => motion_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = touch::event::Motion::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_motion(compositor,
                          touch.weak_reference(),
                          &event);
    };
    cancel_listener => cancel_notify: |this: &mut TouchWrapper, data: *mut libc::c_void,| unsafe {
        let (ref touch, ref mut handler) = this.data;
        let event = touch::event::Cancel::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_cancel(compositor,
                          touch.weak_reference(),
                          &event);
    };
]);
