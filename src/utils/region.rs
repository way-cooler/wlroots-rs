//! Helper functions for manipulating Pixman regions from the Pixman library.
//!
//! [The Pixman library](http://www.pixman.org/) is a library for pixel
//! manipulation.

use std::mem;

use crate::libc::{c_double, c_float, c_int};
use wlroots_sys::{
    pixman_region32_init, pixman_region32_t, wl_output_transform, wlr_region_confine, wlr_region_expand,
    wlr_region_rotated_bounds, wlr_region_scale, wlr_region_transform
};

/// A thin wrapper around a 32 bit Pixman region.
#[derive(Default)]
pub struct PixmanRegion32 {
    pub region: pixman_region32_t
}

impl PixmanRegion32 {
    /// Construct a new Pixman region.
    pub fn new() -> Self {
        unsafe {
            // NOTE This is safe because the init function properly
            // sets up the fields.
            let mut region: pixman_region32_t = mem::MaybeUninit::uninit().assume_init();
            pixman_region32_init(&mut region);
            PixmanRegion32 { region }
        }
    }

    /// Scales a region, ie. multiplies all its coordinates by `scale`
    /// and write out the result to `dest`.
    ///
    /// The resulting coordinates are rounded up or down so that the new region
    /// is at least as big as the original one.
    pub fn scale(&self, dest: &mut PixmanRegion32, scale: c_float) {
        unsafe {
            let region_ptr = &self.region as *const _ as *mut _;
            wlr_region_scale(&mut dest.region, region_ptr, scale);
        }
    }

    /// Applies a transform to a region inside a box of size `width` x `height`.
    /// Writes the result to `dest`.
    pub fn transform(
        &self,
        dest: &mut PixmanRegion32,
        transform: wl_output_transform,
        width: c_int,
        height: c_int
    ) {
        unsafe {
            let region_ptr = &self.region as *const _ as *mut _;
            wlr_region_transform(&mut dest.region, region_ptr, transform, width, height);
        }
    }

    /// Expands the region of `distance`. If `distance` is negative, it shrinks
    /// the region. Writes the result to the `dest`.
    pub fn expand(&self, dest: &mut PixmanRegion32, distance: c_int) {
        unsafe {
            let region_ptr = &self.region as *const _ as *mut _;
            wlr_region_expand(&mut dest.region, region_ptr, distance);
        }
    }

    /// Builds the smallest possible region that contains the region rotated
    /// about the point in output space (ox, oy).
    /// Writes the result to the `dest`.
    pub fn rotated_bounds(&self, dest: &mut PixmanRegion32, rotation: c_float, ox: c_int, oy: c_int) {
        unsafe {
            let region_ptr = &self.region as *const _ as *mut _;
            wlr_region_rotated_bounds(&mut dest.region, region_ptr, rotation, ox, oy);
        }
    }

    /// Confines a region to the box formed by the points.
    ///
    /// If it could not be confined by the points it will return an error.
    pub fn confine(
        &mut self,
        x1: c_double,
        y1: c_double,
        x2: c_double,
        y2: c_double
    ) -> Result<(c_double, c_double), ()> {
        unsafe {
            let (mut x_out, mut y_out) = (0.0, 0.0);
            let res = wlr_region_confine(&mut self.region, x1, y1, x2, y2, &mut x_out, &mut y_out);
            if res {
                Ok((x_out, y_out))
            } else {
                Err(())
            }
        }
    }
}
