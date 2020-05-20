//! TODO Documentation

use std::{
    cell::Cell,
    panic,
    ptr::NonNull,
    rc::{Rc, Weak}
};

use crate::libc::c_void;
use wlroots_sys::{
    wlr_surface, wlr_xdg_popup, wlr_xdg_popup_destroy, wlr_xdg_surface, wlr_xdg_surface_for_each_surface,
    wlr_xdg_surface_ping, wlr_xdg_surface_role, wlr_xdg_surface_surface_at, wlr_xdg_toplevel,
    wlr_xdg_toplevel_send_close, wlr_xdg_toplevel_set_activated, wlr_xdg_toplevel_set_fullscreen,
    wlr_xdg_toplevel_set_maximized, wlr_xdg_toplevel_set_resizing, wlr_xdg_toplevel_set_size,
    wlr_xdg_toplevel_state
};

pub use crate::events::xdg_shell_events as event;
pub use crate::manager::xdg_shell_handler::*;
pub(crate) use crate::manager::xdg_shell_manager::Manager;
pub use crate::manager::xdg_shell_manager::NewSurfaceResult;
use crate::{
    area::Area,
    seat, surface,
    utils::{self, c_to_rust_string, HandleErr, HandleResult, Handleable}
};

pub mod manager {
    //! XDG shell resources are managed by the XDG shell resource manager.
    //!
    //! To manage XDG shells from clients implement a function with
    //! [`NewSurface`](./type.NewSurface.html) as the signature.
    //!
    //! Pass that function to the [`xdg_shell::Builder`](./struct.Builder.html)
    //! which is then passed to the `compositor::Builder`.
    pub use crate::manager::xdg_shell_manager::*;
}

pub type Handle = utils::Handle<OptionalShellState, wlr_xdg_surface, Surface>;

/// A hack to ensure we can clone a shell handle.
#[derive(Debug, Eq, PartialEq, Hash)]
#[doc(hidden)]
pub struct OptionalShellState(Option<ShellState>);

/// Used internally to reclaim a handle from just a *mut wlr_xdg_surface.
pub(crate) struct SurfaceState {
    /// Pointer to the backing storage.
    pub(crate) shell: Option<NonNull<XdgShell>>,
    handle: Weak<Cell<bool>>,
    shell_state: Option<ShellState>
}

