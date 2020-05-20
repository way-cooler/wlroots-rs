//! Handler for stable XDG shell clients.

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::wlr_xdg_surface;

use crate::{
    compositor,
    shell::xdg_shell::{self, SurfaceState},
    surface,
    utils::Handleable
};

/// Handles events from the client stable XDG shells.
#[allow(unused_variables)]
pub trait Handler {
    /// Called when the surface recieve a request event.
    fn on_commit(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }

    /// Called when the wayland shell is destroyed (e.g by the user)
    fn destroyed(&mut self, _: compositor::Handle, _: xdg_shell::Handle) {}

    /// Called when the ping request timed out.
    ///
    /// This usually indicates something is wrong with the client.
    fn ping_timeout(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }

    /// Called when a new popup appears in the xdg tree.
    fn new_popup(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }

    /// Called when there is a request to maximize the XDG surface.
    fn maximize_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }

    /// Called when there is a request to minimize the XDG surface.
    fn minimize_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }

    /// Called when there is a request to move the shell surface somewhere else.
    fn move_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle,
        event: &xdg_shell::event::Move
    ) {
    }

    /// Called when there is a request to resize the shell surface.
    fn resize_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle,
        event: &xdg_shell::event::Resize
    ) {
    }

    /// Called when there is a request to make the shell surface fullscreen.
    fn fullscreen_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle,
        event: &xdg_shell::event::SetFullscreen
    ) {
    }

    /// Called when there is a request to show the window menu.
    fn show_window_menu_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle,
        event: &xdg_shell::event::ShowWindowMenu
    ) {
    }

    /// Called when the surface is ready to be mapped. It should be added to the
    /// list of views at this time.
    fn map_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }

    /// Called when the surface should be unmapped. It should be removed from
    /// the list of views at this time, but may be remapped at a later time.
    fn unmap_request(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: surface::Handle,
        xdg_shell_handle: xdg_shell::Handle
    ) {
    }
}

wayland_listener!(pub(crate) XdgShell, (xdg_shell::Surface, Option<Box<dyn Handler>>), [
    destroy_listener => destroy_notify: |this: &mut XdgShell, data: *mut libc::c_void,| unsafe {
        let (ref shell_surface, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        if let Some(ref mut manager) = manager.as_mut() {
            manager.destroyed(compositor, shell_surface.weak_reference());
        }
        let surface_ptr = data as *mut wlr_xdg_surface;
        let shell_state_ptr = (*surface_ptr).data as *mut SurfaceState;
        if let Some(shell_ptr) = (*shell_state_ptr).shell {
            Box::from_raw(shell_ptr.as_ptr());
        }
    };
    commit_listener => commit_notify: |this: &mut XdgShell, _data: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_commit(compositor,
                          surface,
                          shell_surface.weak_reference());
    };
    ping_timeout_listener => ping_timeout_notify: |this: &mut XdgShell,
                                                   _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.ping_timeout(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
    new_popup_listener => new_popup_notify: |this: &mut XdgShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.new_popup(compositor,
                          surface,
                          shell_surface.weak_reference());
    };
    maximize_listener => maximize_notify: |this: &mut XdgShell, _event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.maximize_request(compositor,
                                 surface,
                                 shell_surface.weak_reference());
    };
    fullscreen_listener => fullscreen_notify: |this: &mut XdgShell, event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let event = xdg_shell::event::SetFullscreen::from_ptr(event as _);

        manager.fullscreen_request(compositor,
                                   surface,
                                   shell_surface.weak_reference(),
                                   &event);
    };
    minimize_listener => minimize_notify: |this: &mut XdgShell, _event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.minimize_request(compositor,
                                 surface,
                                 shell_surface.weak_reference());
    };
    move_listener => move_notify: |this: &mut XdgShell, event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let event = xdg_shell::event::Move::from_ptr(event as _);

        manager.move_request(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    resize_listener => resize_notify: |this: &mut XdgShell, event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let event = xdg_shell::event::Resize::from_ptr(event as _);

        manager.resize_request(compositor,
                               surface,
                               shell_surface.weak_reference(),
                               &event);
    };
    show_window_menu_listener => show_window_menu_notify: |this: &mut XdgShell,
                                                           event: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let event = xdg_shell::event::ShowWindowMenu::from_ptr(event as _);

        manager.show_window_menu_request(compositor,
                                         surface,
                                         shell_surface.weak_reference(),
                                         &event);
    };

    map_listener => map_notify: |this: &mut XdgShell, _event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.map_request(compositor,
                            surface,
                            shell_surface.weak_reference());
    };

    unmap_listener => unmap_notify: |this: &mut XdgShell, _event: *mut libc::c_void,| unsafe {
        let (ref mut shell_surface, ref mut manager) = match &mut this.data {
            (_, None) => return,
            (ss, Some(manager)) => (ss, manager)
        };
        let surface = shell_surface.surface();
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.unmap_request(compositor,
                              surface,
                              shell_surface.weak_reference());
    };
]);

impl XdgShell {
    pub(crate) fn surface_mut(&mut self) -> xdg_shell::Handle {
        self.data.0.weak_reference()
    }
}

impl Drop for XdgShell {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.destroy_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.commit_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.ping_timeout_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.new_popup_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.maximize_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.fullscreen_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.minimize_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.move_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.resize_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.show_window_menu_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.map_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.unmap_listener()).link as *mut _ as _
            );
        }
    }
}
