//! Handler for XDG shell v6 clients.

use libc;

use wlroots_sys::wlr_xdg_surface_v6;

use {SurfaceHandle, XdgV6ShellSurface, XdgV6ShellSurfaceHandle};
use compositor::{compositor_handle, CompositorHandle};
use xdg_shell_v6_events::{MoveEvent, ResizeEvent, SetFullscreenEvent, ShowWindowMenuEvent};

/// Handles events from the client XDG v6 shells.
pub trait XdgV6ShellHandler {
    /// Called when the surface recieve a request event.
    fn on_commit(&mut self, CompositorHandle, SurfaceHandle, XdgV6ShellSurfaceHandle) {}

    /// Called when the wayland shell is destroyed (e.g by the user)

    fn destroy(&mut self, CompositorHandle, SurfaceHandle, XdgV6ShellSurfaceHandle) {}

    /// Called when the ping request timed out.
    ///
    /// This usually indicates something is wrong with the client.
    fn ping_timeout(&mut self, CompositorHandle, SurfaceHandle, XdgV6ShellSurfaceHandle) {}

    /// Called when a new popup appears in the xdg tree.
    fn new_popup(&mut self, CompositorHandle, SurfaceHandle, XdgV6ShellSurfaceHandle) {}

    /// Called when there is a request to maximize the XDG surface.
    fn maximize_request(&mut self, CompositorHandle, SurfaceHandle, XdgV6ShellSurfaceHandle) {}

    /// Called when there is a request to minimize the XDG surface.
    fn minimize_request(&mut self, CompositorHandle, SurfaceHandle, XdgV6ShellSurfaceHandle) {}

    /// Called when there is a request to move the shell surface somewhere else.
    fn move_request(&mut self,
                    CompositorHandle,
                    SurfaceHandle,
                    XdgV6ShellSurfaceHandle,
                    &MoveEvent) {
    }

    /// Called when there is a request to resize the shell surface.
    fn resize_request(&mut self,
                      CompositorHandle,
                      SurfaceHandle,
                      XdgV6ShellSurfaceHandle,
                      &ResizeEvent) {
    }

    /// Called when there is a request to make the shell surface fullscreen.
    fn fullscreen_request(&mut self,
                          CompositorHandle,
                          SurfaceHandle,
                          XdgV6ShellSurfaceHandle,
                          &SetFullscreenEvent) {
    }

    /// Called when there is a request to show the window menu.
    fn show_window_menu_request(&mut self,
                                CompositorHandle,
                                SurfaceHandle,
                                XdgV6ShellSurfaceHandle,
                                &ShowWindowMenuEvent) {
    }

    /// Called when the surface is ready to be mapped. It should be added to the list of views at
    /// this time.
    fn map_request(&mut self,
                   CompositorHandle,
                   SurfaceHandle,
                   XdgV6ShellSurfaceHandle) {
    }

    /// Called when the surface should be unmapped. It should be removed from the list of views at
    /// this time, but may be remapped at a later time.
    fn unmap_request(&mut self,
                   CompositorHandle,
                   SurfaceHandle,
                   XdgV6ShellSurfaceHandle) {
    }
}

wayland_listener!(XdgV6Shell, (XdgV6ShellSurface, Box<XdgV6ShellHandler>), [
    commit_listener => commit_notify: |this: &mut XdgV6Shell, _data: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_commit(compositor,
                          surface,
                          shell_surface.weak_reference());
    };
    ping_timeout_listener => ping_timeout_notify: |this: &mut XdgV6Shell,
                                                   _data: *mut libc::c_void,|
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
    new_popup_listener => new_popup_notify: |this: &mut XdgV6Shell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.new_popup(compositor,
                          surface,
                          shell_surface.weak_reference());
    };
    maximize_listener => maximize_notify: |this: &mut XdgV6Shell, _event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.maximize_request(compositor,
                                 surface,
                                 shell_surface.weak_reference());
    };
    fullscreen_listener => fullscreen_notify: |this: &mut XdgV6Shell, event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = SetFullscreenEvent::from_ptr(event as _);

        manager.fullscreen_request(compositor,
                                   surface,
                                   shell_surface.weak_reference(),
                                   &event);
    };
    minimize_listener => minimize_notify: |this: &mut XdgV6Shell, _event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.minimize_request(compositor,
                                 surface,
                                 shell_surface.weak_reference());
    };
    move_listener => move_notify: |this: &mut XdgV6Shell, event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = MoveEvent::from_ptr(event as _);

        manager.move_request(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    resize_listener => resize_notify: |this: &mut XdgV6Shell, event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = ResizeEvent::from_ptr(event as _);

        manager.resize_request(compositor,
                               surface,
                               shell_surface.weak_reference(),
                               &event);
    };
    show_window_menu_listener => show_window_menu_notify: |this: &mut XdgV6Shell,
                                                           event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = ShowWindowMenuEvent::from_ptr(event as _);

        manager.show_window_menu_request(compositor,
                                         surface,
                                         shell_surface.weak_reference(),
                                         &event);
    };

    map_listener => map_notify: |this: &mut XdgV6Shell, _event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.map_request(compositor,
                            surface,
                            shell_surface.weak_reference());
    };

    unmap_listener => unmap_notify: |this: &mut XdgV6Shell, _event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };

        manager.unmap_request(compositor,
                            surface,
                            shell_surface.weak_reference());
    };
]);

impl XdgV6Shell {
    pub(crate) unsafe fn surface_ptr(&self) -> *mut wlr_xdg_surface_v6 {
        self.data.0.as_ptr()
    }

    pub(crate) fn surface_mut(&mut self) -> XdgV6ShellSurfaceHandle {
        self.data.0.weak_reference()
    }
}
