//! Support for the wlroots Gamma Control Protocol
//!
//! Warning: This protocol is unstable and can change in the future
//! Current Protocol: https://github.com/swaywm/wlroots/blob/master/protocol/wlr-gamma-control-unstable-v1.xml

use crate::wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{
    wl_display, wlr_gamma_control_manager_v1, wlr_gamma_control_manager_v1_create
};

#[derive(Debug)]
/// Manager that can adjust gamma controls for an output
pub struct ZManagerV1 {
    manager: *mut wlr_gamma_control_manager_v1
}

impl ZManagerV1 {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_gamma_control_manager_v1_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(ZManagerV1 { manager: manager_raw })
        } else {
            None
        }
    }
}
