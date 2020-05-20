//! XWayland client resources are managed by the XWayland resource manager
//! and server.
//!
//! To manage XWayland clients (and run an XServer) implement a function
//! with [`NewSurface`](./type.NewSurface.html) as the signature.
//!
//! Pass that function to the [`xwayland::Builder`](./struct.Builder.html)
//! which is then passed to the `compositor::Builder`.

use std::ptr::NonNull;

use crate::libc;
use crate::wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_xwayland_surface;

use crate::{compositor, utils::Handleable, xwayland};

/// Callback that's triggered when the XWayland library is ready.
pub type OnReady = fn(compositor::Handle);

/// Callback that's triggered when a new surface is presented to the X
/// server.
pub type NewSurface = fn(
    compositor_handle: compositor::Handle,
    xwayland_surface: xwayland::surface::Handle
) -> Option<Box<dyn xwayland::surface::Handler>>;

wayland_listener_static! {
    static mut MANAGER;
    (Manager, Builder): [
        (OnReady, on_ready_listener, xwayland_ready) => (ready_notify, xwayland_ready):
        |manager: &mut Manager, _data: *mut libc::c_void,|
        unsafe {
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };

            if let Some(xwayland_ready) = manager.xwayland_ready {
                xwayland_ready(compositor)
            }
        };

        (NewSurface, new_surface_listener, surface_added) => (add_notify, surface_added):
        |manager: &mut Manager, data: *mut libc::c_void,|
        unsafe {
            let surface_ptr = data as *mut wlr_xwayland_surface;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            let shell_surface = xwayland::surface::Surface::new(surface_ptr);
            let xwayland_handler = manager.surface_added
                .and_then(|f| f(compositor, shell_surface.weak_reference()));
            let mut shell = xwayland::surface::Shell::new((shell_surface, xwayland_handler));

            wl_signal_add(&mut (*surface_ptr).events.destroy as *mut _ as _,
                          shell.destroy_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_configure as *mut _ as _,
                          shell.request_configure_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_move as *mut _ as _,
                          shell.request_move_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_resize as *mut _ as _,
                          shell.request_resize_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_maximize as *mut _ as _,
                          shell.request_maximize_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.request_fullscreen as *mut _ as _,
                          shell.request_fullscreen_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.map as *mut _ as _,
                          shell.map_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.unmap as *mut _ as _,
                          shell.unmap_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_title as *mut _ as _,
                          shell.set_title_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_class as *mut _ as _,
                          shell.set_class_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_parent as *mut _ as _,
                          shell.set_parent_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_pid as *mut _ as _,
                          shell.set_pid_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.set_window_type as *mut _ as _,
                          shell.set_window_type_listener() as *mut _ as _);
            wl_signal_add(&mut (*surface_ptr).events.ping_timeout as *mut _ as _,
                          shell.ping_timeout_listener() as *mut _ as _);
            let shell_data = (*surface_ptr).data as *mut xwayland::surface::State;
            (*shell_data).shell = NonNull::new(Box::into_raw(shell));
            // TODO Pass in the new surface from the data
        };
    ]
}
