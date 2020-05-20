use std::mem;

use crate::libc::{c_int, c_uint};
use wlroots_sys::{
    pixman_region32_fini, pixman_region32_init, pixman_region32_t, pixman_region32_union_rect
};

/// A pixman region, used for damage tracking.
#[derive(Debug, Default)]
pub struct PixmanRegion {
    pub region: pixman_region32_t
}

impl PixmanRegion {
    /// Make a new pixman region.
    pub fn new() -> Self {
        unsafe {
            // NOTE Rational for uninitialized memory:
            // We are automatically filling it in with pixman_region32_init.
            let mut region = mem::MaybeUninit::uninit().assume_init();
            pixman_region32_init(&mut region);
            PixmanRegion { region }
        }
    }

    pub fn rectangle(&mut self, x: c_int, y: c_int, width: c_uint, height: c_uint) {
        unsafe {
            let region_ptr = &mut self.region as *mut _;
            pixman_region32_union_rect(region_ptr, region_ptr, x, y, width, height);
        }
    }
}

impl Drop for PixmanRegion {
    fn drop(&mut self) {
        unsafe { pixman_region32_fini(&mut self.region) }
    }
}
