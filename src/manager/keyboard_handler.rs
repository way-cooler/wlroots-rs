//! Handler for keyboards

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{wlr_event_keyboard_key, wlr_input_device};

use crate::{
    compositor,
    input::keyboard::{self, Keyboard},
    utils::Handleable
};

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that is triggered when a key is pressed.
    fn on_key(
        &mut self,
        compositor_handle: compositor::Handle,
        keyboard_handle: keyboard::Handle,
        event: &keyboard::event::Key
    ) {
    }

    /// Callback that is triggered when modifiers are pressed.
    fn modifiers(&mut self, compositor_handle: compositor::Handle, keyboard_handle: keyboard::Handle) {}

    /// Callback that is triggered when the keymap is updated.
    fn keymap(&mut self, compositor_handle: compositor::Handle, keyboard_handle: keyboard::Handle) {}

    /// Callback that is triggered when repeat info is updated.
    fn repeat_info(&mut self, compositor_handle: compositor::Handle, keyboard_handle: keyboard::Handle) {}

    /// Callback that is triggered when the keyboard is destroyed.
    fn destroyed(&mut self, compositor_handle: compositor::Handle, keyboard_handle: keyboard::Handle) {}
}

wayland_listener!(pub(crate) KeyboardWrapper, (Keyboard, Box<dyn Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut KeyboardWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut keyboard, ref mut keyboard_handler) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            keyboard_handler.destroyed(compositor, keyboard.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.key_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.modifiers_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.keymap_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.repeat_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut KeyboardWrapper);
    };
    key_listener => key_notify: |this: &mut KeyboardWrapper, data: *mut libc::c_void,| unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let xkb_state = (*keyboard.as_ptr()).xkb_state;
        let key = keyboard::event::Key::new(data as *mut wlr_event_keyboard_key, xkb_state);

        keyboard_handler.on_key(compositor, keyboard.weak_reference(), &key);
    };
    modifiers_listener => modifiers_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        keyboard_handler.modifiers(compositor, keyboard.weak_reference());
    };
    keymap_listener => keymap_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        keyboard_handler.keymap(compositor, keyboard.weak_reference());
    };
   repeat_listener => repeat_notify: |this: &mut KeyboardWrapper, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut keyboard, ref mut keyboard_handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        keyboard_handler.repeat_info(compositor, keyboard.weak_reference());
    };
]);
