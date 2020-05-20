use std::marker::PhantomData;

use wlroots_sys::{wlr_xwayland_surface_hints, wlr_xwayland_surface_size_hints};

use crate::xwayland;

/// Hints provided by the XWayland client to aid in compositing.
pub struct Hints<'surface> {
    hints: *mut wlr_xwayland_surface_hints,
    phantom: PhantomData<&'surface xwayland::surface::Surface>
}

/// Hints provided by the XWayland client to aid in compositing specifically
/// for placement.
pub struct SizeHints<'surface> {
    hints: *mut wlr_xwayland_surface_size_hints,
    phantom: PhantomData<&'surface xwayland::surface::Surface>
}

impl<'surface> Hints<'surface> {
    pub(crate) unsafe fn from_ptr(hints: *mut wlr_xwayland_surface_hints) -> Self {
        Hints {
            hints,
            phantom: PhantomData
        }
    }

    pub fn flags(&self) -> u32 {
        unsafe { (*self.hints).flags }
    }

    pub fn input(&self) -> u32 {
        unsafe { (*self.hints).input }
    }

    pub fn initial_state(&self) -> i32 {
        unsafe { (*self.hints).initial_state }
    }

    pub unsafe fn icon_pixmap(&self) -> u32 {
        (*self.hints).icon_pixmap
    }

    pub unsafe fn icon_window(&self) -> u32 {
        (*self.hints).icon_window
    }

    /// Get the coordinates of the icon.
    ///
    /// Return format is (x, y).
    pub fn icon_coords(&self) -> (i32, i32) {
        unsafe { ((*self.hints).icon_x, (*self.hints).icon_y) }
    }

    pub unsafe fn icon_mask(&self) -> u32 {
        (*self.hints).icon_mask
    }

    pub unsafe fn window_group(&self) -> u32 {
        (*self.hints).window_group
    }
}

impl<'surface> SizeHints<'surface> {
    pub(crate) unsafe fn from_ptr(hints: *mut wlr_xwayland_surface_size_hints) -> Self {
        SizeHints {
            hints,
            phantom: PhantomData
        }
    }

    /// Get the flags associated with the surface size.
    pub fn flags(&self) -> u32 {
        unsafe { (*self.hints).flags }
    }

    /// Get the coordinates of the surface.
    ///
    /// Return format is (x, y).
    pub fn coords(&self) -> (i32, i32) {
        unsafe { ((*self.hints).x, (*self.hints).y) }
    }

    /// Get the dimensions of the surface.
    ///
    /// Return format is (width, height).
    pub fn dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.hints).width, (*self.hints).height) }
    }

    /// Get the minimal allowed dimensions of the surface.
    ///
    /// Return format is (width, height).
    pub fn min_dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.hints).min_width, (*self.hints).min_height) }
    }

    /// Get the maximal allowed dimensions of the surface.
    ///
    /// Return format is (width, height).
    pub fn max_dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.hints).max_width, (*self.hints).max_height) }
    }

    /// TODO What is this
    ///
    /// Return format is (width, height).
    pub fn inc_dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.hints).width_inc, (*self.hints).height_inc) }
    }

    /// TODO What is this
    ///
    /// Return format is (width, height).
    pub fn base_dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.hints).base_width, (*self.hints).base_height) }
    }

    pub fn min_aspect_num(&self) -> i32 {
        unsafe { (*self.hints).min_aspect_num }
    }

    pub fn min_aspect_den(&self) -> i32 {
        unsafe { (*self.hints).min_aspect_den }
    }

    pub fn max_aspect_num(&self) -> i32 {
        unsafe { (*self.hints).max_aspect_num }
    }

    pub fn max_aspect_den(&self) -> i32 {
        unsafe { (*self.hints).max_aspect_den }
    }

    pub fn win_gravity(&self) -> u32 {
        unsafe { (*self.hints).win_gravity }
    }
}
