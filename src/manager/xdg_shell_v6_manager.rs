//! Manager for XDG shell v6 client.

use std::ptr::NonNull;

use crate::libc;
use crate::wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{wlr_xdg_surface_v6, wlr_xdg_surface_v6_role::*};

use super::xdg_shell_v6_handler::XdgShellV6;
use crate::{
    compositor,
    shell::xdg_shell_v6::{self, ShellState},
    surface,
    utils::Handleable
};

pub type NewSurfaceResult = (Option<Box<dyn xdg_shell_v6::Handler>>, Option<Box<dyn surface::Handler>>);

/// Callback that is triggered when a new XDG shell v6 surface appears.
pub type NewSurface =
    fn(compositor_handle: compositor::Handle, xdg_shell_v6_handle: xdg_shell_v6::Handle) -> NewSurfaceResult;

wayland_listener_static! {
    static mut MANAGER;
    (Manager, Builder): [
        (NewSurface, add_listener, surface_added) => (add_notify, surface_added):
        |manager: &mut Manager, data: *mut libc::c_void,| unsafe {
            let xdg_v6_surface = NonNull::new(data as *mut wlr_xdg_surface_v6)
                .expect("XDGv6 Surface was null");
            let xdg_v6_surface_ptr = xdg_v6_surface.as_ptr();
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            wlr_log!(WLR_DEBUG, "New xdg_shell_v6_surface request {:p}", xdg_v6_surface_ptr);
            let state = unsafe {
                match (*xdg_v6_surface_ptr).role {
                    WLR_XDG_SURFACE_V6_ROLE_NONE => None,
                    WLR_XDG_SURFACE_V6_ROLE_TOPLEVEL => {
                        let toplevel = NonNull::new((*xdg_v6_surface_ptr).__bindgen_anon_1.toplevel)
                            .expect("XDGv6 Toplevel was null");
                        Some(ShellState::TopLevel(xdg_shell_v6::TopLevel::from_shell(xdg_v6_surface,
                                                                                     toplevel)))
                    }
                    WLR_XDG_SURFACE_V6_ROLE_POPUP => {
                        let popup = NonNull::new((*xdg_v6_surface_ptr).__bindgen_anon_1.popup)
                            .expect("XDGv6 Popup was null");
                        Some(ShellState::Popup(xdg_shell_v6::Popup::from_shell(xdg_v6_surface, popup)))
                    }
                }
            };
            let shell_surface = xdg_shell_v6::Surface::new(xdg_v6_surface, state);

            let (shell_surface_handler, surface_handler) =
                match manager.surface_added {
                    None => (None, None),
                    Some(f) => f(compositor, shell_surface.weak_reference())
                };

            let mut shell_surface = XdgShellV6::new((shell_surface, shell_surface_handler));
            let surface_state = (*(*xdg_v6_surface_ptr).surface).data as *mut surface::InternalState;
            if let Some(surface_handler) = surface_handler {
                (*(*surface_state).surface.unwrap().as_ptr()).data().1 = surface_handler;
            }

            wl_signal_add(&mut (*xdg_v6_surface_ptr).events.destroy as *mut _ as _,
                          shell_surface.destroy_listener() as _);
            wl_signal_add(&mut (*(*xdg_v6_surface_ptr).surface).events.commit as *mut _ as _,
                          shell_surface.commit_listener() as _);
            wl_signal_add(&mut (*xdg_v6_surface_ptr).events.ping_timeout as *mut _ as _,
                          shell_surface.ping_timeout_listener() as _);
            wl_signal_add(&mut (*xdg_v6_surface_ptr).events.new_popup as *mut _ as _,
                          shell_surface.new_popup_listener() as _);
            wl_signal_add(&mut (*xdg_v6_surface_ptr).events.map as *mut _ as _,
                          shell_surface.map_listener() as _);
            wl_signal_add(&mut (*xdg_v6_surface_ptr).events.unmap as *mut _ as _,
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

            let shell_data = (*xdg_v6_surface_ptr).data as *mut xdg_shell_v6::SurfaceState;
            (*shell_data).shell = NonNull::new(Box::into_raw(shell_surface));
        };
    ]
}
