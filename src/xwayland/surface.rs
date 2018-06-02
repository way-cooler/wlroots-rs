use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}};

use libc::{self, size_t, int16_t, uint16_t};

use wlroots_sys::{pid_t, wl_event_source, wlr_xwayland_surface, xcb_atom_t, xcb_window_t};

use {SurfaceHandle, XWaylandSurfaceHints, XWaylandSurfaceSizeHints};
use compositor::{compositor_handle, CompositorHandle};
use errors::{HandleErr, HandleResult};
use events::xwayland_events::{ConfigureEvent, MoveEvent, ResizeEvent};
use utils::c_to_rust_string;

pub trait XWaylandSurfaceHandler {
    /// Called when the XWayland surface is destroyed (e.g by the user).
    fn destroy(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the XWayland surface wants to be configured.
    fn on_configure(&mut self,
                    CompositorHandle,
                    SurfaceHandle,
                    XWaylandSurfaceHandle,
                    &ConfigureEvent) {
    }

    /// Called when the XWayland surface wants to move.
    fn on_move(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle, &MoveEvent) {}

    /// Called when the XWayland surface wants to be resized.
    fn on_resize(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle, &ResizeEvent) {}

    /// Called when the XWayland surface wants to be maximized.
    fn on_maximize(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the XWayland surface wants to be fullscreen.
    fn on_fullscreen(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    fn on_map(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    fn on_unmap(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the title has been set on the XWayland surface.
    fn title_set(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the class has been set on the XWayland surface.
    fn class_set(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the parent has been set on the XWayland surface.
    fn parent_set(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the PID has been set on the XWayland surface.
    fn pid_set(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the window type has been set on the XWayland surface.
    fn window_type_set(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}

    /// Called when the ping request timed out.
    ///
    /// This usually indicates something is wrong with the client.
    fn ping_timeout(&mut self, CompositorHandle, SurfaceHandle, XWaylandSurfaceHandle) {}
}

wayland_listener!(XWaylandShell, (XWaylandSurface, Box<XWaylandSurfaceHandler>), [
    request_configure_listener => request_configure_notify: |this: &mut XWaylandShell,
                                                             data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = ConfigureEvent::from_ptr(data as *mut _);
        manager.on_configure(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_move_listener => request_move_notify: |this: &mut XWaylandShell,
                                                   data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = MoveEvent::from_ptr(data as *mut _);
        manager.on_move(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_resize_listener => request_resize_notify: |this: &mut XWaylandShell,
                                                       data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let event = ResizeEvent::from_ptr(data as *mut _);
        manager.on_resize(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_maximize_listener => request_maximize_notify: |this: &mut XWaylandShell,
                                                           _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_maximize(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
    request_fullscreen_listener => request_fullscreen_notify: |this: &mut XWaylandShell,
                                                               _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_fullscreen(compositor,
                            surface,
                            shell_surface.weak_reference());
    };
    map_listener => map_notify: |this: &mut XWaylandShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_map(compositor,
                            surface,
                            shell_surface.weak_reference());
    };
    unmap_listener => unmap_notify: |this: &mut XWaylandShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_unmap(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_title_listener => set_title_notify: |this: &mut XWaylandShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.title_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_class_listener => set_class_notify: |this: &mut XWaylandShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.class_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_parent_listener => set_parent_notify: |this: &mut XWaylandShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.parent_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_pid_listener => set_pid_notify: |this: &mut XWaylandShell, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.pid_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_window_type_listener => set_window_type_notify: |this: &mut XWaylandShell,
                                                         _data: *mut libc::c_void,|
    unsafe {
        let (ref mut shell_surface, ref mut manager) = this.data;
        let surface = shell_surface.surface();
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.window_type_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    ping_timeout_listener => ping_timeout_notify: |this: &mut XWaylandShell,
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
]);

impl XWaylandShell {
    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xwayland_surface {
        self.data.0.as_ptr()
    }

    pub(crate) unsafe fn data(&mut self)
                              -> &mut (XWaylandSurface, Box<XWaylandSurfaceHandler>) {
        &mut self.data
    }
}

struct XWaylandSurfaceState {
    handle: Weak<Cell<bool>>
}

/// An Xwayland user interface component. It has an absolute position in
/// layout-local coordinates.
///
/// When a surface is ready to be displayed, the `map` event is emitted. When a
/// surface should no longer be displayed, the `unmap` event is emitted.
///
/// The `unmap` event is guaranteed to be emitted before the `destroy` event if the
/// view is destroyed when mapped.
#[derive(Debug)]
pub struct XWaylandSurface {
    liveliness: Rc<Cell<bool>>,
    shell_surface: *mut wlr_xwayland_surface
}

#[derive(Debug, Clone)]
pub struct XWaylandSurfaceHandle {
    handle: Weak<Cell<bool>>,
    shell_surface: *mut wlr_xwayland_surface
}

impl XWaylandSurface {
    pub(crate) unsafe fn new(shell_surface: *mut wlr_xwayland_surface) -> Self {
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(Cell::new(false));
        let state = Box::new(XWaylandSurfaceState { handle: Rc::downgrade(&liveliness) });
        (*shell_surface).data = Box::into_raw(state) as *mut _;
        XWaylandSurface { liveliness,
                          shell_surface }
    }

    unsafe fn from_handle(handle: &XWaylandSurfaceHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(XWaylandSurface { liveliness,
                             shell_surface: handle.as_ptr() })
    }

    /// Creates a weak reference to an `XWaylandSurface`.
    pub fn weak_reference(&self) -> XWaylandSurfaceHandle {
        XWaylandSurfaceHandle { handle: Rc::downgrade(&self.liveliness),
                                shell_surface: self.shell_surface }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_xwayland_surface {
        self.shell_surface
    }

    /// Get the window id for this surface.
    pub fn window_id(&self) -> xcb_window_t {
        unsafe { (*self.shell_surface).window_id }
    }

    /// Get the surface id for this surface.
    pub fn surface_id(&self) -> u32 {
        unsafe { (*self.shell_surface).surface_id }
    }

    /// Get the Wayland surface associated with this XWaylandSurface.
    pub fn surface(&self) -> SurfaceHandle {
        unsafe { SurfaceHandle::from_ptr((*self.shell_surface).surface) }
    }

    /// Get the coordinates of the window.
    ///
    /// Return format is (x, y)
    pub fn coords(&self) -> (int16_t, int16_t) {
        unsafe { ((*self.shell_surface).x, (*self.shell_surface).y) }
    }

    /// Get the dimensions the XWayland surface.
    ///
    /// Return format is (width, height).
    pub fn dimensions(&self) -> (uint16_t, uint16_t) {
        unsafe { ((*self.shell_surface).width, (*self.shell_surface).height) }
    }

    /// TODO What does this represent?
    ///
    /// Return format is (width, height)
    pub fn saved_dimensions(&self) -> (uint16_t, uint16_t) {
        unsafe { ((*self.shell_surface).saved_width, (*self.shell_surface).saved_height) }
    }

    /// TODO What does this represent?
    pub fn override_redirect(&self) -> bool {
        unsafe { (*self.shell_surface).override_redirect }
    }

    pub fn mapped(&self) -> bool {
        unsafe { (*self.shell_surface).mapped }
    }

    /// Get the title of the client, if there is one.
    pub fn title(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.shell_surface).title) }
    }

    /// Get the class of the client, if there is one.
    pub fn class(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.shell_surface).class) }
    }

    /// Get the instance of the client, if there is one.
    pub fn instance(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.shell_surface).instance) }
    }

    /// Get the PID associated with the client.
    pub fn pid(&self) -> pid_t {
        unsafe { (*self.shell_surface).pid }
    }

    // TODO
    //pub fn has_utf8_title(&self) -> bool {
    //    unsafe { (*self.shell_surface).has_utf8_title }
    //}

    /// Get the parent surface if there is one.
    pub fn parent(&self) -> Option<XWaylandSurfaceHandle> {
        unsafe {
            let parent_ptr = (*self.shell_surface).parent;
            if parent_ptr.is_null() {
                None
            } else {
                Some(XWaylandSurfaceHandle::from_ptr(parent_ptr))
            }
        }
    }

    /// Get the list of children surfaces.
    pub fn children(&self) -> Vec<XWaylandSurfaceHandle> {
        unsafe {
            let mut result = Vec::new();
            wl_list_for_each!((*self.shell_surface).children,
                              parent_link,
                              (child: wlr_xwayland_surface) => {
                                  result.push(XWaylandSurfaceHandle::from_ptr(child))
                              });
            result
        }
    }

    /// Get the type of the window from xcb.
    pub unsafe fn window_type(&self) -> *mut xcb_atom_t {
        (*self.shell_surface).window_type
    }

    /// Get the length of the window_type ptr
    pub unsafe fn window_type_len(&self) -> size_t {
        (*self.shell_surface).window_type_len
    }

    /// Get the protocols of the client.
    pub unsafe fn protocols(&self) -> *mut xcb_atom_t {
        (*self.shell_surface).protocols
    }

    /// Get the length of the protocols ptr.
    pub unsafe fn protocols_len(&self) -> size_t {
        (*self.shell_surface).protocols_len
    }

    /// Get the decorations on this XWayland client.
    pub fn decorations(&self) -> u32 {
        unsafe { (*self.shell_surface).decorations }
    }

    /// Get any surface hints the client is providing.
    pub fn hints<'surface>(&'surface self) -> XWaylandSurfaceHints<'surface> {
        unsafe { XWaylandSurfaceHints::from_ptr((*self.shell_surface).hints) }
    }

    /// Get any size hints the client is providing.
    pub fn size_hints<'surface>(&'surface self) -> XWaylandSurfaceSizeHints<'surface> {
        unsafe { XWaylandSurfaceSizeHints::from_ptr((*self.shell_surface).size_hints) }
    }

    /// Get the urgency of the hints.
    pub fn hints_urgency(&self) -> u32 {
        unsafe { (*self.shell_surface).hints_urgency }
    }

    pub fn pinging(&self) -> bool {
        unsafe { (*self.shell_surface).pinging }
    }

    pub unsafe fn ping_timer(&self) -> *mut wl_event_source {
        (*self.shell_surface).ping_timer
    }

    /// Determine if the client is fullscreen or not.
    pub fn fullscreen(&self) -> bool {
        unsafe { (*self.shell_surface).fullscreen }
    }

    /// Determine if the client is maximized vertically.
    pub fn maximized_vert(&self) -> bool {
        unsafe { (*self.shell_surface).maximized_vert }
    }

    /// Determine if the client is maximized horizontally.
    pub fn maximized_horz(&self) -> bool {
        unsafe { (*self.shell_surface).maximized_horz }
    }

    /// Determine if the client has an alpha channel.
    pub fn has_alpha(&self) -> bool {
        unsafe { (*self.shell_surface).has_alpha }
    }
}

impl XWaylandSurfaceHandle {
    /// Constructs a new `XWaylandSurfaceHandle` that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or for
    /// mocking/testing.
    pub fn new() -> Self {
        unsafe {
            XWaylandSurfaceHandle { handle: Weak::new(),
                                    shell_surface: ptr::null_mut() }
        }
    }

    /// Creates a `XWaylandSurfaceHandle` from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_xwayland_surface) -> Self {
        let data = (*shell_surface).data as *mut XWaylandSurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        XWaylandSurfaceHandle { handle,
                                shell_surface }
    }

    /// Upgrades the xwayland shell handle to a reference to the backing `XWaylandSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `XWaylandSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<XWaylandSurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = XWaylandSurface::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced `XWaylandSurface`, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the `XWaylandSurface` does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `XWaylandSurface`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut XWaylandSurface) -> R
    {
        let mut wl_shell_surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut wl_shell_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running XWaylandSurface callback, \
                                                    mutable lock was false for: {:?}",
                                                   wl_shell_surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_xwayland_surface {
        self.shell_surface
    }
}

impl Drop for XWaylandSurface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) > 1 {
            return
        }
        unsafe {
            Box::from_raw((*self.shell_surface).data as *mut XWaylandSurfaceState);
        }
    }
}

impl Default for XWaylandSurfaceHandle {
    fn default() -> Self {
        XWaylandSurfaceHandle::new()
    }
}

impl PartialEq for XWaylandSurfaceHandle {
    fn eq(&self, other: &XWaylandSurfaceHandle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for XWaylandSurfaceHandle {}
