//! Support for the wlroots Export DMA-BUF Protocol
//!
//! Warning: This protocol is unstable and can change in the future
//! Current Protocol: https://github.com/swaywm/wlroots/blob/master/protocol/wlr-export-dmabuf-unstable-v1.xml

use crate::wayland_sys::server::wl_display as wl_server_display;
use wlroots_sys::{
    wl_display, wlr_export_dmabuf_manager_v1, wlr_export_dmabuf_manager_v1_create,
    wlr_export_dmabuf_manager_v1_destroy
};

#[derive(Debug)]
/// Manager that can export dmabufs for an output
pub struct ZManagerV1 {
    manager: *mut wlr_export_dmabuf_manager_v1
}

impl ZManagerV1 {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_export_dmabuf_manager_v1_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(ZManagerV1 { manager: manager_raw })
        } else {
            None
        }
    }
}

impl Drop for ZManagerV1 {
    fn drop(&mut self) {
        unsafe { wlr_export_dmabuf_manager_v1_destroy(self.manager) }
    }
}
