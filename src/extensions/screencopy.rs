//! Support for the wlroots Screencopy (Version 1) Protocol
//!
//! Warning: This protocol is unstable and can change in the future
//! Current Protocol: https://github.com/swaywm/wlroots/blob/master/protocol/wlr-screencopy-unstable-v1.xml  

use crate::wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{
    wl_display, wlr_screencopy_manager_v1, wlr_screencopy_manager_v1_create
};

#[derive(Debug)]
/// Manager that offers requests to start capturing from a source
pub struct ZManagerV1 {
    manager: *mut wlr_screencopy_manager_v1
}

impl ZManagerV1 {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_screencopy_manager_v1_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(ZManagerV1 { manager: manager_raw })
        } else {
            None
        }
    }
}
