//! TODO Documentation

use std::marker::PhantomData;

use wlroots_sys::wlr_output_mode;

use crate::output::Output;

#[derive(Debug, Eq, PartialEq)]
pub struct Mode<'output> {
    output_mode: *mut wlr_output_mode,
    phantom: PhantomData<&'output Output>
}

impl<'output> Mode<'output> {
    /// NOTE This is a lifetime defined by the user of this function, but it
    /// must not outlive the `Output` that hosts this output mode.
    pub(crate) unsafe fn new<'unbound>(output_mode: *mut wlr_output_mode) -> Mode<'unbound> {
        Mode {
            output_mode,
            phantom: PhantomData
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_mode {
        self.output_mode
    }

    /// Gets the dimensions of this Mode.
    ///
    /// Returned value is (width, height)
    pub fn dimensions(&self) -> (i32, i32) {
        unsafe { ((*self.output_mode).width, (*self.output_mode).height) }
    }

    /// Get the refresh value of the output.
    pub fn refresh(&self) -> i32 {
        unsafe { (*self.output_mode).refresh }
    }
}
