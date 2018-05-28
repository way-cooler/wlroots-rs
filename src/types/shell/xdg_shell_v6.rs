//! TODO Documentation

use std::{panic, ptr};
use std::cell::Cell;
use std::rc::{Rc, Weak};

use wlroots_sys::{wlr_xdg_popup_v6, wlr_xdg_surface_v6, wlr_xdg_surface_v6_ping,
                  wlr_xdg_surface_v6_role, wlr_xdg_surface_v6_send_close,
                  wlr_xdg_surface_v6_surface_at, wlr_xdg_toplevel_v6,
                  wlr_xdg_toplevel_v6_set_activated, wlr_xdg_toplevel_v6_set_fullscreen,
                  wlr_xdg_toplevel_v6_set_maximized, wlr_xdg_toplevel_v6_set_resizing,
                  wlr_xdg_toplevel_v6_set_size, wlr_xdg_toplevel_v6_state,
                  wlr_xdg_surface_v6_for_each_surface, wlr_surface};

use {Area, SeatHandle, SurfaceHandle};
use errors::{HandleErr, HandleResult};
use utils::c_to_rust_string;
use manager::XdgV6Shell;
use libc::c_void;

/// Used internally to reclaim a handle from just a *mut wlr_xdg_surface_v6.
pub(crate) struct XdgV6ShellSurfaceState {
    pub(crate) shell: *mut XdgV6Shell,
    handle: Weak<Cell<bool>>,
    shell_state: Option<XdgV6ShellState>
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct XdgV6TopLevel {
    shell_surface: *mut wlr_xdg_surface_v6,
    toplevel: *mut wlr_xdg_toplevel_v6
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct XdgV6Popup {
    shell_surface: *mut wlr_xdg_surface_v6,
    popup: *mut wlr_xdg_popup_v6
}

/// A tagged enum of the different roles used by the xdg shell.
///
/// Uses the tag to disambiguate the union in `wlr_xdg_surface_v6`.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum XdgV6ShellState {
    TopLevel(XdgV6TopLevel),
    Popup(XdgV6Popup)
}

#[derive(Debug)]
pub struct XdgV6ShellSurface {
    liveliness: Rc<Cell<bool>>,
    state: Option<XdgV6ShellState>,
    shell_surface: *mut wlr_xdg_surface_v6
}

#[derive(Debug)]
pub struct XdgV6ShellSurfaceHandle {
    state: Option<XdgV6ShellState>,
    handle: Weak<Cell<bool>>,
    shell_surface: *mut wlr_xdg_surface_v6
}

impl Clone for XdgV6ShellSurfaceHandle {
    fn clone(&self) -> Self {
        let state = match self.state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        XdgV6ShellSurfaceHandle { state,
                                  handle: self.handle.clone(),
                                  shell_surface: self.shell_surface }
    }
}

impl XdgV6ShellSurface {
    pub(crate) unsafe fn new<T>(shell_surface: *mut wlr_xdg_surface_v6, state: T) -> Self
        where T: Into<Option<XdgV6ShellState>>
    {
        let state = state.into();
        // TODO FIXME Free state in drop impl when Rc == 1
        (*shell_surface).data = ptr::null_mut();
        let liveliness = Rc::new(Cell::new(false));
        let shell_state =
            Box::new(XdgV6ShellSurfaceState { shell: ptr::null_mut(),
                                              handle: Rc::downgrade(&liveliness),
                                              shell_state: match state {
                                                  None => None,
                                                  Some(ref state) => Some(state.clone())
                                              } });
        (*shell_surface).data = Box::into_raw(shell_state) as *mut _;
        XdgV6ShellSurface { liveliness,
                            state: state,
                            shell_surface }
    }

    unsafe fn from_handle(handle: &XdgV6ShellSurfaceHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(XdgV6ShellSurface { liveliness,
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
    pub fn role(&self) -> wlr_xdg_surface_v6_role {
        unsafe { (*self.shell_surface).role }
    }

    pub fn state(&mut self) -> Option<&mut XdgV6ShellState> {
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
            wlr_xdg_surface_v6_ping(self.shell_surface);
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
                wlr_xdg_surface_v6_surface_at(self.shell_surface, sx, sy, sub_sx, sub_sy);
            if sub_surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(sub_surface))
            }
        }
    }

    pub fn for_each_surface(&self, mut iterator: &mut FnMut(SurfaceHandle, i32, i32)) {
        unsafe {
            unsafe extern "C" fn c_iterator(wlr_surface: *mut wlr_surface, sx: i32, sy: i32, data: *mut c_void) {
                let iterator = &mut *(data as *mut &mut FnMut(SurfaceHandle, i32, i32));
                let surface = SurfaceHandle::from_ptr(wlr_surface);
                iterator(surface, sx, sy);
            }
            let iterator_ptr: *mut c_void = &mut iterator as *mut _ as *mut c_void;
            wlr_xdg_surface_v6_for_each_surface(self.shell_surface, Some(c_iterator), iterator_ptr);
        }
    }

