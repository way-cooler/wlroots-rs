//! TODO Documentation

use libc::{self, c_double};
use std::{panic, ptr, cell::Cell, rc::{Rc, Weak}, time::Duration};

use wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{timespec, wlr_subsurface, wlr_surface, wlr_surface_get_root_surface,
                  wlr_surface_has_buffer, wlr_surface_point_accepts_input, wlr_surface_send_enter,
                  wlr_surface_send_frame_done, wlr_surface_send_leave, wlr_surface_surface_at,
                  wlr_surface_is_layer_surface, wlr_surface_is_xdg_surface};

use super::{Subsurface, SubsurfaceHandle, SubsurfaceHandler, SubsurfaceManager, SurfaceState,
            InternalSubsurface};
use compositor::{compositor_handle, CompositorHandle};
use Output;
use errors::{HandleErr, HandleResult};
use render::Texture;
use utils::c_to_rust_string;

pub trait SurfaceHandler {
    fn on_commit(&mut self, CompositorHandle, SurfaceHandle) {}

    fn new_subsurface(&mut self, CompositorHandle, SurfaceHandle, SubsurfaceHandle) -> Option<Box<SubsurfaceHandler>> {
        None
    }

    fn on_destroy(&mut self, CompositorHandle, SurfaceHandle) {}
}

impl SurfaceHandler for () {}

wayland_listener!(InternalSurface, (Surface, Box<SurfaceHandler>), [
    on_commit_listener => on_commit_notify: |this: &mut InternalSurface, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut surface, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_commit(compositor, surface.weak_reference());
    };
    new_subsurface_listener => new_listener_notify: |this: &mut InternalSurface,
                                                     data: *mut libc::c_void,|
    unsafe {
        let (ref mut surface, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        let subsurface_ptr = data as *mut wlr_subsurface;
        let subsurface = Subsurface::new(subsurface_ptr);
        if let Some(subsurface_handler) = manager.new_subsurface(compositor,
                                                                surface.weak_reference(),
                                                                subsurface.weak_reference()) {
            let mut internal_subsurface = InternalSubsurface::new((subsurface, subsurface_handler));
            wl_signal_add(&mut (*subsurface_ptr).events.destroy as *mut _ as _,
                          internal_subsurface.on_destroy_listener() as _);
            (*subsurface_ptr).data = Box::into_raw(internal_subsurface) as *mut _;
        }
    };
    on_destroy_listener => on_destroy_notify: |this: &mut InternalSurface, data: *mut libc::c_void,|
    unsafe {
        let (ref mut surface, ref mut manager) = this.data;
        let compositor = match compositor_handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_destroy(compositor, surface.weak_reference());
        let surface_ptr = data as *mut wlr_surface;
        let surface_state_ptr = (*surface_ptr).data as *mut InternalSurfaceState;
        // NOTE that wlroots cleans up the wlr_surface properly (so the Surface drop is called).
        // This just insures we clean up our listeners.
        Box::<InternalSurface>::from_raw((*surface_state_ptr).surface);
    };
]);

impl InternalSurface {
    pub(crate) unsafe fn data(&mut self) -> &mut (Surface, Box<SurfaceHandler>) {
        &mut self.data
    }
}

/// The state stored in the wlr_surface user data.
pub(crate) struct InternalSurfaceState {
    /// Pointer to the backing storage of the surface.
    pub(crate) surface: *mut InternalSurface,
    /// Used to reconstruct a SurfaceHandle from just an *mut wlr_surface.
    handle: Weak<Cell<bool>>,
    /// Weak reference to the manager for the list of subsurfaces.
    /// This is here so that we can reconstruct the Surface from a SurfaceHandle.
    subsurfaces_manager: Weak<Box<SubsurfaceManager>>
}

/// A Wayland object that represents the data that we display on the screen.
///
/// Most surfaces come from Wayland clients, though they can also be created
/// by the compositor directly.
#[derive(Debug)]
pub struct Surface {
    /// The structe that ensures weak handles to this structure are still alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `SurfaceHandle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `SurfaceHandle`.
    liveliness: Rc<Cell<bool>>,
    /// The manager of the list of subsurfaces for this surface.
    ///
    /// When the subsurface destruction event fires the manager will deal with
    /// updating the list.
    ///
    /// When you have a reference to the Surface you can access its children
    /// through the getter for this list.
    subsurfaces_manager: Rc<Box<SubsurfaceManager>>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    surface: *mut wlr_surface
}

