//! Support for the wlroots Idle Inhibit Protocol
//!
//! Warning: This protocol is unstable and can change in the future

use crate::wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{
    wl_display, wlr_idle_inhibit_manager_v1, wlr_idle_inhibit_v1_create
};

#[derive(Debug)]
pub struct ZManagerV1 {
    manager: *mut wlr_idle_inhibit_manager_v1
}

impl ZManagerV1 {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_idle_inhibit_v1_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(ZManagerV1 { manager: manager_raw })
        } else {
            None
        }
    }
}