impl Clone for OptionalShellState {
    fn clone(&self) -> Self {
        OptionalShellState(match self.0 {
            None => None,
            // NOTE Rationale for safety:
            // This is only stored in the handle, and it's fine to clone
            // the raw pointer when we just have a handle.
            Some(ref state) => Some(unsafe { state.clone() })
        })
    }
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct TopLevel {
    shell_surface: NonNull<wlr_xdg_surface>,
    toplevel: NonNull<wlr_xdg_toplevel>
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Popup {
    shell_surface: NonNull<wlr_xdg_surface>,
    popup: NonNull<wlr_xdg_popup>
}

/// A tagged enum of the different roles used by the xdg shell.
///
/// Uses the tag to disambiguate the union in `wlr_xdg_surface`.
#[derive(Debug, Eq, PartialEq, Hash)]
pub enum ShellState {
    TopLevel(TopLevel),
    Popup(Popup)
}

#[derive(Debug)]
pub struct Surface {
    liveliness: Rc<Cell<bool>>,
    state: Option<ShellState>,
    shell_surface: NonNull<wlr_xdg_surface>
}

impl Surface {
    pub(crate) unsafe fn new<T>(shell_surface: NonNull<wlr_xdg_surface>, state: T) -> Self
    where
        T: Into<Option<ShellState>>
    {
        if !(*shell_surface.as_ptr()).data.is_null() {
            panic!("XDG shell has already been initialized");
        }
        let state = state.into();
        let liveliness = Rc::new(Cell::new(false));
        let shell_state = Box::new(SurfaceState {
            shell: None,
            handle: Rc::downgrade(&liveliness),
            shell_state: match state {
                None => None,
                Some(ref state) => Some(state.clone())
            }
        });
        (*shell_surface.as_ptr()).data = Box::into_raw(shell_state) as *mut _;
        Surface {
            liveliness,
            state,
            shell_surface
        }
    }

    /// Gets the surface used by this XDG shell.
    pub fn surface(&mut self) -> surface::Handle {
        unsafe {
            let surface = (*self.shell_surface.as_ptr()).surface;
            if surface.is_null() {
                panic!("xdg shell had a null surface!")
            }
            surface::Handle::from_ptr(surface)
        }
    }

    /// Get the role of this XDG surface.
    pub fn role(&self) -> wlr_xdg_surface_role {
        unsafe { (*self.shell_surface.as_ptr()).role }
    }

    pub fn state(&mut self) -> Option<&mut ShellState> {
        self.state.as_mut()
    }

    /// Determines if this XDG shell surface has been configured or not.
    pub fn configured(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).configured }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).added }
    }

    pub fn configure_serial(&self) -> u32 {
        unsafe { (*self.shell_surface.as_ptr()).configure_serial }
    }

    pub fn configure_next_serial(&self) -> u32 {
        unsafe { (*self.shell_surface.as_ptr()).configure_next_serial }
    }

    pub fn has_next_geometry(&self) -> bool {
        unsafe { (*self.shell_surface.as_ptr()).has_next_geometry }
    }

    pub fn next_geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface.as_ptr()).next_geometry) }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.shell_surface.as_ptr()).geometry) }
    }

    /// Send a ping to the surface.
    ///
    /// If the surface does not respond with a pong within a reasonable amount
    /// of time, the ping timeout event will be emitted.
    pub fn ping(&mut self) {
        unsafe {
            wlr_xdg_surface_ping(self.shell_surface.as_ptr());
        }
    }

    /// Find a surface within this surface at the surface-local coordinates.
    ///
    /// Returns the popup and coordinates in the topmost surface coordinate
    /// system or None if no popup is found at that location.
    pub fn surface_at(
        &mut self,
        sx: f64,
        sy: f64,
        sub_sx: &mut f64,
        sub_sy: &mut f64
    ) -> Option<surface::Handle> {
        unsafe {
            let sub_surface = wlr_xdg_surface_surface_at(self.shell_surface.as_ptr(), sx, sy, sub_sx, sub_sy);
            if sub_surface.is_null() {
                None
            } else {
                Some(surface::Handle::from_ptr(sub_surface))
            }
        }
    }

    pub fn for_each_surface<F>(&self, mut iterator: F)
    where
        F: FnMut(surface::Handle, i32, i32)
    {
        let mut iterator_ref: &mut dyn FnMut(surface::Handle, i32, i32) = &mut iterator;
        unsafe {
            unsafe extern "C" fn c_iterator(
                wlr_surface: *mut wlr_surface,
                sx: i32,
                sy: i32,
                data: *mut c_void
            ) {
                let iterator_fn = &mut *(data as *mut &mut dyn FnMut(surface::Handle, i32, i32));
                let surface = surface::Handle::from_ptr(wlr_surface);
                iterator_fn(surface, sx, sy);
            }
            let iterator_ptr: *mut c_void = &mut iterator_ref as *mut _ as *mut c_void;
            wlr_xdg_surface_for_each_surface(self.shell_surface.as_ptr(), Some(c_iterator), iterator_ptr);
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped xdg shell {:p}", self.shell_surface.as_ptr());
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(
                    WLR_DEBUG,
                    "Still {} weak pointers to xdg shell {:p}",
                    weak_count,
                    self.shell_surface.as_ptr()
                );
            }
        } else {
            return;
        }
        unsafe {
            let _ = Box::from_raw((*self.shell_surface.as_ptr()).data as *mut SurfaceState);
        }
    }
}

