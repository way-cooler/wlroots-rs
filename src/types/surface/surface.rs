//! TODO Documentation

use crate::libc::{self, c_double, clock_t};
use std::{
    cell::Cell,
    panic,
    ptr::NonNull,
    rc::{Rc, Weak},
    time::Duration
};

use crate::wayland_sys::server::signal::wl_signal_add;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{
    timespec, wlr_subsurface, wlr_surface, wlr_surface_get_root_surface, wlr_surface_get_texture,
    wlr_surface_has_buffer, wlr_surface_is_xdg_surface, wlr_surface_point_accepts_input,
    wlr_surface_send_enter, wlr_surface_send_frame_done, wlr_surface_send_leave, wlr_surface_surface_at
};

use crate::{
    compositor,
    output::Output,
    render::Texture,
    surface::{
        self,
        subsurface::{self, InternalSubsurface, Subsurface},
        subsurface_manager::SubsurfaceManager
    },
    utils::{self, c_to_rust_string, HandleErr, HandleResult, Handleable}
};

pub type Handle = utils::Handle<Weak<Box<SubsurfaceManager>>, wlr_surface, Surface>;

#[allow(unused_variables)]
pub trait Handler {
    fn on_commit(&mut self, compositor_handle: compositor::Handle, suface_handle: Handle) {}

    fn new_subsurface(
        &mut self,
        compositor_hadle: compositor::Handle,
        surface_handle: Handle,
        subsurface_handle: subsurface::Handle
    ) -> Option<Box<dyn subsurface::Handler>> {
        None
    }

    fn on_destroy(&mut self, _: compositor::Handle, _: Handle) {}
}

impl Handler for () {}

