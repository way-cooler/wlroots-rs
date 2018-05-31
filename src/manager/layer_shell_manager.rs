//! Manager for layer shell clients.

use libc;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_layer_surface;

use {LayerSurface, LayerSurfaceHandle, LayerShellHandler, Surface};
use super::layer_shell_handler::LayerShell;
use compositor::{compositor_handle, CompositorHandle};

pub trait LayerShellManagerHandler {
    /// Callback that is triggered when a new layer shell surface appears.
    fn new_surface(&mut self,
                   CompositorHandle,
                   LayerSurfaceHandle)
                   -> Option<Box<LayerShellHandler>>;

    /// Callback that is triggered when a layer shell surface is destroyed.
    fn surface_destroyed(&mut self, CompositorHandle, LayerSurfaceHandle);
}

wayland_listener!(LayerShellManager, (Vec<Box<LayerShell>>, Box<LayerShellManagerHandler>), [
    add_listener => add_notify: |this: &mut LayerShellManager, data: *mut libc::c_void,| unsafe {
        let remove_listener = this.remove_listener() as *mut _ as _;
        let (ref mut shells, ref mut manager) = this.data;
        let data = data as *mut wlr_layer_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        wlr_log!(L_DEBUG, "New layer shell surface request {:p}", data);
        let surface = Surface::new((*data).surface);
        let layer_surface = LayerSurface::new(data);
        let new_surface_res = manager.new_surface(compositor, layer_surface.weak_reference());
        if let Some(layer_surface_handler) = new_surface_res {
            let mut layer_surface = LayerShell::new((layer_surface,
                                                     surface,
                                                     layer_surface_handler));
            // Hook the destroy event into this manager.
            wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                          remove_listener);

            // Hook the other events into the shell surface.
            wl_signal_add(&mut (*data).events.map as *mut _ as _,
                          layer_surface.on_map_listener() as _);
            wl_signal_add(&mut (*data).events.unmap as *mut _ as _,
                          layer_surface.on_unmap_listener() as _);
            wl_signal_add(&mut (*data).events.new_popup as *mut _ as _,
                          layer_surface.new_popup_listener() as _);
            shells.push(layer_surface);
        }
    };
    remove_listener => remove_notify: |this: &mut LayerShellManager, data: *mut libc::c_void,|
    unsafe {
        let (ref mut shells, ref mut manager) = this.data;
        let data = data as *mut wlr_layer_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        if let Some(index) = shells.iter().position(|shell| shell.surface_ptr() == data) {
            let mut removed_shell = shells.remove(index);
            manager.surface_destroyed(compositor, removed_shell.surface_mut());
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.on_map_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.on_unmap_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*removed_shell.new_popup_listener()).link as *mut _ as _);
        }
    };
]);
