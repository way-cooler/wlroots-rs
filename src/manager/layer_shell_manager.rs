//! Manager for layer shell clients.

use libc;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_layer_surface;

use {LayerSurface, LayerSurfaceHandle, LayerShellHandler, Surface, SurfaceHandle, OutputHandle};
use super::layer_shell_handler::LayerShell;
use compositor::{compositor_handle, CompositorHandle};

pub trait LayerShellManagerHandler {
    /// Callback that is triggered when a new layer shell surface appears.
    ///
    /// The output is the output that was specified by the client.
    /// If it was `None` then none was specified and you *must* specify it.
    /// Do so by setting the value in the `Option`.
    fn new_surface(&mut self,
                   CompositorHandle,
                   LayerSurfaceHandle,
                   &mut Option<OutputHandle>)
                   -> Option<Box<LayerShellHandler>>;
}

wayland_listener!(LayerShellManager, Box<LayerShellManagerHandler>, [
    add_listener => add_notify: |this: &mut LayerShellManager, data: *mut libc::c_void,| unsafe {
        let ref mut manager = this.data;
        let layer_surface_ptr = data as *mut wlr_layer_surface;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        wlr_log!(L_DEBUG, "New layer shell surface request {:p}", layer_surface_ptr);
        let surface = SurfaceHandle::from_ptr((*layer_surface_ptr).surface);
        let mut layer_surface = LayerSurface::new(layer_surface_ptr);
        let mut output = if (*layer_surface_ptr).output.is_null() {
            None
        } else {
            Some(OutputHandle::from_ptr((*layer_surface_ptr).output))
        };
        let new_surface_res = manager.new_surface(compositor, layer_surface.weak_reference(), &mut output);
        if output.is_none() {
            layer_surface.close();
            return
        }
        if let Some(layer_surface_handler) = new_surface_res {
            let mut layer_surface = LayerShell::new((layer_surface, layer_surface_handler));
            wl_signal_add(&mut (*layer_surface_ptr).events.destroy as *mut _ as _,
                          layer_surface.destroy_listener() as _);
            wl_signal_add(&mut (*layer_surface_ptr).events.map as *mut _ as _,
                          layer_surface.on_map_listener() as _);
            wl_signal_add(&mut (*layer_surface_ptr).events.unmap as *mut _ as _,
                          layer_surface.on_unmap_listener() as _);
            wl_signal_add(&mut (*layer_surface_ptr).events.new_popup as *mut _ as _,
                          layer_surface.new_popup_listener() as _);
            (*layer_surface_ptr).data = Box::into_raw(layer_surface) as *mut _;
        }
    };
]);