wayland_listener!(pub(crate) InternalSurface, (Surface, Box<dyn Handler>), [
    on_commit_listener => on_commit_notify: |this: &mut InternalSurface, _data: *mut libc::c_void,|
    unsafe {
        let (ref mut surface, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_commit(compositor, surface.weak_reference());
    };
    new_subsurface_listener => new_listener_notify: |this: &mut InternalSurface,
                                                     data: *mut libc::c_void,|
    unsafe {
        let (ref mut surface, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
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
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        manager.on_destroy(compositor, surface.weak_reference());
        let surface_ptr = data as *mut wlr_surface;
        let surface_state_ptr = (*surface_ptr).data as *mut InternalState;
        // NOTE that wlroots cleans up the wlr_surface properly (so the Surface drop is called).
        // This just insures we clean up our listeners.
        if let Some(surface_ptr) = (*surface_state_ptr).surface {
            Box::<InternalSurface>::from_raw(surface_ptr.as_ptr());
        }
    };
]);

impl InternalSurface {
    pub(crate) unsafe fn data(&mut self) -> &mut (Surface, Box<dyn Handler>) {
        &mut self.data
    }
}

/// The state stored in the wlr_surface user data.
pub(crate) struct InternalState {
    /// Pointer to the backing storage of the surface.
    pub(crate) surface: Option<NonNull<InternalSurface>>,
    /// Used to reconstruct a surface::Handle from just an *mut wlr_surface.
    handle: Weak<Cell<bool>>,
    /// Weak reference to the manager for the list of subsurfaces.
    /// This is here so that we can reconstruct the Surface from a
    /// surface::Handle.
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
    /// If this is `None`, then this is from an upgraded `surface::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `surface::Handle`.
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
    surface: NonNull<wlr_surface>
}

impl Surface {
    pub(crate) unsafe fn new(surface: *mut wlr_surface) -> Self {
        let surface = NonNull::new(surface).expect("Surface pointer was null");
        if !(*surface.as_ptr()).data.is_null() {
            panic!("Tried to construct a Surface from an already initialized wlr_surface");
        }
        let liveliness = Rc::new(Cell::new(false));
        let handle = Rc::downgrade(&liveliness);
        let subsurfaces_manager = Rc::new(Surface::create_manager(surface.as_ptr()));
        let weak_manager = Rc::downgrade(&subsurfaces_manager);
        let state = InternalState {
            surface: None,
            handle,
            subsurfaces_manager: weak_manager
        };
        (*surface.as_ptr()).data = Box::into_raw(Box::new(state)) as _;
        Surface {
            liveliness,
            subsurfaces_manager,
            surface
        }
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
            wl_signal_add(
                &mut (*surface).events.new_subsurface as *mut _ as _,
                manager.subsurface_created_listener() as _
            );
            manager
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_surface {
        self.surface.as_ptr()
    }

    /// Get the surface state.
    pub fn current_state(&mut self) -> surface::State {
        unsafe {
            let state = (*self.surface.as_ptr()).current;
            surface::State::new(state)
        }
    }

    /// Get the pending surface state.
    pub fn pending_state(&mut self) -> surface::State {
        unsafe {
            let state = (*self.surface.as_ptr()).current;
            surface::State::new(state)
        }
    }

    /// Gets a list of handles to the `Subsurface`s of this `Surface`.
    pub fn subsurfaces(&self) -> Vec<subsurface::Handle> {
        self.subsurfaces_manager.subsurfaces()
    }

    /// Get the texture of this surface.
    ///
    /// Returns None if no buffer is currently attached or if something went
    /// wrong with uploading the buffer.
    pub fn texture(&self) -> Option<Texture> {
        unsafe {
            let texture_ptr = wlr_surface_get_texture(self.surface.as_ptr());
            if texture_ptr.is_null() {
                None
            } else {
                Some(Texture::from_ptr(texture_ptr))
            }
        }
    }

    /// Get the lifetime bound role (if one exists) for this surface.
    pub fn role(&self) -> Option<String> {
        unsafe { c_to_rust_string((*(*self.surface.as_ptr()).role).name) }
    }

    /// Whether or not this surface currently has an attached buffer.
    ///
    /// A surface has an attached buffer when it commits with a non-null buffer
    /// in its pending state.
    ///
    /// A surface will not have a buffer if it has never committed one, has
    /// committed a null buffer, or something went wrong with uploading the
    /// buffer.
    pub fn has_buffer(&self) -> bool {
        unsafe { wlr_surface_has_buffer(self.surface.as_ptr()) }
    }

    /// Determines if this surface accepts input or not at the provided surface
    /// local coordinates.
    pub fn accepts_input(&self, sx: c_double, sy: c_double) -> bool {
        unsafe { wlr_surface_point_accepts_input(self.surface.as_ptr(), sx, sy) }
    }

    /// Determines if this surface is an XDG surface.
    ///
    /// This is really only useful for getting the parent of popups from stable
    /// XDG shell surfaces.
    pub fn is_xdg_surface(&self) -> bool {
        unsafe { wlr_surface_is_xdg_surface(self.surface.as_ptr()) }
    }

    /// Find a subsurface within this surface at the surface-local coordinates.
    ///
    /// Returns the surface and coordinates in the topmost surface coordinate
    /// system or None if no subsurface is found at that location.
    pub fn subsurface_at(&mut self, sx: f64, sy: f64, sub_x: &mut f64, sub_y: &mut f64) -> Option<Handle> {
        unsafe {
            let surface = wlr_surface_surface_at(self.surface.as_ptr(), sx, sy, sub_x, sub_y);
            if surface.is_null() {
                None
            } else {
                Some(Handle::from_ptr(surface))
            }
        }
    }

    /// Get the top of the subsurface tree for this surface.
    pub fn get_root_surface(&self) -> Option<Handle> {
        unsafe {
            let surface = wlr_surface_get_root_surface(self.surface.as_ptr());
            if surface.is_null() {
                None
            } else {
                Some(Handle::from_ptr(surface))
            }
        }
    }

    pub fn send_enter(&mut self, output: &mut Output) {
        unsafe { wlr_surface_send_enter(self.surface.as_ptr(), output.as_ptr()) }
    }

    pub fn send_leave(&mut self, output: &mut Output) {
        unsafe { wlr_surface_send_leave(self.surface.as_ptr(), output.as_ptr()) }
    }

    /// Send the frame done event.
    #[allow(clippy::cast_lossless)]
    pub fn send_frame_done(&mut self, duration: Duration) {
        unsafe {
            // FIXME
            // This is converting from a u64 -> i64
            // Something bad could happen!
            let when = timespec {
                tv_sec: duration.as_secs() as clock_t,
                tv_nsec: duration.subsec_nanos() as clock_t
            };
            wlr_surface_send_frame_done(self.surface.as_ptr(), &when);
        }
    }
}

impl Handleable<Weak<Box<SubsurfaceManager>>, wlr_surface> for Surface {
    #[doc(hidden)]
    unsafe fn from_ptr(surface: *mut wlr_surface) -> Option<Self> {
        let surface = NonNull::new(surface)?;
        let data = (*surface.as_ptr()).data as *mut InternalState;
        let liveliness = (*data).handle.upgrade()?;
        let subsurfaces_manager = (*data).subsurfaces_manager.clone().upgrade().unwrap();
        Some(Surface {
            surface,
            liveliness,
            subsurfaces_manager
        })
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_surface {
        self.surface.as_ptr()
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let data = (*handle.ptr.as_ptr()).data as *mut InternalState;
        let subsurfaces_manager = (*data)
            .subsurfaces_manager
            .clone()
            .upgrade()
            .expect("Could not upgrade subsurfaces list");
        let liveliness = handle.handle.upgrade().ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Surface {
            liveliness,
            subsurfaces_manager,
            surface: handle.ptr
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle {
            handle: Rc::downgrade(&self.liveliness),
            ptr: self.surface,
            data: Some(Rc::downgrade(&self.subsurfaces_manager)),
            _marker: std::marker::PhantomData
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) != 1 {
            return;
        }
        wlr_log!(WLR_DEBUG, "Dropped surface {:p}", self.surface.as_ptr());
        let weak_count = Rc::weak_count(&self.liveliness);
        if weak_count > 0 {
            wlr_log!(
                WLR_DEBUG,
                "Still {} weak pointers to Surface {:p}",
                weak_count,
                self.surface.as_ptr()
            );
        }
        unsafe {
            Box::from_raw((*self.surface.as_ptr()).data as *mut InternalState);
        }
    }
}

impl Drop for InternalSurface {
    fn drop(&mut self) {
        unsafe {
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.on_commit_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.new_subsurface_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.on_destroy_listener()).link as *mut _ as _
            );
        }
    }
}
