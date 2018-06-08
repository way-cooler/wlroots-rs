//! Manager for stable XDG shell client.

use libc;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_xdg_surface, wlr_xdg_surface_role::*};

use types::{shell::XdgShellSurfaceState, surface::InternalSurfaceState};
use super::xdg_shell_handler::XdgShell;
use {SurfaceHandler, XdgPopup, XdgShellHandler, XdgShellState::*, XdgShellSurface,
     XdgShellSurfaceHandle, XdgTopLevel};
use compositor::{compositor_handle, CompositorHandle};

pub trait XdgShellManagerHandler {
    /// Callback that is triggered when a new stable XDG shell surface appears.
    fn new_surface(&mut self,
                   CompositorHandle,
                   XdgShellSurfaceHandle)
                   -> (Option<Box<XdgShellHandler>>, Option<Box<SurfaceHandler>>);
}

wayland_listener!(XdgShellManager, Box<XdgShellManagerHandler>, [
    add_listener => add_notify: |this: &mut XdgShellManager, data: *mut libc::c_void,|
    unsafe {
        let manager = &mut this.data;
        let data = data as *mut wlr_xdg_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        wlr_log!(L_DEBUG, "New xdg_shell_surface request {:p}", data);
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

        if let (Some(shell_surface_handler), surface_handler) = new_surface_res {
            construct_xdg_shell(shell_surface, shell_surface_handler, surface_handler, data);

        }
    };
]);
pub(crate) unsafe fn construct_xdg_shell(shell_surface: XdgShellSurface,
                                         shell_surface_handler: Box<XdgShellHandler>,
                                         surface_handler: Option<Box<SurfaceHandler>>,
                                         data: *mut wlr_xdg_surface) {
    let mut shell_surface = XdgShell::new((shell_surface, shell_surface_handler));
    let surface_state = (*(*data).surface).data as *mut InternalSurfaceState;
    if let Some(surface_handler) = surface_handler {
        (*(*surface_state).surface).data().1 = surface_handler;
    }

    wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                  shell_surface.destroy_listener() as _);
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
    let shell_data = (*data).data as *mut XdgShellSurfaceState;
    (*shell_data).shell = Box::into_raw(shell_surface);
}