    /// Creates a weak reference to an `XdgV6ShellSurface`.
    pub fn weak_reference(&self) -> XdgV6ShellSurfaceHandle {
        XdgV6ShellSurfaceHandle { handle: Rc::downgrade(&self.liveliness),
                                  state: match self.state {
                                      None => None,
                                      Some(ref state) => unsafe { Some(state.clone()) }
                                  },
                                  shell_surface: self.shell_surface }
    }
}

impl XdgV6ShellSurfaceHandle {
    /// Constructs a new XdgV6ShellSurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            XdgV6ShellSurfaceHandle { handle: Weak::new(),
                                      state: None,
                                      shell_surface: ptr::null_mut() }
        }
    }

    /// Creates a XdgV6ShellSurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(shell_surface: *mut wlr_xdg_surface_v6) -> Self {
        if shell_surface.is_null() {
            panic!("shell surface was null")
        }
        let data = (*shell_surface).data as *mut XdgV6ShellSurfaceState;
        if data.is_null() {
            panic!("Cannot construct handle from a shell surface that has not been set up!");
        }
        let handle = (*data).handle.clone();
        let state = match (*data).shell_state {
            None => None,
            Some(ref state) => Some(unsafe { state.clone() })
        };
        XdgV6ShellSurfaceHandle { handle,
                                  state,
                                  shell_surface }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `XdgV6ShellSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `XdgV6ShellSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<XdgV6ShellSurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = XdgV6ShellSurface::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced XdgV6ShellSurface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the XdgV6ShellSurface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut XdgV6ShellSurface) -> R
    {
        let mut xdg_surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut xdg_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running XdgV6ShellSurface callback, \
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

    unsafe fn as_ptr(&self) -> *mut wlr_xdg_surface_v6 {
        self.shell_surface
    }
}

impl Default for XdgV6ShellSurfaceHandle {
    fn default() -> Self {
        XdgV6ShellSurfaceHandle::new()
    }
}

impl PartialEq for XdgV6ShellSurfaceHandle {
    fn eq(&self, other: &XdgV6ShellSurfaceHandle) -> bool {
        self.shell_surface == other.shell_surface
    }
}

impl Eq for XdgV6ShellSurfaceHandle {}

impl XdgV6TopLevel {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface_v6,
                                    toplevel: *mut wlr_xdg_toplevel_v6)
                                    -> XdgV6TopLevel {
        XdgV6TopLevel { shell_surface,
                        toplevel }
    }

    /// Get the title associated with this XDG shell toplevel.
    pub fn title(&self) -> String {
        unsafe { c_to_rust_string((*self.toplevel).title).expect("Could not parse class as UTF-8") }
    }

    /// Get the app id associated with this XDG shell toplevel.
    pub fn app_id(&self) -> String {
        unsafe {
            c_to_rust_string((*self.toplevel).app_id).expect("Could not parse class as UTF-8")
        }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.toplevel).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.toplevel).parent) }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.toplevel).added }
    }

    /// Get the pending client state.
    pub fn client_pending_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).client_pending }
    }

    /// Get the pending server state.
    pub fn server_pending_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).server_pending }
    }

    /// Get the current configure state.
    pub fn current_state(&self) -> wlr_xdg_toplevel_v6_state {
        unsafe { (*self.toplevel).current }
    }

    /// Request that this toplevel surface be the given size.
    ///
    /// Returns the associated configure serial.
    pub fn set_size(&mut self, width: u32, height: u32) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_size(self.shell_surface, width, height) }
    }

    /// Request that this toplevel surface show itself in an activated or deactivated
    /// state.
    ///
    /// Returns the associated configure serial.
    pub fn set_activated(&mut self, activated: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_activated(self.shell_surface, activated) }
    }

    /// Request that this toplevel surface consider itself maximized or not
    /// maximized.
    ///
    /// Returns the associated configure serial.
    pub fn set_maximized(&mut self, maximized: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_maximized(self.shell_surface, maximized) }
    }

    /// Request that this toplevel surface consider itself fullscreen or not
    /// fullscreen.
    ///
    /// Returns the associated configure serial.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_fullscreen(self.shell_surface, fullscreen) }
    }

    /// Request that this toplevel surface consider itself to be resizing or not
    /// resizing.
    ///
    /// Returns the associated configure serial.
    pub fn set_resizing(&mut self, resizing: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_v6_set_resizing(self.shell_surface, resizing) }
    }

    /// Request that this toplevel surface closes.
    pub fn close(&mut self) {
        unsafe { wlr_xdg_surface_v6_send_close(self.shell_surface) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xdg_toplevel_v6 {
        self.toplevel
    }
}

impl XdgV6Popup {
    pub(crate) unsafe fn from_shell(shell_surface: *mut wlr_xdg_surface_v6,
                                    popup: *mut wlr_xdg_popup_v6)
                                    -> XdgV6Popup {
        XdgV6Popup { shell_surface,
                     popup }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.popup).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> XdgV6ShellSurfaceHandle {
        unsafe { XdgV6ShellSurfaceHandle::from_ptr((*self.popup).parent) }
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

impl XdgV6ShellState {
    /// Unsafe copy of the pointer
    unsafe fn clone(&self) -> Self {
        use XdgV6ShellState::*;
        match *self {
            TopLevel(XdgV6TopLevel { shell_surface,
                                     toplevel }) => {
                TopLevel(XdgV6TopLevel { shell_surface,
                                         toplevel })
            }
            Popup(XdgV6Popup { shell_surface,
                               popup }) => {
                Popup(XdgV6Popup { shell_surface,
                                   popup })
            }
        }
    }
}
