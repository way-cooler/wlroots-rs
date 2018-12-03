//! TODO Documentation

use std::{panic, ptr};
use std::cell::Cell;
use std::rc::{Rc, Weak};

use wlroots_sys::{wlr_xdg_popup, wlr_xdg_surface, wlr_xdg_surface_ping,
                  wlr_xdg_surface_role, wlr_xdg_surface_send_close,
                  wlr_xdg_surface_surface_at, wlr_xdg_toplevel,
                  wlr_xdg_toplevel_set_activated, wlr_xdg_toplevel_set_fullscreen,
                  wlr_xdg_toplevel_set_maximized, wlr_xdg_toplevel_set_resizing,
                  wlr_xdg_toplevel_set_size, wlr_xdg_toplevel_state,
                  wlr_xdg_surface_for_each_surface, wlr_surface, wlr_xdg_surface_from_wlr_surface};

use {Area, SeatHandle, SurfaceHandle, Surface};
use errors::{HandleErr, HandleResult};
use utils::c_to_rust_string;
use manager::XdgShell;
use libc::c_void;

/// Used internally to reclaim a handle from just a *mut wlr_xdg_surface.
pub(crate) struct XdgShellSurfaceState {
    /// Pointer to the backing storage.
    pub(crate) shell: *mut XdgShell,
    handle: Weak<Cell<bool>>,
    shell_state: Option<XdgShellState>
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct XdgTopLevel {
    shell_surface: *mut wlr_xdg_surface,
    toplevel: *mut wlr_xdg_toplevel
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct XdgPopup {
    shell_surface: *mut wlr_xdg_surface,
    popup: *mut wlr_xdg_popup
}

/// A tagged enum of the different roles used by the xdg shell.
///
/// Uses the tag to disambiguate the union in `wlr_xdg_surface`.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum XdgShellState {
    TopLevel(XdgTopLevel),
    /// A top level that has been destroyed. The state is not dropped but none
    /// of the functions can be used.
    ///
    /// If the client reuses the xdg surface it must make a top level as the role,
    /// so this could become non-inert again.
    TopLevelUnmapped(XdgTopLevel),
    Popup(XdgPopup),
    /// A popup that has been destroyed. The state is not dropped but none
    /// of the functions can be used.
    ///
    /// If the client reuses the xdg surface it must make a popup as the role,
    /// so this could become non-inert again.
    PopupUnmapped(XdgPopup)
}

#[derive(Debug)]
pub struct XdgShellSurface {
    liveliness: Rc<Cell<bool>>,
    state: Option<XdgShellState>,
    shell_surface: *mut wlr_xdg_surface
}

#[derive(Debug)]
pub struct XdgShellSurfaceHandle {
    state: Option<XdgShellState>,
    handle: Weak<Cell<bool>>,
    shell_surface: *mut wlr_xdg_surface
}

impl Clone for XdgShellSurfaceHandle {
    fn clone(&self) -> Self {
        let state = match self.state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        XdgShellSurfaceHandle { state,
                                  handle: self.handle.clone(),
                                  shell_surface: self.shell_surface }
    }
}

impl XdgShellSurface {
    pub(crate) unsafe fn new<T>(shell_surface: *mut wlr_xdg_surface, state: T) -> Self
        where T: Into<Option<XdgShellState>>
    {
        let state = state.into();
        // TODO FIXME Free state in drop impl when Rc == 1
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(Cell::new(false));
        let shell_state =
            Box::new(XdgShellSurfaceState { shell: ptr::null_mut(),
                                            handle: Rc::downgrade(&liveliness),
                                            shell_state: match state {
                                                None => None,
                                                Some(ref state) => Some(state.clone())
                                            } });
        (*shell_surface).data = Box::into_raw(shell_state) as *mut _;
        XdgShellSurface { liveliness,
                          state: state,
                          shell_surface }
    }

    unsafe fn from_handle(handle: &XdgShellSurfaceHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(XdgShellSurface { liveliness,
                               state: handle.clone().state,
                               shell_surface: handle.as_ptr() })
    }

