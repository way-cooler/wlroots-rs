//! Handler for layer shell client.

use libc;

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{wlr_layer_surface, wlr_xdg_popup};

use {Surface, SurfaceHandle, LayerSurface, LayerSurfaceHandle, XdgShellSurface, XdgShellSurfaceHandle,
     XdgPopup, XdgShellState};
use compositor::{compositor_handle, CompositorHandle};


/// Handles events from the client layer shells.
pub trait LayerShellHandler {
    /// Called when the surface is ready to be mapped. It should be added to the list of views
    /// at this time.
    fn on_map(&mut self, CompositorHandle, SurfaceHandle, LayerSurfaceHandle) {}

    /// Called when the surface should be unmapped.
    ///
    /// It should be removed from the list of views at this time,
    /// but may be remapped at a later time.
    fn on_unmap(&mut self, CompositorHandle, SurfaceHandle, LayerSurfaceHandle) {}

    /// Called when there is a new popup.
    fn new_popup(&mut self, CompositorHandle, SurfaceHandle, LayerSurfaceHandle, XdgShellSurfaceHandle) {}

    /// Called when the Layer Shell is destroyed.
    fn destroyed(&mut self, CompositorHandle, SurfaceHandle, LayerSurfaceHandle) {}
}

wayland_listener!(LayerShell, (LayerSurface, Surface, Box<LayerShellHandler>), [
    destroy_listener => destroy_notify: |this: &mut LayerShell, data: *mut libc::c_void,| unsafe {
        let layer_surface_ptr = data as *mut wlr_layer_surface;
        {
            let (ref shell_surface, ref surface, ref mut manager) = this.data;
            let compositor = match compositor_handle() {
                Some(handle) => handle,
                None => return
            };
            manager.destroyed(compositor,
                            surface.weak_reference(),
                            shell_surface.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_map_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_unmap_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.new_popup_listener()).link as *mut _ as _);
        Box::from_raw((*layer_surface_ptr).data as *mut LayerShell);
    };
    on_map_listener => on_map_notify: |this: &mut LayerShell, _data: *mut libc::c_void,| unsafe {
        let (ref shell_surface, ref surface, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_map(compositor,
                       surface.weak_reference(),
                       shell_surface.weak_reference());
    };
    on_unmap_listener => on_unmap_notify: |this: &mut LayerShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref shell_surface, ref surface, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_unmap(compositor,
                         surface.weak_reference(),
                         shell_surface.weak_reference());
    };
    new_popup_listener => new_popup_notify: |this: &mut LayerShell, data: *mut libc::c_void,|
    unsafe {
        let (ref shell_surface, ref surface, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let popup_ptr = data as *mut wlr_xdg_popup;
        // TODO This seems really incorrect.
        // Is base right?
        // Shouldn't we store this somewhere now?
        // ugh
        let xdg_surface = (*popup_ptr).base;
        let popup = XdgPopup::from_shell(xdg_surface, popup_ptr);
        let xdg_surface = XdgShellSurface::new(xdg_surface, XdgShellState::Popup(popup));

        manager.new_popup(compositor,
                          surface.weak_reference(),
                          shell_surface.weak_reference(),
                          xdg_surface.weak_reference());
    };
]);
