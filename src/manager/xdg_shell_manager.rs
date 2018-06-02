//! Manager for stable XDG shell client.

use libc;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_xdg_surface, wlr_xdg_surface_role::*};

use super::xdg_shell_handler::XdgShell;
use {SurfaceHandle, XdgPopup, XdgShellHandler, XdgShellState::*, XdgShellSurface,
     XdgShellSurfaceHandle, XdgTopLevel};
use compositor::{compositor_handle, CompositorHandle};

pub trait XdgShellManagerHandler {
    /// Callback that is triggered when a new stable XDG shell surface appears.
    fn new_surface(&mut self,
                   CompositorHandle,
                   XdgShellSurfaceHandle)
                   -> Option<Box<XdgShellHandler>>;

    /// Callback that is triggered when an stable XDG shell surface is destroyed.
    fn surface_destroyed(&mut self, CompositorHandle, XdgShellSurfaceHandle);
}

wayland_listener!(XdgShellManager, (Vec<Box<XdgShell>>, Box<XdgShellManagerHandler>), [
    add_listener => add_notify: |this: &mut XdgShellManager, data: *mut libc::c_void,|
    unsafe {
        let remove_listener = this.remove_listener() as *mut _ as _;
        let (ref mut shells, ref mut manager) = this.data;
        let data = data as *mut wlr_xdg_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        wlr_log!(L_DEBUG, "New xdg_shell_surface request {:p}", data);
        let surface = SurfaceHandle::from_ptr((*data).surface);
        let state = unsafe {
            match (*data).role {
                WLR_XDG_SURFACE_ROLE_NONE => None,
                WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
                    let toplevel = (*data).__bindgen_anon_1.toplevel;
                    Some(TopLevel(XdgTopLevel::from_shell(data, toplevel)))
                }
                WLR_XDG_SURFACE_ROLE_POPUP => {
                    let popup = (*data).__bindgen_anon_1.popup;
                    Some(Popup(XdgPopup::from_shell(data, popup)))
                }
            }
        };
        let shell_surface = XdgShellSurface::new(data, state);

        let new_surface_res = manager.new_surface(compositor,
                                                  shell_surface.weak_reference());

        if let Some(shell_surface_handler) = new_surface_res {

            let mut shell_surface = XdgShell::new((shell_surface, shell_surface_handler));

            // Hook the destroy event into this manager.
            wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                          remove_listener);

            // Hook the commit signal from the surface into the shell handler.
            wl_signal_add(&mut (*(*data).surface).events.commit as *mut _ as _,
                          shell_surface.commit_listener() as _);

            wl_signal_add(&mut (*data).events.ping_timeout as *mut _ as _,
                          shell_surface.ping_timeout_listener() as _);
            wl_signal_add(&mut (*data).events.new_popup as *mut _ as _,
                          shell_surface.new_popup_listener() as _);
            let events = with_handles!([(shell_surface: {shell_surface.surface_mut()})] => {
                match shell_surface.state() {
                    None | Some(&mut Popup(_)) => None,
                    Some(&mut TopLevel(ref mut toplevel)) => Some((*toplevel.as_ptr()).events)
                }
            }).expect("Cannot borrow xdg shell surface");
            if let Some(mut events) = events {
                wl_signal_add(&mut events.request_maximize as *mut _ as _,
                              shell_surface.maximize_listener() as _);
                wl_signal_add(&mut events.request_fullscreen as *mut _ as _,
                              shell_surface.fullscreen_listener() as _);
                wl_signal_add(&mut events.request_minimize as *mut _ as _,
                              shell_surface.minimize_listener() as _);
                wl_signal_add(&mut events.request_move as *mut _ as _,
                              shell_surface.move_listener() as _);
                wl_signal_add(&mut events.request_resize as *mut _ as _,
                              shell_surface.resize_listener() as _);
                wl_signal_add(&mut events.request_show_window_menu as *mut _ as _,
                              shell_surface.show_window_menu_listener() as _);
            }

            shells.push(shell_surface);
        }
    };
    remove_listener => remove_notify: |this: &mut XdgShellManager, data: *mut libc::c_void,|
    unsafe {
        let (ref mut shells, ref mut manager) = this.data;
        let data = data as *mut wlr_xdg_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        if let Some(shell) = shells.iter_mut().find(|shell| shell.surface_ptr() == data) {
            let mut shell_surface = shell.surface_mut();
            manager.surface_destroyed(compositor, shell_surface);
        }
        if let Some(index) = shells.iter().position(|shell| shell.surface_ptr() == data) {
            let mut removed_shell = shells.remove(index);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.commit_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.ping_timeout_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.new_popup_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.maximize_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.fullscreen_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.minimize_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.move_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.resize_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.show_window_menu_listener()).link as *mut _ as _);
        }
    };
]);