    /// Gets the surface used by this XDG shell.
    pub fn surface(&mut self) -> SurfaceHandle {
        unsafe {
            let surface = (*self.shell_surface).surface;
            if surface.is_null() {
                panic!("xdg shell had a null surface!")
            }
            SurfaceHandle::from_ptr(surface)
        }
    }

    /// Get the role of this XDG surface.
    pub fn role(&self) -> wlr_xdg_surface_role {
        unsafe { (*self.shell_surface).role }
    }

    pub fn state(&mut self) -> Option<&mut XdgShellState> {
        self.state.as_mut()
    }

    /// Determines if this XDG shell surface has been configured or not.
    pub fn configured(&self) -> bool {
        unsafe { (*self.shell_surface).configured }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.shell_surface).added }
    }

    pub fn configure_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).configure_serial }
    }

    pub fn configure_next_serial(&self) -> u32 {
        unsafe { (*self.shell_surface).configure_next_serial }
    }

    pub fn has_next_geometry(&self) -> bool {
        unsafe { (*self.shell_surface).has_next_geometry }
    }

    pub fn next_geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface).next_geometry) }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface).geometry) }
    }

    /// Send a ping to the surface.
    ///
    /// If the surface does not respond with a pong within a reasonable amount of time,
    /// the ping timeout event will be emitted.
    pub fn ping(&mut self) {
        unsafe {
            wlr_xdg_surface_ping(self.shell_surface);
        }
    }

    /// Find a surface within this surface at the surface-local coordinates.
    ///
    /// Returns the popup and coordinates in the topmost surface coordinate system
    /// or None if no popup is found at that location.
    pub fn surface_at(&mut self,
                      sx: f64,
                      sy: f64,
                      sub_sx: &mut f64,
                      sub_sy: &mut f64)
                      -> Option<SurfaceHandle> {
        unsafe {
            let sub_surface =
                wlr_xdg_surface_surface_at(self.shell_surface, sx, sy, sub_sx, sub_sy);
            if sub_surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(sub_surface))
            }
        }
    }

    pub fn for_each_surface<F>(&self, mut iterator: F)
            where F: FnMut(SurfaceHandle, i32, i32) {
        let mut iterator_ref: &mut FnMut(SurfaceHandle, i32, i32) = &mut iterator;
        unsafe {
            unsafe extern "C" fn c_iterator(wlr_surface: *mut wlr_surface, sx: i32, sy: i32, data: *mut c_void) {
                let iterator_fn = &mut *(data as *mut &mut FnMut(SurfaceHandle, i32, i32));
                let surface = SurfaceHandle::from_ptr(wlr_surface);
                iterator_fn(surface, sx, sy);
            }
            let iterator_ptr: *mut c_void = &mut iterator_ref as *mut _ as *mut c_void;
            wlr_xdg_surface_for_each_surface(self.shell_surface, Some(c_iterator), iterator_ptr);
        }
    }

    /// Creates a weak reference to an `XdgShellSurface`.
    ///
    /// # Panics
    /// If this `XdgShellSurface` is a previously upgraded `XdgShellSurfaceHandle`,
    /// then this function will panic.
    pub fn weak_reference(&self) -> XdgShellSurfaceHandle {
        XdgShellSurfaceHandle { handle: Rc::downgrade(&self.liveliness),
                                  state: match self.state {
                                      None => None,
                                      Some(ref state) => unsafe { Some(state.clone()) }
                                  },
                                  shell_surface: self.shell_surface }
    }
}

