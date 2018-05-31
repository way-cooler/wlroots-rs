//! Handler for layer shell client.

use libc;

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
}

wayland_listener!(LayerShell, (LayerSurface, Surface, Box<LayerShellHandler>), [
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

impl LayerShell {
    pub(crate) unsafe fn surface_ptr(&self) -> *mut wlr_layer_surface {
        self.data.0.as_ptr()
    }

    pub(crate) fn surface_mut(&mut self) -> LayerSurfaceHandle {
        self.data.0.weak_reference()
    }
}
