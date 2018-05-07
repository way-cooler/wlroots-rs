use libc::{c_double, c_uint};

use wlroots_sys::{wlr_input_device, wlr_input_device_pointer, wlr_input_device_type};

use utils::c_to_rust_string;

/// Wrapper for wlr_input_device
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct InputDevice {
    device: *mut wlr_input_device
}

impl InputDevice {
    /// Just like `std::clone::Clone`, but unsafe.
    ///
    /// # Unsafety
    /// This is unsafe because the user should not be able to clone
    /// this type out because it isn't bound by anything but the underlying
    /// pointer could be removed at any time.
    ///
    /// This isn't exposed to the user, but still marked as `unsafe` to reduce
    /// possible bugs from using this.
    pub(crate) unsafe fn clone(&self) -> Self {
        InputDevice { device: self.device }
    }

    pub fn vendor(&self) -> c_uint {
        unsafe { (*self.device).vendor }
    }

    pub fn product(&self) -> c_uint {
        unsafe { (*self.device).product }
    }

    pub fn name(&self) -> Option<String> {
        unsafe { c_to_rust_string((*self.device).name) }
    }

    /// Get the size in (width_mm, height_mm) format.
    ///
    /// These values will be 0 if it's not supported.
    pub fn size(&self) -> (c_double, c_double) {
        unsafe { ((*self.device).width_mm, (*self.device).height_mm) }
    }

    /// Get the type of the device
    pub fn dev_type(&self) -> wlr_input_device_type {
        unsafe { (*self.device).type_ }
    }

    // TODO Wrapper around the union
    pub unsafe fn dev_union(&self) -> wlr_input_device_pointer {
        (*self.device).__bindgen_anon_1
    }

    pub(crate) unsafe fn from_ptr(device: *mut wlr_input_device) -> Self {
        InputDevice { device: device }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_input_device {
        self.device
    }
}
