//! Wrapper for wlr_seat. For more information about what a seat is, please
//! consult the Wayland documentation ([libinput docs](https://wayland.freedesktop.org/libinput/doc/latest/seats.html), [wayland docs](https://wayland.freedesktop.org/docs/html/apa.html#protocol-spec-wl_seat))
//!
//! TODO This module could really use some examples, as the API surface is huge.

use std::{
    cell::Cell,
    fmt, panic, ptr,
    rc::{Rc, Weak},
    time::Duration
};

use crate::libc;
use crate::wayland_sys::server::{signal::wl_signal_add, WAYLAND_SERVER_HANDLE};
pub use wlroots_sys::wayland_server::protocol::wl_seat::Capability;
use wlroots_sys::{wlr_axis_orientation, wlr_axis_source, wlr_seat, wlr_seat_create, wlr_seat_destroy, wlr_seat_get_keyboard, wlr_seat_keyboard_clear_focus, wlr_seat_keyboard_end_grab, wlr_seat_keyboard_enter, wlr_seat_keyboard_has_grab, wlr_seat_keyboard_notify_enter, wlr_seat_keyboard_notify_key, wlr_seat_keyboard_notify_modifiers, wlr_seat_keyboard_send_key, wlr_seat_keyboard_send_modifiers, wlr_seat_keyboard_start_grab, wlr_seat_pointer_clear_focus, wlr_seat_pointer_end_grab, wlr_seat_pointer_enter, wlr_seat_pointer_has_grab, wlr_seat_pointer_notify_axis, wlr_seat_pointer_notify_button, wlr_seat_pointer_notify_enter, wlr_seat_pointer_notify_motion, wlr_seat_pointer_request_set_cursor_event, wlr_seat_pointer_send_axis, wlr_seat_pointer_send_button, wlr_seat_pointer_send_motion, wlr_seat_pointer_start_grab, wlr_seat_pointer_surface_has_focus, wlr_seat_set_capabilities, wlr_seat_set_keyboard, wlr_seat_set_name, wlr_seat_touch_end_grab, wlr_seat_touch_get_point, wlr_seat_touch_has_grab, wlr_seat_touch_notify_down, wlr_seat_touch_notify_motion, wlr_seat_touch_notify_up, wlr_seat_touch_num_points, wlr_seat_touch_point_clear_focus, wlr_seat_touch_point_focus, wlr_seat_touch_send_down, wlr_seat_touch_send_motion, wlr_seat_touch_send_up, wlr_seat_touch_start_grab, wlr_button_state};
use xkbcommon::xkb::Keycode;

pub use crate::events::seat_events as event;
use crate::{
    compositor::{self, Compositor},
    input::{self, keyboard},
    seat::{
        self, grab,
        touch_point::{TouchId, TouchPoint}
    },
    surface::Surface,
    utils::{c_to_rust_string, safe_as_cstring, HandleErr, HandleResult, ToMs}
};

struct SeatState {
    /// A counter that will always have a strong count of 1.
    ///
    /// Once the seat is destroyed, this will signal to the `seat::Handle`s that
    /// they cannot be upgraded.
    counter: Rc<Cell<bool>>,
    /// A raw pointer to the Seat on the heap.
    seat: *mut Seat
}

#[derive(Debug, Clone)]
pub struct Handle {
    seat: *mut wlr_seat,
    handle: Weak<Cell<bool>>
}

#[allow(unused_variables)]
pub trait Handler {
    /// Callback triggered when a client has grabbed a pointer.
    fn pointer_grabbed(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &grab::Pointer
    ) {
    }

    /// Callback triggered when a client has ended a pointer grab.
    fn pointer_released(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &grab::Pointer
    ) {
    }

    /// Callback triggered when a client has grabbed a keyboard.
    fn keyboard_grabbed(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &grab::Keyboard
    ) {
    }

    /// Callback triggered when a client has ended a keyboard grab.
    fn keyboard_released(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &grab::Keyboard
    ) {
    }

    /// Callback triggered when a client has grabbed a touch.
    fn touch_grabbed(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &grab::Touch
    ) {
    }

    /// Callback triggered when a client has ended a touch grab.
    fn touch_released(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &grab::Touch
    ) {
    }

