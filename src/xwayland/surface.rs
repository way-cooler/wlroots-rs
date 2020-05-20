use std::{
    cell::Cell,
    ptr::NonNull,
    rc::{Rc, Weak}
};

use crate::libc::{self, size_t};

use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{
    pid_t, wl_event_source, wlr_xwayland_surface, wlr_xwayland_surface_activate,
    wlr_xwayland_surface_configure, xcb_atom_t, xcb_window_t
};

pub use crate::xwayland::hints::{Hints, SizeHints};
use crate::{
    area::{Area, Origin, Size},
    compositor,
    surface::{self, InternalState},
    utils::{self, c_to_rust_string, HandleErr, HandleResult, Handleable},
    xwayland
};

pub type Handle = utils::Handle<(), wlr_xwayland_surface, Surface>;

#[allow(unused_variables)]
pub trait Handler {
    /// Called when the XWayland surface is destroyed (e.g by the user).
    fn destroyed(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the XWayland surface wants to be configured.
    fn on_configure(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle,
        configure: &xwayland::event::Configure
    ) {
    }

    /// Called when the XWayland surface wants to move.
    fn on_move(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle,
        event: &xwayland::event::Move
    ) {
    }

    /// Called when the XWayland surface wants to be resized.
    fn on_resize(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle,
        event: &xwayland::event::Resize
    ) {
    }

    /// Called when the XWayland surface wants to be maximized.
    fn on_maximize(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the XWayland surface wants to be fullscreen.
    fn on_fullscreen(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    fn on_map(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) -> Option<Box<dyn surface::Handler>> {
        None
    }

    fn on_unmap(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the title has been set on the XWayland surface.
    fn title_set(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the class has been set on the XWayland surface.
    fn class_set(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the parent has been set on the XWayland surface.
    fn parent_set(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the PID has been set on the XWayland surface.
    fn pid_set(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the window type has been set on the XWayland surface.
    fn window_type_set(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }

    /// Called when the ping request timed out.
    ///
    /// This usually indicates something is wrong with the client.
    fn ping_timeout(
        &mut self,
        compositor_handle: compositor::Handle,
        surface_handle: Option<surface::Handle>,
        xwayland_surface_handle: Handle
    ) {
    }
}

wayland_listener!(pub(crate) Shell, (Surface, Option<Box<dyn Handler>>), [
    destroy_listener => destroy_notify: |this: &mut Shell, data: *mut libc::c_void,|
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
        manager.destroyed(compositor, surface, shell_surface.weak_reference());
        let surface_ptr = data as *mut wlr_xwayland_surface;
        let shell_state_ptr = (*surface_ptr).data as *mut State;
        if let Some(shell_ptr) = (*shell_state_ptr).shell {
            Box::from_raw(shell_ptr.as_ptr());
        }
    };
    request_configure_listener => request_configure_notify: |this: &mut Shell,
                                                             data: *mut libc::c_void,|
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
        let event = xwayland::event::Configure::from_ptr(data as *mut _);
        manager.on_configure(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_move_listener => request_move_notify: |this: &mut Shell,
                                                   data: *mut libc::c_void,|
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
        let event = xwayland::event::Move::from_ptr(data as *mut _);
        manager.on_move(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_resize_listener => request_resize_notify: |this: &mut Shell,
                                                       data: *mut libc::c_void,|
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
        let event = xwayland::event::Resize::from_ptr(data as *mut _);
        manager.on_resize(compositor,
                             surface,
                             shell_surface.weak_reference(),
                             &event);
    };
    request_maximize_listener => request_maximize_notify: |this: &mut Shell,
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
        manager.on_maximize(compositor,
                             surface,
                             shell_surface.weak_reference());
    };
    request_fullscreen_listener => request_fullscreen_notify: |this: &mut Shell,
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
        manager.on_fullscreen(compositor,
                            surface,
                            shell_surface.weak_reference());
    };
    map_listener => map_notify: |this: &mut Shell, _data: *mut libc::c_void,|
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
        let surface_handler = manager.on_map(compositor,
                                             surface,
                                             shell_surface.weak_reference());

        if let Some(surface_handler) = surface_handler {
            let surface_state = (*(*shell_surface.shell_surface.as_ptr()).surface).data as *mut InternalState;
            (*(*surface_state).surface.unwrap().as_ptr()).data().1 = surface_handler;
        }

    };
    unmap_listener => unmap_notify: |this: &mut Shell, _data: *mut libc::c_void,|
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
        manager.on_unmap(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_title_listener => set_title_notify: |this: &mut Shell, _data: *mut libc::c_void,|
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
        manager.title_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_class_listener => set_class_notify: |this: &mut Shell, _data: *mut libc::c_void,|
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
        manager.class_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_parent_listener => set_parent_notify: |this: &mut Shell, _data: *mut libc::c_void,|
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
        manager.parent_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_pid_listener => set_pid_notify: |this: &mut Shell, _data: *mut libc::c_void,|
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
        manager.pid_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    set_window_type_listener => set_window_type_notify: |this: &mut Shell,
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
        manager.window_type_set(compositor,
                       surface,
                       shell_surface.weak_reference());
    };
    ping_timeout_listener => ping_timeout_notify: |this: &mut Shell,
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
]);

pub(crate) struct State {
    pub(crate) shell: Option<NonNull<Shell>>,
    handle: Weak<Cell<bool>>
}

/// An Xwayland user interface component. It has an absolute position in
/// layout-local coordinates.
///
/// When a surface is ready to be displayed, the `map` event is emitted. When a
/// surface should no longer be displayed, the `unmap` event is emitted.
///
/// The `unmap` event is guaranteed to be emitted before the `destroy` event if
/// the view is destroyed when mapped.
#[derive(Debug)]
pub struct Surface {
    liveliness: Rc<Cell<bool>>,
    shell_surface: NonNull<wlr_xwayland_surface>
}

impl Surface {
    pub(crate) unsafe fn new(shell_surface: *mut wlr_xwayland_surface) -> Self {
        let shell_surface = NonNull::new(shell_surface).expect("Shell Surface pointer was null");
        if !(*shell_surface.as_ptr()).data.is_null() {
            panic!("Shell surface has already been created");
        }
        let liveliness = Rc::new(Cell::new(false));
        let state = Box::new(State {
            shell: None,
            handle: Rc::downgrade(&liveliness)
        });
        (*shell_surface.as_ptr()).data = Box::into_raw(state) as *mut _;
        Surface {
            liveliness,
            shell_surface
        }
    }

    /// Get the window id for this surface.
    pub fn window_id(&self) -> xcb_window_t {
        unsafe { (*self.shell_surface.as_ptr()).window_id }
    }

    /// Get the surface id for this surface.
    pub fn surface_id(&self) -> u32 {
        unsafe { (*self.shell_surface.as_ptr()).surface_id }
    }

    /// Get the Wayland surface associated with this Surface. If the shell
    /// surface is not mapped, then it has no surface, and this will return
    /// None.
    pub fn surface(&self) -> Option<surface::Handle> {
        unsafe {
            let surface = (*self.shell_surface.as_ptr()).surface;

            if surface.is_null() {
                None
            } else {
                Some(surface::Handle::from_ptr((*self.shell_surface.as_ptr()).surface))
            }
        }
    }

    /// Get the coordinates of the window.
    ///
    /// Return format is (x, y)
    pub fn coords(&self) -> (i16, i16) {
        unsafe { ((*self.shell_surface.as_ptr()).x, (*self.shell_surface.as_ptr()).y) }
    }

    /// Get the dimensions the XWayland surface.
    ///
    /// Return format is (width, height).
    pub fn dimensions(&self) -> (u16, u16) {
        unsafe {
            (
                (*self.shell_surface.as_ptr()).width,
                (*self.shell_surface.as_ptr()).height
            )
        }
    }

    /// TODO What does this represent?
    ///
    /// Return format is (width, height)
    pub fn saved_dimensions(&self) -> (u16, u16) {
        unsafe {
            (
                (*self.shell_surface.as_ptr()).saved_width,
                (*self.shell_surface.as_ptr()).saved_height
            )
        }
    }

    /// TODO What does this represent?
    pub fn override_redirect(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).override_redirect }
    }

    pub fn mapped(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).mapped }
    }

    /// Get the title of the client, if there is one.
    pub fn title(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.shell_surface.as_ptr()).title) }
    }

    /// Get the class of the client, if there is one.
    pub fn class(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.shell_surface.as_ptr()).class) }
    }

    /// Get the instance of the client, if there is one.
    pub fn instance(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.shell_surface.as_ptr()).instance) }
    }

    /// Get the PID associated with the client.
    pub fn pid(&self) -> pid_t {
        unsafe { (*self.shell_surface.as_ptr()).pid }
    }

    // TODO
    // pub fn has_utf8_title(&self) -> bool {
    //    unsafe { (*self.shell_surface.as_ptr()).has_utf8_title }
    //}

    /// Get the parent surface if there is one.
    pub fn parent(&self) -> Option<Handle> {
        unsafe {
            let parent_ptr = (*self.shell_surface.as_ptr()).parent;
            if parent_ptr.is_null() {
                None
            } else {
                Some(Handle::from_ptr(parent_ptr))
            }
        }
    }

    /// Get the list of children surfaces.
    pub fn children(&self) -> Vec<Handle> {
        unsafe {
            let mut result = Vec::new();
            wl_list_for_each!((*self.shell_surface.as_ptr()).children,
            parent_link,
            (child: wlr_xwayland_surface) => {
                result.push(Handle::from_ptr(child))
            });
            result
        }
    }

    /// Get the type of the window from xcb.
    pub unsafe fn window_type(&self) -> *mut xcb_atom_t {
        (*self.shell_surface.as_ptr()).window_type
    }

    /// Get the length of the window_type ptr
    pub unsafe fn window_type_len(&self) -> size_t {
        (*self.shell_surface.as_ptr()).window_type_len
    }

    /// Get the protocols of the client.
    pub unsafe fn protocols(&self) -> *mut xcb_atom_t {
        (*self.shell_surface.as_ptr()).protocols
    }

    /// Get the length of the protocols ptr.
    pub unsafe fn protocols_len(&self) -> size_t {
        (*self.shell_surface.as_ptr()).protocols_len
    }

    /// Get the decorations on this XWayland client.
    pub fn decorations(&self) -> u32 {
        unsafe { (*self.shell_surface.as_ptr()).decorations }
    }

    /// Get any surface hints the client is providing.
    pub fn hints(&self) -> xwayland::surface::Hints {
        unsafe { xwayland::surface::Hints::from_ptr((*self.shell_surface.as_ptr()).hints) }
    }

    /// Get any size hints the client is providing.
    pub fn size_hints(&self) -> xwayland::surface::SizeHints {
        unsafe { xwayland::surface::SizeHints::from_ptr((*self.shell_surface.as_ptr()).size_hints) }
    }

    /// Get the urgency of the hints.
    pub fn hints_urgency(&self) -> u32 {
        unsafe { (*self.shell_surface.as_ptr()).hints_urgency }
    }

    pub fn pinging(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).pinging }
    }

    pub unsafe fn ping_timer(&self) -> *mut wl_event_source {
        (*self.shell_surface.as_ptr()).ping_timer
    }

    /// Determine if the client is fullscreen or not.
    pub fn fullscreen(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).fullscreen }
    }

    /// Determine if the client is maximized vertically.
    pub fn maximized_vert(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).maximized_vert }
    }

    /// Determine if the client is maximized horizontally.
    pub fn maximized_horz(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).maximized_horz }
    }

    /// Determine if the client has an alpha channel.
    pub fn has_alpha(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).has_alpha }
    }

    /// Geometry of the surface in layout-local coordinates
    pub fn geometry(&self) -> Area {
        let (x, y, width, height) = unsafe {
            (
                i32::from((*self.shell_surface.as_ptr()).x),
                i32::from((*self.shell_surface.as_ptr()).y),
                i32::from((*self.shell_surface.as_ptr()).width),
                i32::from((*self.shell_surface.as_ptr()).height)
            )
        };
        Area {
            origin: Origin { x, y },
            size: Size { width, height }
        }
    }

    /// Send the surface a configure request, requesting the new position and
    /// dimensions
    pub fn configure(&self, x: i16, y: i16, width: u16, height: u16) {
        unsafe {
            wlr_xwayland_surface_configure(self.shell_surface.as_ptr(), x, y, width, height);
        }
    }

    /// Tell the window whether it is the foucsed window
    pub fn set_activated(&self, active: bool) {
        unsafe {
            wlr_xwayland_surface_activate(self.shell_surface.as_ptr(), active);
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) > 1 {
            return;
        }
        unsafe {
            Box::from_raw((*self.shell_surface.as_ptr()).data as *mut State);
        }
    }
}

impl Handleable<(), wlr_xwayland_surface> for Surface {
    #[doc(hidden)]
    unsafe fn from_ptr(shell_surface: *mut wlr_xwayland_surface) -> Option<Self> {
        let shell_surface = NonNull::new(shell_surface)?;
        let data = (*shell_surface.as_ptr()).data as *mut State;
        let liveliness = (*data).handle.upgrade().unwrap();
        Some(Surface {
            liveliness,
            shell_surface
        })
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_xwayland_surface {
        self.shell_surface.as_ptr()
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle.upgrade().ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Surface {
            liveliness,
            shell_surface: handle.as_non_null()
        })
    }

    /// Creates a weak reference to an `Surface`.
    fn weak_reference(&self) -> Handle {
        Handle {
            ptr: self.shell_surface,
            handle: Rc::downgrade(&self.liveliness),
            _marker: std::marker::PhantomData,
            data: Some(())
        }
    }
}

impl Drop for Shell {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.destroy_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.request_configure_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.request_move_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.request_resize_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.request_maximize_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.request_fullscreen_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.map_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.unmap_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.set_title_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.set_class_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.set_parent_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.set_pid_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.set_window_type_listener() as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                self.ping_timeout_listener() as *mut _ as _
            );
        }
    }
}
