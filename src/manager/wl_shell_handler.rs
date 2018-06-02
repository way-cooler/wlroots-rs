//! Handler for Wayland shell clients.

use libc;
use wayland_sys::server::WAYLAND_SERVER_HANDLE;

use {SurfaceHandle, WlShellSurface, WlShellSurfaceHandle};
use compositor::{compositor_handle, CompositorHandle};
use wl_shell_events::{FullscreenEvent, MaximizeEvent, MoveEvent, ResizeEvent};

/// Handles events from client Wayland shells.
pub trait WlShellHandler {
    /// Called when the Wayland shell is destroyed (e.g by the user)
    fn destroy(&mut self, CompositorHandle, SurfaceHandle, WlShellSurfaceHandle) {}

    /// Called when the ping request timed out.
    ///
    /// This usually indicates something is wrong with the client
    fn ping_timeout(&mut self, CompositorHandle, SurfaceHandle, WlShellSurfaceHandle) {}

    /// Called when there is a request to move the shell surface somewhere else.
    fn move_request(&mut self, CompositorHandle, SurfaceHandle, WlShellSurfaceHandle, &MoveEvent) {}

    /// Called when there is a request to resize the shell surface.
    fn resize_request(&mut self,
                      CompositorHandle,
                      SurfaceHandle,
                      WlShellSurfaceHandle,
                      &ResizeEvent) {
    }

    /// Called when there is a request to make the shell surface fullscreen.
    fn fullscreen_request(&mut self,
                          CompositorHandle,
                          SurfaceHandle,
                          WlShellSurfaceHandle,
                          &FullscreenEvent) {
    }

    /// Called when there is a request to make the shell surface maximized.
    fn maximize_request(&mut self,
                        CompositorHandle,
                        SurfaceHandle,
                        WlShellSurfaceHandle,
                        &MaximizeEvent) {
    }

    /// Called when there is a request to change the state of the Wayland shell.
    fn state_change(&mut self, CompositorHandle, SurfaceHandle, WlShellSurfaceHandle) {}

    /// Called when there is a request to change the title of the Wayland shell.
    fn title_change(&mut self, CompositorHandle, SurfaceHandle, WlShellSurfaceHandle) {}

    /// Called when there is a request to change the class of the Wayland shell.
    fn class_change(&mut self, CompositorHandle, SurfaceHandle, WlShellSurfaceHandle) {}
}

wayland_listener!(WlShell, (WlShellSurface, Box<WlShellHandler>), [
    destroy_listener => destroy_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        // TODO NLL
        {
            let (ref mut shell_surface, ref mut manager) = this.data;
            let surface = shell_surface.surface();
            let compositor = match compositor_handle() {
                Some(handle) => handle,
                None => return
            };

            manager.destroy(compositor,
                            surface,
                            shell_surface.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.ping_timeout_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_move_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_resize_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_fullscreen_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.request_maximize_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.set_state_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.set_title_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.set_class_listener()).link as *mut _ as _);
        let shell_ptr = this as *mut _;
        drop(this);
        // Destroy the WlShell data. This is necessary because WlShellManager doesn't
        // have an event to listen to Wayland shell destruction.
        // NOTE **DO NOT** use `this` after this line.
        let _ = Box::from_raw(shell_ptr);
    };
    ping_timeout_listener => ping_timeout_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.ping_timeout(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
    request_move_listener => request_move_notify: |this: &mut WlShell, data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = MoveEvent::from_ptr(data as _);

        manager.move_request(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_resize_listener => request_resize_notify: |this: &mut WlShell,
                                                       data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = ResizeEvent::from_ptr(data as _);

        manager.resize_request(compositor,
                               surface,
                               shell_surface.weak_reference(),
                               &event);
    };
    request_fullscreen_listener => request_fullscreen_notify: |this: &mut WlShell,
                                                               data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = FullscreenEvent::from_ptr(data as _);

        manager.fullscreen_request(compositor,
                                   surface,
                                   shell_surface.weak_reference(),
                                   &event);
    };
    request_maximize_listener => request_maximize_notify: |this: &mut WlShell,
                                                           data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = MaximizeEvent::from_ptr(data as _);

        manager.maximize_request(compositor,
                                 surface,
                                 shell_surface.weak_reference(),
                                 &event);
    };
    set_state_listener => set_state_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.state_change(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
    set_title_listener => set_title_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.title_change(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
    set_class_listener => set_class_notify: |this: &mut WlShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.class_change(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
]);
