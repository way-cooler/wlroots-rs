use wlroots_sys::{wlr_xwayland_move_event, wlr_xwayland_resize_event, wlr_xwayland_surface_configure_event};

use crate::{utils::edges::Edges, xwayland};

/// Event for when XWayland surface needs to be configured.
pub struct Configure {
    event: *mut wlr_xwayland_surface_configure_event
}

/// Event for when an XWayland surface is moved.
pub struct Move {
    event: *mut wlr_xwayland_move_event
}

/// Event for when an XWayland surface is resized.
pub struct Resize {
    event: *mut wlr_xwayland_resize_event
}

impl Configure {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xwayland_surface_configure_event) -> Self {
        Configure { event }
    }

    /// Get the surface associated with this configure event.
    pub fn surface(&self) -> Option<xwayland::surface::Handle> {
        unsafe {
            if (*self.event).surface.is_null() {
                None
            } else {
                Some(xwayland::surface::Handle::from_ptr((*self.event).surface))
            }
        }
    }

    /// Get the coordinates for where the XWayland surface wants to be.
    ///
    /// Return format is (x, y).
    pub fn coords(&self) -> (i16, i16) {
        unsafe { ((*self.event).x, (*self.event).y) }
    }

    /// Get the dimensions the XWayland surface wants to have.
    ///
    /// Return format is (width, height).
    pub fn dimensions(&self) -> (u16, u16) {
        unsafe { ((*self.event).width, (*self.event).height) }
    }
}

impl Move {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xwayland_move_event) -> Self {
        Move { event }
    }

    /// Get the surface associated with this move event.
    pub fn surface(&self) -> Option<xwayland::surface::Handle> {
        unsafe {
            if (*self.event).surface.is_null() {
                None
            } else {
                Some(xwayland::surface::Handle::from_ptr((*self.event).surface))
            }
        }
    }
}

impl Resize {
    pub(crate) unsafe fn from_ptr(event: *mut wlr_xwayland_resize_event) -> Self {
        Resize { event }
    }

    /// Get the surface associated with this resize event.
    pub fn surface(&self) -> Option<xwayland::surface::Handle> {
        unsafe {
            if (*self.event).surface.is_null() {
                None
            } else {
                Some(xwayland::surface::Handle::from_ptr((*self.event).surface))
            }
        }
    }

    /// Get the resize edge information for the resize action.
    pub fn edges(&self) -> Edges {
        unsafe {
            let edges_bits = (*self.event).edges;
            match Edges::from_bits(edges_bits) {
                Some(edges) => edges,
                None => panic!("got invalid edges: {}", edges_bits)
            }
        }
    }
}
