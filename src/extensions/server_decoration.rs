//! Support for the KDE Server Decoration Protocol

use crate::wayland_sys::server::wl_display as wl_server_display;
pub use wlroots_sys::protocols::server_decoration::server::org_kde_kwin_server_decoration_manager::Mode;

use wlroots_sys::{
    wl_display, wlr_server_decoration_manager, wlr_server_decoration_manager_create,
    wlr_server_decoration_manager_set_default_mode
};

#[derive(Debug)]
/// Coordinates whether the server should create
/// server-side window decorations.
pub struct Manager {
    manager: *mut wlr_server_decoration_manager
}

impl Manager {
    pub(crate) unsafe fn new(display: *mut wl_server_display) -> Option<Self> {
        let manager_raw = wlr_server_decoration_manager_create(display as *mut wl_display);

        if !manager_raw.is_null() {
            Some(Manager { manager: manager_raw })
        } else {
            None
        }
    }

    /// Given a mode, set the server decoration mode
    pub fn set_default_mode(&mut self, mode: Mode) {
        wlr_log!(WLR_INFO, "New server decoration mode: {:?}", mode);
        unsafe { wlr_server_decoration_manager_set_default_mode(self.manager, mode.to_raw()) }
    }
}