/// See `Surface` for more information on how to use this structure.
#[derive(Clone, Debug)]
pub struct SurfaceHandle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<Cell<bool>>,
    /// Weak reference to the manager of the list of subsurfaces for this surface.
    ///
    /// Used when reconstructing a `Surface` so that we can access
    /// the list of subsurfaces.
    subsurfaces_manager: Weak<Box<SubsurfaceManager>>,
    /// The pointer to the wlroots object that wraps a wl_surface.
    surface: *mut wlr_surface
}

impl Surface {
    pub(crate) unsafe fn new(surface: *mut wlr_surface) -> Self {
        if !(*surface).data.is_null() {
            panic!("Tried to construct a Surface from an already initialized wlr_surface");
        }
        let liveliness = Rc::new(Cell::new(false));
        let handle = Rc::downgrade(&liveliness);
        let subsurfaces_manager = Rc::new(Surface::create_manager(surface));
        let weak_manager = Rc::downgrade(&subsurfaces_manager);
        (*surface).data = Box::into_raw(Box::new(InternalSurfaceState { surface: ptr::null_mut(),
                                                                        handle,
                                                                        subsurfaces_manager:
                                                                        weak_manager }))
            as _;
        Surface { liveliness,
                  subsurfaces_manager,
                  surface }
    }

    /// Create the subsurface manager and ensures theat the listeners are
    /// set up correctly to listen for subsurface creation and deletion.
    fn create_manager(surface: *mut wlr_surface) -> Box<SubsurfaceManager> {
        unsafe {
            let mut subsurfaces = vec![];
            wl_list_for_each!((*surface).subsurfaces, parent_link,
                              (subsurface: wlr_subsurface) => {
                                  subsurfaces.push(Subsurface::new(subsurface))
                              });
            let mut manager = SubsurfaceManager::new(subsurfaces);
            for sub_surface in &mut manager.subsurfaces() {
                wl_signal_add(&mut (*sub_surface.as_ptr()).events.destroy as *mut _ as _,
                              manager.subsurface_destroyed_listener() as _);
            }
            wl_signal_add(&mut (*surface).events.new_subsurface as *mut _ as _,
                          manager.subsurface_created_listener() as _);
            manager
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_surface {
        self.surface
    }

    /// Get the surface state.
    pub fn current_state<'surface>(&'surface mut self) -> SurfaceState<'surface> {
        unsafe {
            let state = (*self.surface).current;
            SurfaceState::new(state)
        }
    }