    /// Callback triggered when a client sets the cursor for this seat.
    ///
    /// E.g this happens when the seat enters a surface.
    fn cursor_set(
        &mut self,
        compositor_handle: compositor::Handle,
        seat_handle: Handle,
        event: &seat::event::SetCursor
    ) {
    }

    /// The seat was provided with a selection by the client.
    fn set_selection(&mut self, compositor_handle: compositor::Handle, seat_handle: Handle) {}

    /// The seat was provided with a selection from the primary buffer
    /// by the client.
    fn set_primary_selection(&mut self, compositor_handle: compositor::Handle, seat_handle: Handle) {}

    /// The seat is being destroyed.
    fn destroy(&mut self, compositor_handle: compositor::Handle, seat_handle: Handle) {}
}

wayland_listener!(pub Seat, (*mut wlr_seat, Box<dyn Handler>), [
    pointer_grab_begin_listener => pointer_grab_begin_notify: |this: &mut Seat,
                                                               event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let pointer_grab = grab::Pointer::from_ptr(event as _);
        let seat = Seat::from_ptr(seat_ptr);

        handler.pointer_grabbed(compositor,
                                seat.weak_reference(),
                                &pointer_grab);

        Box::into_raw(seat);
    };

    pointer_grab_end_listener => pointer_grab_end_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let pointer_grab = grab::Pointer::from_ptr(event as _);
        let seat = Seat::from_ptr(seat_ptr);

        handler.pointer_released(compositor,
                                 seat.weak_reference(),
                                 &pointer_grab);

        Box::into_raw(seat);
    };
    keyboard_grab_begin_listener => keyboard_grab_begin_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let keyboard_grab = grab::Keyboard::from_ptr(event as _);
        let seat = Seat::from_ptr(seat_ptr);

        handler.keyboard_grabbed(compositor,
                                 seat.weak_reference(),
                                 &keyboard_grab);

        Box::into_raw(seat);
    };
    keyboard_grab_end_listener => keyboard_grab_end_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let keyboard_grab = grab::Keyboard::from_ptr(event as _);
        let seat = Seat::from_ptr(seat_ptr);

        handler.keyboard_released(compositor,
                                  seat.weak_reference(),
                                  &keyboard_grab);

        Box::into_raw(seat);
    };
    touch_grab_begin_listener => touch_grab_begin_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let touch_grab = grab::Touch::from_ptr(event as _);
        let seat = Seat::from_ptr(seat_ptr);

        handler.touch_grabbed(compositor,
                              seat.weak_reference(),
                              &touch_grab);

        Box::into_raw(seat);
    };
    touch_grab_end_listener => touch_grab_end_notify: |this: &mut Seat,
    event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let touch_grab = grab::Touch::from_ptr(event as _);
        let seat = Seat::from_ptr(seat_ptr);

        handler.touch_released(compositor,
                               seat.weak_reference(),
                               &touch_grab);

        Box::into_raw(seat);
    };
    request_set_cursor_listener => request_set_cursor_notify: |this: &mut Seat,
    event_ptr: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let event_ptr = event_ptr as *mut wlr_seat_pointer_request_set_cursor_event;
        let event = seat::event::SetCursor::from_ptr(event_ptr);
        let seat = Seat::from_ptr(seat_ptr);

        handler.cursor_set(compositor,
                           seat.weak_reference(),
                           &event);

        Box::into_raw(seat);
    };
    set_selection_listener => selection_notify: |this: &mut Seat, _event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let seat = Seat::from_ptr(seat_ptr);

        handler.set_selection(compositor, seat.weak_reference());

        Box::into_raw(seat);
    };
    set_primary_selection_listener => primary_selection_notify: |this: &mut Seat,
    _event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let seat = Seat::from_ptr(seat_ptr);

        handler.set_primary_selection(compositor, seat.weak_reference());

        Box::into_raw(seat);
    };
    destroy_listener => destroy_notify: |this: &mut Seat, _event: *mut libc::c_void,|
    unsafe {
        let (seat_ptr, ref mut handler) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let seat = Seat::from_ptr(seat_ptr);

        handler.destroy(compositor, seat.weak_reference());

        // NOTE Destructor is already being run,
        // otherwise this would be a double free.
        Box::into_raw(seat);
    };
]);

