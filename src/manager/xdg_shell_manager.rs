//! Manager for stable XDG shell client.

use std::ptr::NonNull;

use crate::libc;
use crate::wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_xdg_surface, wlr_xdg_surface_role::*};

use super::xdg_shell_handler::XdgShell;
use crate::{
    compositor,
    shell::xdg_shell::{self, ShellState},
    surface,
    utils::Handleable
};

pub type NewSurfaceResult = (Option<Box<dyn xdg_shell::Handler>>, Option<Box<dyn surface::Handler>>);

/// Callback that is triggered when a new stable XDG shell surface appears.
pub type NewSurface =
    fn(compositor_handle: compositor::Handle, xdg_shell_handle: xdg_shell::Handle) -> NewSurfaceResult;

wayland_listener_static! {
    static mut MANAGER;
    (Manager, Builder): [
        (NewSurface, add_listener, surface_added) => (add_notify, surface_added):
        |manager: &mut Manager, data: *mut libc::c_void,|
        unsafe {
            let xdg_surface = NonNull::new(data as *mut wlr_xdg_surface)
                .expect("Xdg shell surface was null");
            let xdg_surface_ptr = xdg_surface.as_ptr();
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            wlr_log!(WLR_DEBUG, "New xdg_shell_surface request {:p}", xdg_surface_ptr);
            let state = unsafe {
                match (*xdg_surface_ptr).role {
                    WLR_XDG_SURFACE_ROLE_NONE => None,
                    WLR_XDG_SURFACE_ROLE_TOPLEVEL => {
                        let toplevel = NonNull::new((*xdg_surface_ptr).__bindgen_anon_1.toplevel)
                            .expect("XDG Toplevel pointer was null");
                        Some(ShellState::TopLevel(xdg_shell::TopLevel::from_shell(xdg_surface, toplevel)))
                    }
                    WLR_XDG_SURFACE_ROLE_POPUP => {
                        let popup = NonNull::new((*xdg_surface_ptr).__bindgen_anon_1.popup)
                            .expect("XDG Popup pointer was null");
                        Some(ShellState::Popup(xdg_shell::Popup::from_shell(xdg_surface, popup)))
                    }
                }
            };
            let shell_surface = xdg_shell::Surface::new(xdg_surface, state);

            let (shell_surface_manager, surface_handler) =
                match manager.surface_added {
                    None => (None, None),
                    Some(f) => f(compositor, shell_surface.weak_reference())
                };

            let mut shell_surface = XdgShell::new((shell_surface, shell_surface_manager));
            let surface_state = (*(*xdg_surface_ptr).surface).data as *mut surface::InternalState;
            if let Some(surface_handler) = surface_handler {
                (*(*surface_state).surface.unwrap().as_ptr()).data().1 = surface_handler;
            }

            wl_signal_add(&mut (*xdg_surface_ptr).events.destroy as *mut _ as _,
                          shell_surface.destroy_listener() as _);
            wl_signal_add(&mut (*(*xdg_surface_ptr).surface).events.commit as *mut _ as _,
                          shell_surface.commit_listener() as _);
            wl_signal_add(&mut (*xdg_surface_ptr).events.ping_timeout as *mut _ as _,
                          shell_surface.ping_timeout_listener() as _);
            wl_signal_add(&mut (*xdg_surface_ptr).events.new_popup as *mut _ as _,
                          shell_surface.new_popup_listener() as _);
            wl_signal_add(&mut (*xdg_surface_ptr).events.map as *mut _ as _,
                          shell_surface.map_listener() as _);
            wl_signal_add(&mut (*xdg_surface_ptr).events.unmap as *mut _ as _,
                          shell_surface.unmap_listener() as _);
            let events = with_handles!([(shell_surface: {shell_surface.surface_mut()})] => {
                match shell_surface.state() {
                    None | Some(&mut ShellState::Popup(_)) => None,
                    Some(&mut ShellState::TopLevel(ref mut toplevel)) => Some((*toplevel.as_ptr()).events)
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
            let shell_data = (*xdg_surface_ptr).data as *mut xdg_shell::SurfaceState;
            (*shell_data).shell = NonNull::new(Box::into_raw(shell_surface));
        };
    ]
}
