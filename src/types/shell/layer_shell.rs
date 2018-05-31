//! wlr_layer_shell allows clients to arrange themselves in "layers" on the
//! desktop in accordance with the wlr-layer-shell protocol.
//!
//! When a client is added, the new_surface signal will be raised and passed
//! a reference to our wlr_layer_surface. At this time, the client will have
//! configured the surface as it desires, including information like
//! desired anchors and margins.
//!
//! The compositor should use this information to decide how to arrange the layer
//! on-screen, then determine the dimensions of the layer and call
//! wlr_layer_surface_configure.
//!
//! The client will then attach a buffer and commit
//! the surface, at which point the wlr_layer_surface map signal is raised and
//! the compositor should begin rendering the surface.

use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}, marker::PhantomData};

use libc::{c_double, c_void};
use wlroots_sys::{wlr_layer_surface_state, wlr_layer_surface,
                  wlr_layer_surface_configure, wlr_layer_surface_close,
                  wlr_layer_surface_for_each_surface, wlr_surface,
                  wlr_layer_surface_surface_at, zwlr_layer_shell_v1_layer};

use utils::c_to_rust_string;
use errors::{HandleErr, HandleResult};

use {SurfaceHandle, OutputHandle};

#[derive(Debug)]
pub struct LayerSurface {
    liveliness: Rc<Cell<bool>>,
    layer_surface: *mut wlr_layer_surface
}

#[derive(Debug, Clone)]
/// A handle to a layer surface that can be upgraded when there are no
/// other references active to it.
pub struct LayerSurfaceHandle {
    handle: Weak<Cell<bool>>,
    layer_surface: *mut wlr_layer_surface
}

/// The meta information about a layer surface.
pub struct LayerSurfaceState<'surface> {
    state: *const wlr_layer_surface_state,
    phantom: PhantomData<&'surface LayerSurface>
}

/// The configuration sent with a change in state.
pub struct LayerSurfaceConfigure<'surface> {
    configure: *const wlr_layer_surface_configure,
    phantom: PhantomData<&'surface LayerSurface>
}

impl <'surface> LayerSurfaceState<'surface> {
    unsafe fn new<'unbound>(state: *const wlr_layer_surface_state) -> Self {
        LayerSurfaceState { state, phantom: PhantomData }
    }

    pub fn anchor(&self) -> u32 {
        unsafe { (*self.state).anchor }
    }

    pub fn exclusive_zone(&self) -> i32 {
        unsafe { (*self.state).exclusive_zone }
    }

    /// Get the margin in this format: (top, right, bottom, left).
    pub fn margin(&self) -> (u32, u32, u32, u32) {
        unsafe {
            ((*self.state).margin.top,
             (*self.state).margin.right,
             (*self.state).margin.bottom,
             (*self.state).margin.left)
        }
    }

    pub fn keyboard_interactive(&self) -> bool {
        unsafe { (*self.state).keyboard_interactive }
    }

    /// Get the desired size of the surface in (width, height) format.
    pub fn desired_size(&self) -> (u32, u32) {
        unsafe { ((*self.state).desired_width, (*self.state).desired_height) }
    }

    /// Get the desired size of the surface in (width, height) format.
    pub fn actual_size(&self) -> (u32, u32) {
        unsafe { ((*self.state).actual_width, (*self.state).actual_height) }
    }
}

impl <'surface> LayerSurfaceConfigure<'surface> {
    unsafe fn new<'unbound>(configure: *const wlr_layer_surface_configure) -> Self {
        LayerSurfaceConfigure { configure, phantom: PhantomData }
    }

    pub fn serial(&self) -> u32 {
        unsafe { (*self.configure).serial }
    }

    pub fn state(&'surface self) -> LayerSurfaceState<'surface> {
        unsafe { LayerSurfaceState::new(&(*self.configure).state) }
    }
}