impl Seat {
    /// Allocates a new `wlr_seat` and adds a wl_seat global to the display.
    pub fn create(compositor: &mut Compositor, name: String, handler: Box<dyn Handler>) -> Handle {
        unsafe {
            let name = safe_as_cstring(name);
            let seat = wlr_seat_create(compositor.display as _, name.as_ptr());
            if seat.is_null() {
                panic!("Could not allocate a wlr_seat");
            }
            let mut res = Seat::new((seat, handler));
            wl_signal_add(
                &mut (*seat).events.pointer_grab_begin as *mut _ as _,
                res.pointer_grab_begin_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.pointer_grab_end as *mut _ as _,
                res.pointer_grab_end_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.keyboard_grab_begin as *mut _ as _,
                res.keyboard_grab_begin_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.keyboard_grab_end as *mut _ as _,
                res.keyboard_grab_end_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.touch_grab_begin as *mut _ as _,
                res.touch_grab_begin_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.touch_grab_end as *mut _ as _,
                res.touch_grab_end_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.request_set_cursor as *mut _ as _,
                res.request_set_cursor_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.set_selection as *mut _ as _,
                res.set_selection_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.set_primary_selection as *mut _ as _,
                res.set_primary_selection_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*seat).events.destroy as *mut _ as _,
                res.destroy_listener() as *mut _ as _
            );
            let counter = Rc::new(Cell::new(false));
            let handle = Rc::downgrade(&counter);
            let state = Box::new(SeatState {
                counter,
                seat: Box::into_raw(res)
            });
            (*seat).data = Box::into_raw(state) as *mut libc::c_void;
            Handle { seat, handle }
        }
    }

    /// Reconstruct the box from the wlr_seat.
    unsafe fn from_ptr(seat: *mut wlr_seat) -> Box<Seat> {
        let data = (*seat).data as *mut SeatState;
        if data.is_null() {
            panic!("Data pointer on the seat was null!");
        }
        Box::from_raw((*data).seat)
    }

    /// Get a weak reference to this seat.
    pub fn weak_reference(&self) -> Handle {
        unsafe {
            let handle = Rc::downgrade(&(*((*self.data.0).data as *mut SeatState)).counter);
            Handle {
                seat: self.data.0,
                handle
            }
        }
    }

    /// Get the name of the seat.
    pub fn name(&self) -> Option<String> {
        unsafe {
            let name_ptr = (*self.data.0).name;
            if name_ptr.is_null() {
                return None;
            }
            c_to_rust_string(name_ptr)
        }
    }

    /// Updates the name of this seat.
    /// Will automatically send it to all clients.
    pub fn set_name(&mut self, name: String) {
        let name = safe_as_cstring(name);
        unsafe {
            wlr_seat_set_name(self.data.0, name.as_ptr());
        }
    }

    /// Gets the capabilities of this seat.
    pub fn capabilities(&self) -> Capability {
        unsafe { Capability::from_raw((*self.data.0).capabilities).expect("Invalid capabilities") }
    }

    /// Updates the capabilities available on this seat.
    /// Will automatically send it to all clients.
    pub fn set_capabilities(&mut self, capabilities: Capability) {
        unsafe { wlr_seat_set_capabilities(self.data.0, capabilities.bits()) }
    }

    /// Determines if the surface has pointer focus.
    pub fn pointer_surface_has_focus(&self, surface: &mut Surface) -> bool {
        unsafe { wlr_seat_pointer_surface_has_focus(self.data.0, surface.as_ptr()) }
    }

    // Sends a pointer enter event to the given surface and considers it to be
    // the focused surface for the pointer.
    //
    // This will send a leave event to the last surface that was entered.
    //
    // Coordinates for the enter event are surface-local.
    //
    // Compositor should use `Seat::pointer_notify_enter` to
    // change pointer focus to respect pointer grabs.
    pub fn pointer_enter(&self, surface: &mut Surface, sx: f64, sy: f64) {
        unsafe {
            wlr_seat_pointer_enter(self.data.0, surface.as_ptr(), sx, sy);
        }
    }

    /// Clears the focused surface for the pointer and leaves all entered
    /// surfaces.
    pub fn pointer_clear_focus(&self) {
        unsafe { wlr_seat_pointer_clear_focus(self.data.0) }
    }

    /// Sends a motion event to the surface with pointer focus.
    ///
    /// Coordinates for the motion event are surface-local.
    ///
    /// Compositors should use `Seat::notify_motion` to
    /// send motion events to the respect pointer grabs.
    pub fn send_motion(&self, time: Duration, sx: f64, sy: f64) {
        unsafe { wlr_seat_pointer_send_motion(self.data.0, time.to_ms(), sx, sy) }
    }

    // TODO Button and State should probably be wrapped in some sort of type...

    /// Send a button event to the surface with pointer focus.
    ///
    /// Coordinates for the button event are surface-local.
    ///
    /// Returns the serial.
    ///
    /// Compositors should use `Seat::notify_button` to
    /// send button events to respect pointer grabs.
    pub fn send_button(&self, time: Duration, button: u32, state: wlr_button_state) -> u32 {
        unsafe { wlr_seat_pointer_send_button(self.data.0, time.to_ms(), button, state) }
    }

    /// Send an axis event to the surface with pointer focus.
    ///
    /// Compositors should use `Seat::notify_axis` to
    /// send axis events to respect pointer grabs.
    pub fn send_axis(
        &self,
        time: Duration,
        orientation: wlr_axis_orientation,
        value: f64,
        value_discrete: i32,
        source: wlr_axis_source
    ) {
        unsafe {
            wlr_seat_pointer_send_axis(
                self.data.0,
                time.to_ms(),
                orientation,
                value,
                value_discrete,
                source
            );
        }
    }

    /// Start a grab of the pointer of this seat. The grabber is responsible for
    /// handling all pointer events until the grab ends.
    pub fn pointer_start_grab(&self, grab: grab::Pointer) {
        unsafe { wlr_seat_pointer_start_grab(self.data.0, grab.as_ptr()) }
    }

    /// End the grab of the pointer of this seat. This reverts the grab back to
    /// the default grab for the pointer.
    pub fn pointer_end_grab(&self) {
        unsafe { wlr_seat_pointer_end_grab(self.data.0) }
    }

    /// Whether or not the pointer has a grab other than the default grab.
    pub fn pointer_has_grab(&self) -> bool {
        unsafe { wlr_seat_pointer_has_grab(self.data.0) }
    }

    /// Notify the seat of a pointer enter event to the given surface and
    /// request it to be the focused surface for the pointer.
    ///
    /// Pass surface-local coordinates where the enter occurred.
    pub fn pointer_notify_enter(&self, surface: &mut Surface, sx: f64, sy: f64) {
        unsafe { wlr_seat_pointer_notify_enter(self.data.0, surface.as_ptr(), sx, sy) }
    }

    /// Notify the seat of motion over the given surface.
    ///
    /// Pass surface-local coordinates where the pointer motion occurred.
    pub fn pointer_notify_motion(&self, time: Duration, sx: f64, sy: f64) {
        unsafe { wlr_seat_pointer_notify_motion(self.data.0, time.to_ms(), sx, sy) }
    }

    // TODO Wrapper type around Button and State

    /// Notify the seat that a button has been pressed.
    ///
    /// Returns the serial of the button press or zero if no button press was
    /// sent.
    pub fn pointer_notify_button(&self, time: Duration, button: u32, state: wlr_button_state) -> u32 {
        unsafe { wlr_seat_pointer_notify_button(self.data.0, time.to_ms(), button, state) }

    }

    /// Notify the seat of an axis event.
    pub fn pointer_notify_axis(
        &self,
        time: Duration,
        orientation: wlr_axis_orientation,
        value: f64,
        value_discrete: i32,
        source: wlr_axis_source
    ) {
        unsafe {
            wlr_seat_pointer_notify_axis(
                self.data.0,
                time.to_ms(),
                orientation,
                value,
                value_discrete,
                source
            )
        }
    }

    /// Set this keyboard as the active keyboard for the seat.
    pub fn set_keyboard(&mut self, dev: &input::Device) {
        unsafe { wlr_seat_set_keyboard(self.data.0, dev.as_ptr()) }
    }

    // TODO Point to the correct function name in this documentation.

    /// Send the keyboard key to focused keyboard resources.
    ///
    /// Compositors should use `wlr_seat_notify_key()` to respect keyboard
    /// grabs.
    pub fn keyboard_send_key(&self, time: Duration, key: u32, state: u32) {
        unsafe { wlr_seat_keyboard_send_key(self.data.0, time.to_ms(), key, state) }
    }

    /// Send the modifier state to focused keyboard resources.
    ///
    /// Compositors should use `Seat::keyboard_notify_modifiers()` to respect
    /// any keyboard grabs.
    pub fn keyboard_send_modifiers(&self, modifiers: &mut keyboard::Modifiers) {
        let mut mods = (*modifiers).into();
        unsafe { wlr_seat_keyboard_send_modifiers(self.data.0, &mut mods) }
    }

    /// Get the keyboard associated with this Seat, if there is one.
    pub fn get_keyboard(&self) -> Option<keyboard::Handle> {
        unsafe {
            let keyboard_ptr = wlr_seat_get_keyboard(self.data.0);
            if keyboard_ptr.is_null() {
                None
            } else {
                Some(keyboard::Handle::from_ptr(keyboard_ptr))
            }
        }
    }

    /// Notify the seat that the keyboard focus has changed and request it to be
    /// the focused surface for this keyboard.
    ///
    /// Defers to any current grab of the seat's keyboard.
    pub fn keyboard_notify_enter(
        &self,
        surface: &mut Surface,
        keycodes: &mut [Keycode],
        modifiers: &mut keyboard::Modifiers
    ) {
        let keycodes_length = keycodes.len();
        let mut mods = (*modifiers).into();
        unsafe {
            wlr_seat_keyboard_notify_enter(
                self.data.0,
                surface.as_ptr(),
                keycodes.as_mut_ptr(),
                keycodes_length,
                &mut mods
            )
        }
    }

    /// Send a keyboard enter event to the given surface and consider it to be
    /// the focused surface for the keyboard.
    ///
    /// This will send a leave event to the last surface that was entered.
    ///
    /// Compositors should use `Seat::keyboard_notify_enter()` to
    /// change keyboard focus to respect keyboard grabs.
    pub fn keyboard_enter(
        &self,
        surface: &mut Surface,
        keycodes: &mut [Keycode],
        modifiers: &mut keyboard::Modifiers
    ) {
        let keycodes_length = keycodes.len();
        let mut mods = (*modifiers).into();
        unsafe {
            wlr_seat_keyboard_enter(
                self.data.0,
                surface.as_ptr(),
                keycodes.as_mut_ptr(),
                keycodes_length,
                &mut mods
            )
        }
    }

    /// Start a grab of the keyboard of this seat. The grabber is responsible
    /// for handling all keyboard events until the grab ends.
    pub fn keyboard_start_grab(&self, grab: grab::Keyboard) {
        unsafe { wlr_seat_keyboard_start_grab(self.data.0, grab.as_ptr()) }
    }

    /// End the grab of the keyboard of this seat. This reverts the grab back to
    /// the default grab for the keyboard.
    pub fn keyboard_end_grab(&self) {
        unsafe { wlr_seat_keyboard_end_grab(self.data.0) }
    }

    /// Whether or not the keyboard has a grab other than the default grab
    pub fn keyboard_has_grab(&self) -> bool {
        unsafe { wlr_seat_keyboard_has_grab(self.data.0) }
    }

    /// Clear the focused surface for the keyboard and leave all entered
    /// surfaces.
    pub fn keyboard_clear_focus(&self) {
        unsafe { wlr_seat_keyboard_clear_focus(self.data.0) }
    }

    /// Notify the seat that the modifiers for the keyboard have changed.
    ///
    /// Defers to any keyboard grabs.
    pub fn keyboard_notify_modifiers(&self, modifiers: &mut keyboard::Modifiers) {
        let mut mods = (*modifiers).into();
        unsafe { wlr_seat_keyboard_notify_modifiers(self.data.0, &mut mods) }
    }

    // TODO Wrapper type for Key and State

    /// Notify the seat that a key has been pressed on the keyboard.
    ///
    /// Defers to any keyboard grabs.
    pub fn keyboard_notify_key(&self, time: Duration, key: u32, state: u32) {
        unsafe { wlr_seat_keyboard_notify_key(self.data.0, time.to_ms(), key, state) }
    }

    /// How many touch ponits are currently down for the seat.
    pub fn touch_num_points(&self) -> i32 {
        unsafe { wlr_seat_touch_num_points(self.data.0) }
    }

    /// Start a grab of the touch device of this seat. The grabber is
    /// responsible for handling all touch events until the grab ends.
    pub fn touch_start_grab(&self, grab: grab::Touch) {
        unsafe { wlr_seat_touch_start_grab(self.data.0, grab.as_ptr()) }
    }

    /// End the grab of the touch device of this seat. This reverts the grab
    /// back to the default grab for the touch device.
    pub fn touch_end_grab(&self) {
        unsafe { wlr_seat_touch_end_grab(self.data.0) }
    }

    /// Whether or not the seat has a touch grab other than the default grab.
    pub fn touch_has_grab(&self) -> bool {
        unsafe { wlr_seat_touch_has_grab(self.data.0) }
    }

    // Get the active touch point with the given `touch_id`. If the touch point does
    // not exist or is no longer active, returns None.
    pub fn get_touch_point(&self, touch_id: TouchId) -> Option<TouchPoint> {
        unsafe {
            let touch_point = wlr_seat_touch_get_point(self.data.0, touch_id.into());
            if touch_point.is_null() {
                None
            } else {
                Some(TouchPoint::from_ptr(touch_point))
            }
        }
    }

    /// Notify the seat that the touch point given by `touch_id` has entered a
    /// new surface.
    ///
    /// The surface is required. To clear focus, use
    /// `Seat::touch_point_clear_focus()`.
    pub fn touch_point_focus(
        &self,
        surface: &mut Surface,
        time: Duration,
        touch_id: TouchId,
        sx: f64,
        sy: f64
    ) {
        unsafe {
            wlr_seat_touch_point_focus(
                self.data.0,
                surface.as_ptr(),
                time.to_ms(),
                touch_id.into(),
                sx,
                sy
            )
        }
    }

    //// Clear the focused surface for the touch point given by `touch_id`.
    pub fn touch_point_clear_focus(&self, time: Duration, touch_id: TouchId) {
        unsafe { wlr_seat_touch_point_clear_focus(self.data.0, time.to_ms(), touch_id.into()) }
    }

    /// Send a touch down event to the client of the given surface.
    ///
    /// All future touch events for this point will go to this surface.
    ///
    /// If the touch down is valid, this will add a new touch point with the
    /// given `touch_id`.
    ///
    /// The touch down may not be valid if the surface seat client does not
    /// accept touch input.
    ///
    /// Coordinates are surface-local.
    ///
    /// Compositors should use `Seat::touch_notify_down()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_down(
        &self,
        surface: &mut Surface,
        time: Duration,
        touch_id: TouchId,
        sx: f64,
        sy: f64
    ) -> u32 {
        unsafe {
            wlr_seat_touch_send_down(
                self.data.0,
                surface.as_ptr(),
                time.to_ms(),
                touch_id.into(),
                sx,
                sy
            )
        }
    }

    /// Send a touch up event for the touch point given by the `touch_id`.
    ///
    /// The event will go to the client for the surface given in the
    /// cooresponding touch down event.
    ///
    /// This will remove the touch point.
    ///
    /// Compositors should use `Seat::touch_notify_up()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_up(&self, time: Duration, touch_id: TouchId) {
        unsafe { wlr_seat_touch_send_up(self.data.0, time.to_ms(), touch_id.into()) }
    }

    /// Send a touch motion event for the touch point given by the `touch_id`.
    ///
    /// The event will go to the client for the surface given in the
    /// corresponding touch down event.
    ///
    /// Compositors should use `Seat::touch_notify_motion()` to
    /// respect any grabs of the touch device.
    pub fn touch_send_motion(&self, time: Duration, touch_id: TouchId, sx: f64, sy: f64) {
        unsafe { wlr_seat_touch_send_motion(self.data.0, time.to_ms(), touch_id.into(), sx, sy) }
    }

    // TODO Should this be returning a u32? Should I wrap whatever that number is?

    /// Notify the seat of a touch down on the given surface. Defers to any grab
    /// of the touch device.
    pub fn touch_notify_down(
        &self,
        surface: &mut Surface,
        time: Duration,
        touch_id: TouchId,
        sx: f64,
        sy: f64
    ) -> u32 {
        unsafe {
            wlr_seat_touch_notify_down(
                self.data.0,
                surface.as_ptr(),
                time.to_ms(),
                touch_id.into(),
                sx,
                sy
            )
        }
    }

    /// Notify the seat that the touch point given by `touch_id` is up. Defers
    /// to any grab of the touch device.
    pub fn touch_notify_up(&self, time: Duration, touch_id: TouchId) {
        unsafe { wlr_seat_touch_notify_up(self.data.0, time.to_ms(), touch_id.into()) }
    }

    /// Notify the seat that the touch point given by `touch_id` has moved.
    ///
    /// Defers to any grab of the touch device.
    ///
    /// The seat should be notified of touch motion even if the surface is
    /// not the owner of the touch point for processing by grabs.
    pub fn touch_notify_motion(&self, time: Duration, touch_id: TouchId, sx: f64, sy: f64) {
        unsafe { wlr_seat_touch_notify_motion(self.data.0, time.to_ms(), touch_id.into(), sx, sy) }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_seat {
        self.data.0
    }
}