impl XdgShellSurfaceHandle {
    /// Constructs a new XdgShellSurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            XdgShellSurfaceHandle { handle: Weak::new(),
                                      state: None,
                                      shell_surface: ptr::null_mut() }
        }
    }

    /// If the surface is an XDG surface, get a handle to the XDG surface.
    pub fn from_surface(surface: &Surface) -> Option<XdgShellSurfaceHandle> {
        unsafe {
            if !surface.is_xdg_surface() {
                None
            } else {
                let xdg_surface_ptr = wlr_xdg_surface_from_wlr_surface(surface.as_ptr());
                Some(XdgShellSurfaceHandle::from_ptr(xdg_surface_ptr))
            }
        }
    }

    /// Creates a XdgShellSurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_xdg_surface) -> Self {
        let data = (*shell_surface).data as *mut XdgShellSurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        let state = match (*data).shell_state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        XdgShellSurfaceHandle { handle,
                                  state,
                                  shell_surface }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `XdgShellSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `XdgShellSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<XdgShellSurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = XdgShellSurface::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced XdgShellSurface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the XdgShellSurface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut XdgShellSurface) -> R
    {
        use wlroots_sys::wlr_xdg_surface_role::*;
        let mut xdg_surface = unsafe { self.upgrade()? };
        match (xdg_surface.role(), xdg_surface.state.take()) {
            (WLR_XDG_SURFACE_ROLE_NONE, Some(XdgShellState::TopLevel(toplevel))) => {
                xdg_surface.state = Some(XdgShellState::TopLevelUnmapped(toplevel))
            }
            (WLR_XDG_SURFACE_ROLE_TOPLEVEL, Some(XdgShellState::TopLevelUnmapped(toplevel))) => {
                xdg_surface.state = Some(XdgShellState::TopLevel(toplevel))
            }
            (WLR_XDG_SURFACE_ROLE_NONE, Some(XdgShellState::Popup(popup))) => {
                xdg_surface.state = Some(XdgShellState::PopupUnmapped(popup))
            }
            (WLR_XDG_SURFACE_ROLE_POPUP, Some(XdgShellState::PopupUnmapped(popup))) => {
                xdg_surface.state = Some(XdgShellState::Popup(popup))
            }
            (_, state) => {
                xdg_surface.state = state
            }
        }
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut xdg_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(WLR_ERROR,
                                                   "After running XdgShellSurface callback, \
                                                    mutable lock was false for: {:?}",
                                                   xdg_surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_xdg_surface {
        self.shell_surface
    }
}

impl Default for XdgShellSurfaceHandle {
    fn default() -> Self {
        XdgShellSurfaceHandle::new()
    }
}

impl PartialEq for XdgShellSurfaceHandle {
    fn eq(&self, other: &XdgShellSurfaceHandle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for XdgShellSurfaceHandle {}

impl XdgTopLevel {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface,
                                    toplevel: *mut wlr_xdg_toplevel)
                                    -> XdgTopLevel {
        let toplevel = XdgTopLevel { shell_surface,
                                     toplevel };
        toplevel.assert_toplevel();
        toplevel
    }

    /// Get the title associated with this XDG shell toplevel.
    pub fn title(&self) -> String {
        self.assert_toplevel();
        unsafe { c_to_rust_string((*self.toplevel).title).expect("Could not parse class as UTF-8") }
    }

    /// Get the app id associated with this XDG shell toplevel.
    pub fn app_id(&self) -> String {
        self.assert_toplevel();
        unsafe {
            c_to_rust_string((*self.toplevel).app_id).expect("Could not parse class as UTF-8")
        }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> XdgShellSurfaceHandle {
        self.assert_toplevel();
        unsafe { XdgShellSurfaceHandle::from_ptr((*self.toplevel).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> XdgShellSurfaceHandle {
        self.assert_toplevel();
        unsafe { XdgShellSurfaceHandle::from_ptr((*self.toplevel).parent) }
    }

    pub fn added(&self) -> bool {
        self.assert_toplevel();
        unsafe { (*self.toplevel).added }
    }

    /// Get the pending client state.
    pub fn client_pending_state(&self) -> wlr_xdg_toplevel_state {
        self.assert_toplevel();
        unsafe { (*self.toplevel).client_pending }
    }

    /// Get the pending server state.
    pub fn server_pending_state(&self) -> wlr_xdg_toplevel_state {
        self.assert_toplevel();
        unsafe { (*self.toplevel).server_pending }
    }

    /// Get the current configure state.
    pub fn current_state(&self) -> wlr_xdg_toplevel_state {
        self.assert_toplevel();
        unsafe { (*self.toplevel).current }
    }

    /// Request that this toplevel surface be the given size.
    ///
    /// Returns the associated configure serial.
    pub fn set_size(&mut self, width: u32, height: u32) -> u32 {
        self.assert_toplevel();
        unsafe { wlr_xdg_toplevel_set_size(self.shell_surface, width, height) }
    }

    /// Request that this toplevel surface show itself in an activated or deactivated
    /// state.
    ///
    /// Returns the associated configure serial.
    pub fn set_activated(&mut self, activated: bool) -> u32 {
        self.assert_toplevel();
        unsafe { wlr_xdg_toplevel_set_activated(self.shell_surface, activated) }
    }

    /// Request that this toplevel surface consider itself maximized or not
    /// maximized.
    ///
    /// Returns the associated configure serial.
    pub fn set_maximized(&mut self, maximized: bool) -> u32 {
        self.assert_toplevel();
        unsafe { wlr_xdg_toplevel_set_maximized(self.shell_surface, maximized) }
    }

    /// Request that this toplevel surface consider itself fullscreen or not
    /// fullscreen.
    ///
    /// Returns the associated configure serial.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> u32 {
        self.assert_toplevel();
        unsafe { wlr_xdg_toplevel_set_fullscreen(self.shell_surface, fullscreen) }
    }

    /// Request that this toplevel surface consider itself to be resizing or not
    /// resizing.
    ///
    /// Returns the associated configure serial.
    pub fn set_resizing(&mut self, resizing: bool) -> u32 {
        self.assert_toplevel();
        unsafe { wlr_xdg_toplevel_set_resizing(self.shell_surface, resizing) }
    }

    /// Request that this toplevel surface closes.
    pub fn close(&mut self) {
        self.assert_toplevel();
        unsafe { wlr_xdg_surface_send_close(self.shell_surface) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xdg_toplevel {
        self.toplevel
    }

    /// Asserts that this is indeed a toplevel. This is done in wlroots, but is
    /// also added here to make it easier to detect bugs.
    fn assert_toplevel(&self) {
        use wlroots_sys::wlr_xdg_surface_role::WLR_XDG_SURFACE_ROLE_TOPLEVEL;
        unsafe {
            if (*self.shell_surface).role != WLR_XDG_SURFACE_ROLE_TOPLEVEL {
                wlr_log!(WLR_ERROR, "Expected {:?} got {:?}",
                         WLR_XDG_SURFACE_ROLE_TOPLEVEL,
                         (*self.shell_surface).role);
                assert!((*self.shell_surface).role == WLR_XDG_SURFACE_ROLE_TOPLEVEL);
            }
        }
    }
}

impl XdgPopup {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface,
                                    popup: *mut wlr_xdg_popup)
                                    -> XdgPopup {
        XdgPopup { shell_surface,
                   popup }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> XdgShellSurfaceHandle {
        unsafe { XdgShellSurfaceHandle::from_ptr((*self.popup).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> SurfaceHandle {
        unsafe { SurfaceHandle::from_ptr((*self.popup).parent) }
    }

    pub fn committed(&self) -> bool {
        unsafe { (*self.popup).committed }
    }

    /// Get a handle to the seat associated with this popup.
    pub fn seat_handle(&self) -> Option<SeatHandle> {
        unsafe {
            let seat = (*self.popup).seat;
            if seat.is_null() {
                None
            } else {
                Some(SeatHandle::from_ptr(seat))
            }
        }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.popup).geometry) }
    }
}

impl XdgShellState {
    /// Unsafe copy of the pointer
    unsafe fn clone(&self) -> Self {
        use XdgShellState::*;
        match *self {
            TopLevel(XdgTopLevel { shell_surface,
                                   toplevel }) => {
                TopLevel(XdgTopLevel { shell_surface,
                                       toplevel })
            }
            TopLevelUnmapped(XdgTopLevel { shell_surface,
                                        toplevel }) => {
                TopLevelUnmapped(XdgTopLevel { shell_surface,
                                            toplevel })
            }
            Popup(XdgPopup { shell_surface,
                             popup }) => {
                Popup(XdgPopup { shell_surface,
                                 popup })
            }
            PopupUnmapped(XdgPopup { shell_surface,
                                  popup }) => {
                PopupUnmapped(XdgPopup { shell_surface,
                                      popup })
            }
        }
    }
}
