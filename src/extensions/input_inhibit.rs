//! Support for the wlroots Input Inhibit Protocol
//!
//! Warning: This protocol is unstable and can change in the future

use crate::wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{
    wl_display, wlr_input_inhibit_manager, wlr_input_inhibit_manager_create
};

#[derive(Debug)]
pub struct ZManagerV1 {
    manager: *mut wlr_input_inhibit_manager
}

impl ZManagerV1 {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_input_inhibit_manager_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(ZManagerV1 { manager: manager_raw })
        } else {
            None
        }
    }
}