impl fmt::Debug for Seat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Seat {:p}", self.data.0)
    }
}

impl Drop for Seat {
    fn drop(&mut self) {
        let seat_ptr = self.data.0;
        unsafe {
            let data = Box::from_raw((*seat_ptr).data as *mut SeatState);
            let mut manager = Box::from_raw(data.seat);
            assert_eq!(
                Rc::strong_count(&data.counter),
                1,
                "Seat had more than 1 reference count"
            );
            (*seat_ptr).data = ptr::null_mut();
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.pointer_grab_begin_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.pointer_grab_end_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.keyboard_grab_begin_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.keyboard_grab_end_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.touch_grab_begin_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.touch_grab_end_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.request_set_cursor_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.set_selection_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.set_primary_selection_listener()).link as *mut _ as _
            );
            wlr_seat_destroy(seat_ptr);
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.destroy_listener()).link as *mut _ as _
            );
        }
    }
}

impl Handle {
    /// Constructs a new seat::Handle that is always invalid. Calling `run` on
    /// this will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the
    /// server, or for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            Handle {
                handle: Weak::new(),
                seat: ptr::null_mut()
            }
        }
    }

    /// Creates an seat::Handle from the raw pointer, using the saved
    /// user data to recreate the memory model.
    pub(crate) unsafe fn from_ptr(seat: *mut wlr_seat) -> Self {
        if (*seat).data.is_null() {
            panic!("Seat data was null!")
        }
        let data = Box::from_raw((*seat).data as *mut SeatState);
        let handle = Rc::downgrade(&data.counter);
        Box::into_raw(data);
        Handle { seat, handle }
    }

    /// Upgrades the seat handle to a reference to the backing `Seat`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Seat`
    /// which may live forever..
    /// But a seat could be destroyed else where.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Box<Seat>> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                check.set(true);
                Ok(Seat::from_ptr(self.seat))
            })
    }

    /// Run a function on the referenced Seat, if it still exists
    ///
    /// If the Seat is returned, then the Seat is not deallocated.
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the Seat
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Seat does not live during a callback
    /// (at which point you would have aliased mutability).
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Seat`.
    ///
    /// So don't nest `run` calls or call this in a Seat callback
    /// and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
    where
        F: FnOnce(&mut Seat) -> R
    {
        let mut seat = unsafe { self.upgrade()? };
        let seat_ptr = seat.data.0;
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut seat)));
        Box::into_raw(seat);
        if let Some(check) = self.handle.upgrade() {
            // Sanity check that it hasn't been tampered with.
            if !check.get() {
                wlr_log!(
                    WLR_ERROR,
                    "After running seat callback, mutable lock was \
                     false for {:p}",
                    seat_ptr
                );
                panic!("Lock in incorrect state!");
            }
            check.set(false);
        };
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Destroy the seat that this handle refers to.
    ///
    /// This will invalidate the other handles.
    ///
    /// If the seat was previously destroyed, does nothing.
    pub fn destroy(self) {
        unsafe {
            self.upgrade().ok();
        }
    }
}

impl Default for Handle {
    fn default() -> Self {
        Handle::new()
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Handle) -> bool {
        self.seat == other.seat
    }
}

impl Eq for Handle {}