impl LayerSurface {
    pub(crate) unsafe fn new(layer_surface: *mut wlr_layer_surface) -> Self {
        if (*layer_surface).output.is_null() {
            // TODO Don't do this, instead gotta return a builder
            panic!("Layer surface had a null output")
        }
        let liveliness = Rc::new(Cell::new(false));
        LayerSurface { liveliness,
                       layer_surface }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_layer_surface {
        self.layer_surface
    }

    unsafe fn from_handle(handle: &LayerSurfaceHandle) -> HandleResult<Self> {
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(LayerSurface { liveliness, layer_surface: handle.as_ptr() })
    }

    /// Creates a weak reference to a `LayerSurface`.
    pub fn weak_reference(&self) -> LayerSurfaceHandle {
        LayerSurfaceHandle { handle: Rc::downgrade(&self.liveliness),
                             layer_surface: self.layer_surface }
    }

    /// Gets the surface used by this Layer shell.
    pub fn surface(&self) -> SurfaceHandle {
        unsafe {
            let surface = (*self.layer_surface).surface;
            if surface.is_null() {
                panic!("Layer surface had a null surface!")
            }
            SurfaceHandle::from_ptr(surface)
        }
    }

    pub fn output(&self) -> OutputHandle {
        unsafe {
            let output = (*self.layer_surface).output;
            if output.is_null() {
                panic!("Layer surface had a null output!")
            }
            OutputHandle::from_ptr(output)
        }
    }

    // TODO Implement when xdg shell stable is implemented
    //pub fn popups(&self) -> Vec<>

    /// Get the namespace this surface resides in.
    pub fn namespace(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.layer_surface).namespace) }
    }

    pub fn layer(&self) -> zwlr_layer_shell_v1_layer {
        unsafe { (*self.layer_surface).layer }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.layer_surface).added }
    }

    pub fn configured(&self) -> bool {
        unsafe { (*self.layer_surface).configured }
    }

    pub fn mapped(&self) -> bool {
        unsafe { (*self.layer_surface).mapped }
    }

    pub fn closed(&self) -> bool {
        unsafe { (*self.layer_surface).closed}
    }

    pub fn configure_serial(&self) -> u32 {
        unsafe { (*self.layer_surface).configure_serial }
    }

    pub fn configure_next_serial(&self) -> u32 {
        unsafe { (*self.layer_surface).configure_next_serial }
    }

    pub fn configure_list<'surface>(&'surface self) -> Vec<LayerSurfaceConfigure<'surface>> {
        let mut result = Vec::new();
        unsafe {
            wl_list_for_each!((*self.layer_surface).configure_list,
                            link,
                            (configure: wlr_layer_surface_configure) => {
                result.push(LayerSurfaceConfigure::new(configure))
            });
        }
        result
    }

    pub fn acked_configure<'surface>(&'surface self) -> Option<LayerSurfaceConfigure<'surface>> {
        unsafe {
            let acked_configure = (*self.layer_surface).acked_configure;
            if acked_configure.is_null() {
                None
            } else {
                Some(LayerSurfaceConfigure::new(acked_configure))
            }
        }
    }

    pub fn client_pending<'surface>(&'surface self) -> LayerSurfaceState<'surface> {
        unsafe {
            LayerSurfaceState::new(&(*self.layer_surface).client_pending)
        }
    }

    pub fn server_pending<'surface>(&'surface self) -> LayerSurfaceState<'surface> {
        unsafe {
            LayerSurfaceState::new(&(*self.layer_surface).server_pending)
        }
    }

    pub fn current<'surface>(&'surface self) -> LayerSurfaceState<'surface> {
        unsafe {
            LayerSurfaceState::new(&(*self.layer_surface).current)
        }
    }

    /// Unmaps this layer surface and notifies the client that it has been closed.
    pub fn close(&mut self) {
        unsafe {
            wlr_layer_surface_close(self.layer_surface)
        }
    }

    /// Find a surface within this layer-surface tree at the given surface-local
    /// coordinates.
    ///
    //// Returns the surface and coordinates in the leaf surface
    /// coordinate system or None if no surface is found at that location.
    ///
    /// Return coordinates are in (x, y) format
    pub fn surface_at(&self, sx: c_double, sy: c_double) -> Option<(SurfaceHandle, c_double, c_double)> {
        unsafe {
            let (mut sub_x, mut sub_y) = (0.0, 0.0);
            let surface_ptr = wlr_layer_surface_surface_at(self.layer_surface, sx, sy, &mut sub_x, &mut sub_y);
            if surface_ptr.is_null() {
                None
            } else {
                Some((SurfaceHandle::from_ptr(surface_ptr), sub_x, sub_y))
            }
        }
    }

    /// Calls the iterator function for each sub-surface and popup of this surface
    pub fn for_each_surface(&self, mut iterator: &mut FnMut(SurfaceHandle, i32, i32)) {
        unsafe extern "C" fn c_iterator(wlr_surface: *mut wlr_surface, sx: i32, sy: i32, data: *mut c_void) {
            let iterator = &mut *(data as *mut &mut FnMut(SurfaceHandle, i32, i32));
            let surface = SurfaceHandle::from_ptr(wlr_surface);
            iterator(surface, sx, sy);
        }
        unsafe {
            let iterator_ptr: *mut c_void = &mut iterator as *mut _ as *mut c_void;
            wlr_layer_surface_for_each_surface(self.layer_surface, Some(c_iterator), iterator_ptr);
        }
    }
}

impl Drop for LayerSurface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(L_DEBUG, "Dropped Layer Shell Surface {:p}", self.layer_surface);
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(L_DEBUG,
                        "Still {} weak pointers to Layer Shell Surface {:p}",
                        weak_count, self.layer_surface);
            }
        }
    }
}

impl LayerSurfaceHandle {
    /// Constructs a new LayerSurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            LayerSurfaceHandle { handle: Weak::new(),
                                 layer_surface: ptr::null_mut() }
        }
    }

    /// Upgrades the wayland shell handle to a reference to the backing `LayerSurface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `LayerSurface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<LayerSurface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                let shell_surface = LayerSurface::from_handle(self)?;
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(shell_surface)
            })
    }

    /// Run a function on the referenced LayerSurface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the output
    /// to a short lived scope of an anonymous function,
    /// this function ensures the LayerSurface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `LayerSurface`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&mut self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut LayerSurface) -> R
    {
        let mut layer_surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut layer_surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running LayerSurface callback, \
                                                    mutable lock was false for: {:?}",
                                                   layer_surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    unsafe fn as_ptr(&self) -> *mut wlr_layer_surface {
        self.layer_surface
    }
}

impl Default for LayerSurfaceHandle {
    fn default() -> Self {
        LayerSurfaceHandle::new()
    }
}

impl PartialEq for LayerSurfaceHandle {
    fn eq(&self, other: &LayerSurfaceHandle) -> bool {
        self.layer_surface == other.layer_surface
    }
}

impl Eq for LayerSurfaceHandle {}