impl Handleable<OptionalShellState, wlr_xdg_surface> for Surface {
    #[doc(hidden)]
    unsafe fn from_ptr(shell_surface: *mut wlr_xdg_surface) -> Option<Self> {
        let shell_surface = NonNull::new(shell_surface)?;
        let data = &mut *((*shell_surface.as_ptr()).data as *mut SurfaceState);
        let state = match data.shell_state {
            None => None,
            Some(ref state) => Some(state.clone())
        };
        let liveliness = data.handle.upgrade().unwrap();
        Some(Surface {
            liveliness,
            state,
            shell_surface
        })
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_xdg_surface {
        self.shell_surface.as_ptr()
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle.upgrade().ok_or_else(|| HandleErr::AlreadyDropped)?;
        Ok(Surface {
            liveliness,
            shell_surface: handle.ptr,
            state: handle.data.clone().and_then(|d| d.0)
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle {
            ptr: self.shell_surface,
            handle: Rc::downgrade(&self.liveliness),
            data: Some(OptionalShellState(match self.state {
                None => None,
                Some(ref state) => Some(unsafe { state.clone() })
            })),
            _marker: std::marker::PhantomData
        }
    }
}

impl TopLevel {
    pub(crate) unsafe fn from_shell(
        shell_surface: NonNull<wlr_xdg_surface>,
        toplevel: NonNull<wlr_xdg_toplevel>
    ) -> TopLevel {
        TopLevel {
            shell_surface,
            toplevel
        }
    }

    /// Get the title associated with this XDG shell toplevel.
    pub fn title(&self) -> String {
        unsafe {
            c_to_rust_string((*self.toplevel.as_ptr()).title).expect(
                "Could not parse class as \
                 UTF-8"
            )
        }
    }

    /// Get the app id associated with this XDG shell toplevel.
    pub fn app_id(&self) -> String {
        unsafe {
            c_to_rust_string((*self.toplevel.as_ptr()).app_id).expect(
                "Could not parse class as \
                 UTF-8"
            )
        }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> Handle {
        unsafe { Handle::from_ptr((*self.toplevel.as_ptr()).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> Handle {
        unsafe { Handle::from_ptr((*self.toplevel.as_ptr()).parent) }
    }

    pub fn added(&self) -> bool {
        unsafe { (*self.toplevel.as_ptr()).added }
    }

    /// Get the pending client state.
    pub fn client_pending_state(&self) -> wlr_xdg_toplevel_state {
        unsafe { (*self.toplevel.as_ptr()).client_pending }
    }

    /// Get the pending server state.
    pub fn server_pending_state(&self) -> wlr_xdg_toplevel_state {
        unsafe { (*self.toplevel.as_ptr()).server_pending }
    }

    /// Get the current configure state.
    pub fn current_state(&self) -> wlr_xdg_toplevel_state {
        unsafe { (*self.toplevel.as_ptr()).current }
    }

    /// Request that this toplevel surface be the given size.
    ///
    /// Returns the associated configure serial.
    pub fn set_size(&mut self, width: u32, height: u32) -> u32 {
        unsafe { wlr_xdg_toplevel_set_size(self.shell_surface.as_ptr(), width, height) }
    }

    /// Request that this toplevel surface show itself in an activated or
    /// deactivated state.
    ///
    /// Returns the associated configure serial.
    pub fn set_activated(&mut self, activated: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_activated(self.shell_surface.as_ptr(), activated) }
    }

    /// Request that this toplevel surface consider itself maximized or not
    /// maximized.
    ///
    /// Returns the associated configure serial.
    pub fn set_maximized(&mut self, maximized: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_maximized(self.shell_surface.as_ptr(), maximized) }
    }

    /// Request that this toplevel surface consider itself fullscreen or not
    /// fullscreen.
    ///
    /// Returns the associated configure serial.
    pub fn set_fullscreen(&mut self, fullscreen: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_fullscreen(self.shell_surface.as_ptr(), fullscreen) }
    }

    /// Request that this toplevel surface consider itself to be resizing or not
    /// resizing.
    ///
    /// Returns the associated configure serial.
    pub fn set_resizing(&mut self, resizing: bool) -> u32 {
        unsafe { wlr_xdg_toplevel_set_resizing(self.shell_surface.as_ptr(), resizing) }
    }

    /// Request that this toplevel surface closes.
    pub fn close(&mut self) {
        unsafe { wlr_xdg_toplevel_send_close(self.shell_surface.as_ptr()) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_xdg_toplevel {
        self.toplevel.as_ptr()
    }
}

impl Popup {
    pub(crate) unsafe fn from_shell(
        shell_surface: NonNull<wlr_xdg_surface>,
        popup: NonNull<wlr_xdg_popup>
    ) -> Popup {
        Popup { shell_surface, popup }
    }

    /// Request that this popup closes.
    pub fn close(&mut self) {
        unsafe { wlr_xdg_popup_destroy(self.shell_surface.as_ptr()) }
    }

    /// Get a handle to the base surface of the xdg tree.
    pub fn base(&self) -> Handle {
        unsafe { Handle::from_ptr((*self.popup.as_ptr()).base) }
    }

    /// Get a handle to the parent surface of the xdg tree.
    pub fn parent(&self) -> surface::Handle {
        unsafe { surface::Handle::from_ptr((*self.popup.as_ptr()).parent) }
    }

    pub fn committed(&self) -> bool {
        unsafe { (*self.popup.as_ptr()).committed }
    }

    /// Get a handle to the seat associated with this popup.
    pub fn seat_handle(&self) -> Option<seat::Handle> {
        unsafe {
            let seat = (*self.popup.as_ptr()).seat;
            if seat.is_null() {
                None
            } else {
                Some(seat::Handle::from_ptr(seat))
            }
        }
    }

    pub fn geometry(&self) -> Area {
        unsafe { Area::from_box((*self.popup.as_ptr()).geometry) }
    }
}

impl ShellState {
    /// Unsafe copy of the pointer
    unsafe fn clone(&self) -> Self {
        match *self {
            ShellState::TopLevel(TopLevel {
                shell_surface,
                toplevel
            }) => ShellState::TopLevel(TopLevel {
                shell_surface,
                toplevel
            }),
            ShellState::Popup(Popup { shell_surface, popup }) => {
                ShellState::Popup(Popup { shell_surface, popup })
            },
        }
    }
}
