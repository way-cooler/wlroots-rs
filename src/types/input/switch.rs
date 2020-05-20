//! TODO Documentation

use std::{cell::Cell, ptr::NonNull, rc::Rc};

pub use crate::events::switch_events as event;
pub use crate::manager::switch_handler::*;
use wlroots_sys::{wlr_input_device, wlr_switch};
use crate::{
    input::{self, InputState},
    utils::{self, HandleErr, HandleResult, Handleable}
};

pub type Handle = utils::Handle<NonNull<wlr_input_device>, wlr_switch, Switch>;

#[derive(Debug)]
pub struct Switch {
    /// The structure that ensures weak handles to this structure are still
    /// alive.
    ///
    /// They contain weak handles, and will safely not use dead memory when this
    /// is freed by wlroots.
    ///
    /// If this is `None`, then this is from an upgraded `pointer::Handle`, and
    /// the operations are **unchecked**.
    /// This is means safe operations might fail, but only if you use the unsafe
    /// marked function `upgrade` on a `pointer::Handle`.
    liveliness: Rc<Cell<bool>>,
    /// The device that refers to this pointer.
    device: input::Device,
    /// The underlying switch data.
    switch: NonNull<wlr_switch>
}

impl Switch {
    /// Tries to convert an input device to a Switch
    ///
    /// Returns none if it is of a different input variant.
    ///
    /// # Safety
    /// This creates a totally new Switch (e.g with its own reference count)
    /// so only do this once per `wlr_input_device`!
    pub(crate) unsafe fn new_from_input_device(device: *mut wlr_input_device) -> Option<Self> {
        use wlroots_sys::wlr_input_device_type::*;
        match (*device).type_ {
            WLR_INPUT_DEVICE_SWITCH => {
                let switch = NonNull::new((*device).__bindgen_anon_1.switch_device).expect(
                    "Switch pointer \
                     was null"
                );
                let liveliness = Rc::new(Cell::new(false));
                let handle = Rc::downgrade(&liveliness);
                let state = Box::new(InputState {
                    handle,
                    device: input::Device::from_ptr(device)
                });
                (*switch.as_ptr()).data = Box::into_raw(state) as *mut _;
                Some(Switch {
                    liveliness,
                    device: input::Device::from_ptr(device),
                    switch
                })
            },
            _ => None
        }
    }

    /// Gets the wlr_input_device associated with this switch.
    pub fn input_device(&self) -> &input::Device {
        &self.device
    }
}

impl Drop for Switch {
    fn drop(&mut self) {
        if Rc::strong_count(&self.liveliness) == 1 {
            wlr_log!(WLR_DEBUG, "Dropped Switch {:p}", self.switch.as_ptr());
            unsafe {
                let _ = Box::from_raw((*self.switch.as_ptr()).data as *mut InputState);
            }
            let weak_count = Rc::weak_count(&self.liveliness);
            if weak_count > 0 {
                wlr_log!(
                    WLR_DEBUG,
                    "Still {} weak pointers to Switch {:p}",
                    weak_count,
                    self.switch.as_ptr()
                );
            }
        }
    }
}

impl Handleable<NonNull<wlr_input_device>, wlr_switch> for Switch {
    #[doc(hidden)]
    unsafe fn from_ptr(switch: *mut wlr_switch) -> Option<Self> {
        let switch = NonNull::new(switch)?;
        let data = Box::from_raw((*switch.as_ptr()).data as *mut InputState);
        let handle = data.handle.clone();
        let device = data.device.clone();
        (*switch.as_ptr()).data = Box::into_raw(data) as *mut _;
        Some(Switch {
            liveliness: handle.upgrade().unwrap(),
            device,
            switch
        })
    }

    #[doc(hidden)]
    unsafe fn as_ptr(&self) -> *mut wlr_switch {
        self.switch.as_ptr()
    }

    #[doc(hidden)]
    unsafe fn from_handle(handle: &Handle) -> HandleResult<Self> {
        let liveliness = handle.handle.upgrade().ok_or(HandleErr::AlreadyDropped)?;
        let device = handle.data.ok_or(HandleErr::AlreadyDropped)?;
        Ok(Switch {
            liveliness,
            // NOTE Rationale for cloning:
            // If we already dropped we don't reach this point.
            device: input::Device { device },
            switch: handle.as_non_null()
        })
    }

    fn weak_reference(&self) -> Handle {
        Handle {
            ptr: self.switch,
            handle: Rc::downgrade(&self.liveliness),
            // NOTE Rationale for cloning:
            // Since we have a strong reference already,
            // the input must still be alive.
            data: unsafe { Some(self.device.as_non_null()) },
            _marker: std::marker::PhantomData
        }
    }
}