    /// Get the pending surface state.
    pub fn pending_state<'surface>(&'surface mut self) -> SurfaceState<'surface> {
        unsafe {
            let state = (*self.surface).current;
            SurfaceState::new(state)
        }
    }

    /// Gets a list of handles to the `Subsurface`s of this `Surface`.
    pub fn subsurfaces(&self) -> Vec<SubsurfaceHandle> {
        self.subsurfaces_manager.subsurfaces()
    }

    /// Get the texture of this surface.
    pub fn texture(&self) -> Texture {
        unsafe { Texture::from_ptr((*self.surface).texture) }
    }

    /// Get the lifetime bound role (if one exists) for this surface.
    pub fn role(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.surface).role) }
    }

    /// Whether or not this surface currently has an attached buffer.
    ///
    /// A surface has an attached buffer when it commits with a non-null buffer in its pending
    /// state.
    ///
    /// A surface will not have a buffer if it has never committed one, has
    /// committed a null buffer, or something went wrong with uploading the buffer.
    pub fn has_buffer(&self) -> bool {
        unsafe { wlr_surface_has_buffer(self.surface) }
    }

    /// Determines if this surface accepts input or not at the provided surface
    /// local coordinates.
    pub fn accepts_input(&self, sx: c_double, sy: c_double) -> bool {
        unsafe { wlr_surface_point_accepts_input(self.surface, sx, sy) }
    }

    /// Determines if this surface is an XDG surface.
    ///
    /// This is really only useful for getting the parent of popups from stable XDG
    /// shell surfaces.
    pub fn is_xdg_surface(&self) -> bool {
        unsafe { wlr_surface_is_xdg_surface(self.surface) }
    }

    /// Find a subsurface within this surface at the surface-local coordinates.
    ///
    /// Returns the surface and coordinates in the topmost surface coordinate system
    /// or None if no subsurface is found at that location.
    pub fn subsurface_at(&mut self,
                         sx: f64,
                         sy: f64,
                         sub_x: &mut f64,
                         sub_y: &mut f64)
                         -> Option<SurfaceHandle> {
        unsafe {
            let surface = wlr_surface_surface_at(self.surface, sx, sy, sub_x, sub_y);
            if surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(surface))
            }
        }
    }

    /// Get the top of the subsurface tree for this surface.
    pub fn get_root_surface(&self) -> Option<SurfaceHandle> {
        unsafe {
            let surface = wlr_surface_get_root_surface(self.surface);
            if surface.is_null() {
                None
            } else {
                Some(SurfaceHandle::from_ptr(surface))
            }
        }
    }

    pub fn send_enter(&mut self, output: &mut Output) {
        unsafe { wlr_surface_send_enter(self.surface, output.as_ptr()) }
    }

    pub fn send_leave(&mut self, output: &mut Output) {
        unsafe { wlr_surface_send_leave(self.surface, output.as_ptr()) }
    }

    /// Send the frame done event.
    pub fn send_frame_done(&mut self, duration: Duration) {
        unsafe {
            // FIXME
            // This is converting from a u64 -> i64
            // Something bad could happen!
            let when = timespec { tv_sec: duration.as_secs() as i64,
                                  tv_nsec: duration.subsec_nanos() as i64 };
            wlr_surface_send_frame_done(self.surface, &when);
        }
    }

    /// Get the matrix used to convert the internal byte buffer to use in the
    /// surface.
    pub fn buffer_to_surface_matrix(&self) -> [f32; 9] {
        unsafe { (*self.surface).buffer_to_surface_matrix }
    }

    /// Get the matrix used to convert the surface back to the internal byte
    /// buffer.
    pub fn surface_to_buffer_matrix(&self) -> [f32; 9] {
        unsafe { (*self.surface).surface_to_buffer_matrix }
    }

    /// Determines if this surface is a layer surface.
    pub fn is_layer_surface(&self) -> bool {
        unsafe {
            wlr_surface_is_layer_surface(self.surface)
        }
    }

    /// Creates a weak reference to a `Surface`.
    ///
    /// # Panics
    /// If this `Surface` is a previously upgraded `SurfaceHandle`
    /// then this function will panic.
    pub fn weak_reference(&self) -> SurfaceHandle {
        SurfaceHandle { handle: Rc::downgrade(&self.liveliness),
                        surface: self.surface,
                        subsurfaces_manager: Rc::downgrade(&self.subsurfaces_manager) }
    }

    unsafe fn from_handle(handle: &SurfaceHandle) -> HandleResult<Self> {
        let data = (*handle.surface).data as *mut InternalSurfaceState;
        let subsurfaces_manager = (*data).subsurfaces_manager
                                         .clone()
                                         .upgrade()
                                         .expect("Could not upgrade subsurfaces list");
        let liveliness = handle.handle
                               .upgrade()
                               .ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Surface { liveliness,
                     subsurfaces_manager,
                     surface: handle.surface })
    }
}

impl SurfaceHandle {
    /// Constructs a new SurfaceHandle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            SurfaceHandle { handle: Weak::new(),
                            subsurfaces_manager: Weak::new(),
                            surface: ptr::null_mut() }
        }
    }
    /// Creates an SurfaceHandle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(surface: *mut wlr_surface) -> Self {
        let data = (*surface).data as *mut InternalSurfaceState;
        if data.is_null() {
            panic!("Surface has not been set up");
        }
        let handle = (*data).handle.clone();
        let subsurfaces_manager = (*data).subsurfaces_manager.clone();
        SurfaceHandle { handle,
                        surface,
                        subsurfaces_manager }
    }

    /// Upgrades the surface handle to a reference to the backing `Surface`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Surface`
    /// which may live forever..
    /// But no surface lives forever and might be disconnected at any time.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Surface> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Surface::from_handle(self)
            })
    }

    /// Run a function on the referenced Surface, if it still exists
    ///
    /// Returns the result of the function, if successful
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the surface
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Surface does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Surface`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut Surface) -> R
    {
        let mut surface = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut surface)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running surface callback, mutable lock \
                                                    was false for: {:?}",
                                                   surface);
                                          panic!("Lock in incorrect state!");
                                      }
                                      check.set(false);
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }
}

impl Default for SurfaceHandle {
    fn default() -> Self {
        SurfaceHandle::new()
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) != 1 {
            return
        }
        wlr_log!(L_DEBUG, "Dropped surface {:p}", self.surface);
        let weak_count = Rc::weak_count(&self.liveliness);
        if weak_count > 0 {
            wlr_log!(L_DEBUG,
                     "Still {} weak pointers to Surface {:p}",
                     weak_count,
                     self.surface);
        }
        unsafe {
            Box::from_raw((*self.surface).data as *mut InternalSurfaceState);
        }
    }
}

impl Drop for InternalSurface {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.on_commit_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.new_subsurface_listener()).link as *mut _ as _);
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_list_remove,
                          &mut (*self.on_destroy_listener()).link as *mut _ as _);
        }
    }
}
