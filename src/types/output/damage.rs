use std::{ptr, time::Duration};

use wlroots_sys::{
    wlr_output, wlr_output_damage, wlr_output_damage_add, wlr_output_damage_add_box,
    wlr_output_damage_add_whole, wlr_output_damage_create, wlr_output_damage_destroy,
    wlr_output_damage_attach_render, wlr_output_set_damage, wlr_output_commit
};

use crate::{area::Area, render::PixmanRegion};

#[derive(Debug)]
/// Tracks damage for an output.
///
/// When a `frame` event is emitted, `make_current` should be
/// called. If necessary, the output should be repainted and
/// `swap_buffers` should be called.
///
/// No rendering should happen outside a `frame` event handler.
pub struct Damage {
    damage: *mut wlr_output_damage
}

impl Damage {
    /// Makes a new `Damage` bound to the given Output.
    ///
    /// # Safety
    /// This function is unsafe because the `Damage` should not outlive the
    /// past in `Output`.
    pub(crate) unsafe fn new(output: *mut wlr_output) -> Self {
        unsafe {
            let damage = wlr_output_damage_create(output);
            if damage.is_null() {
                panic!("Damage was none")
            }
            Damage { damage }
        }
    }

    pub(crate) unsafe fn from_ptr(damage: *mut wlr_output_damage) -> Self {
        Damage { damage }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_damage {
        self.damage
    }

    /// Just like `std::clone::Clone` but unsafe.
    ///
    /// # Unsafety
    /// You can create a reference leak using this method very easily,
    /// and it could make it so that weak handles to an output are dangling.
    ///
    /// This exists due to an issue in output_manager.rs that might be fixed
    /// with NLL, so if this is no longer necessary it should be removed asap.
    pub(crate) unsafe fn clone(&self) -> Self {
        Damage { damage: self.damage }
    }

    /// Makes the output rendering context current.
    /// Returns `true` if `wlr_output_damage_swap_buffers` needs to be called.
    ///
    /// The region of the output that needs to be repainted is added to
    /// `damage`.
    pub fn make_current<'a, T>(&mut self, damage: T) -> bool
    where
        T: Into<Option<&'a mut PixmanRegion>>
    {
        unsafe {
            let mut res = false;
            let damage = match damage.into() {
                Some(region) => &mut region.region as *mut _,
                None => ptr::null_mut()
            };
            wlr_output_damage_attach_render(self.damage, &mut res, damage);
            res
        }
    }

    /// Swaps the output buffers.
    ///
    /// If the time of the frame isn't known, set `when` to `None`.
    ///
    /// Swapping buffers schedules a `frame` event.
    pub fn swap_buffers<'a, T, U>(&mut self, _when: T, damage: U) -> bool
    where
        T: Into<Option<Duration>>,
        U: Into<Option<&'a mut PixmanRegion>>
    {
        unsafe {
            let damage = match damage.into() {
                Some(region) => &mut region.region as *mut _,
                None => ptr::null_mut()
            };

            wlr_output_set_damage((*self.damage).output, damage);
            wlr_output_commit((*self.damage).output)
        }
    }

    /// Accumulates damage and schedules a `frame` event.
    pub fn add(&mut self, damage: &mut PixmanRegion) {
        unsafe {
            wlr_output_damage_add(self.damage, &mut damage.region);
        }
    }

    /// Damages the whole output and schedules a `frame` event.
    pub fn add_whole(&mut self) {
        unsafe { wlr_output_damage_add_whole(self.damage) }
    }

    /// Accumulates damage from an `Area` and schedules a `frame` event.
    pub fn add_area(&mut self, area: Area) {
        unsafe { wlr_output_damage_add_box(self.damage, &mut area.into()) }
    }
}

impl Drop for Damage {
    fn drop(&mut self) {
        wlr_log!(WLR_DEBUG, "Dropped Damage {:p}", self.damage);
        unsafe {
            wlr_output_damage_destroy(self.damage);
        }
    }
}
