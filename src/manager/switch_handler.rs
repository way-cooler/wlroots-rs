//! Handler for lid switches

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{wlr_event_switch_toggle, wlr_input_device};

use crate::{
    compositor,
    input::switch::{self, Switch},
    utils::Handleable
};

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that is triggered when the switch moves.
    fn on_toggle(
        &mut self,
        compositor_handle: compositor::Handle,
        switch_handle: switch::Handle,
        event: &switch::event::Toggle
    ) {
    }

    /// Callback that is triggered when the switch is destroyed.
    fn destroyed(&mut self, compositor_handle: compositor::Handle, switch_handle: switch::Handle) {}
}

wayland_listener!(pub(crate) SwitchWrapper, (Switch, Box<dyn Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut SwitchWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut switch, ref mut switch_handler) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            switch_handler.destroyed(compositor, switch.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_toggle_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut SwitchWrapper);
    };
    on_toggle_listener => on_toggle_notify: |this: &mut SwitchWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref mut switch, ref mut switch_handler) = this.data;
        let event = switch::event::Toggle::from_ptr(data as *mut wlr_event_switch_toggle);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        switch_handler.on_toggle(compositor, switch.weak_reference(), &event);
    };
]);
