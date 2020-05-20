//! Wrapper for wlr_cursor

use std::{
    cell::Cell,
    fmt, panic, ptr,
    rc::{Rc, Weak}
};

use crate::libc;
use crate::wayland_sys::server::signal::wl_signal_add;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::{
    wlr_cursor, wlr_cursor_absolute_to_layout_coords, wlr_cursor_attach_input_device,
    wlr_cursor_attach_output_layout, wlr_cursor_create, wlr_cursor_destroy, wlr_cursor_detach_input_device,
    wlr_cursor_map_input_to_output, wlr_cursor_map_input_to_region, wlr_cursor_map_to_output,
    wlr_cursor_map_to_region, wlr_cursor_move, wlr_cursor_set_image, wlr_cursor_set_surface, wlr_cursor_warp,
    wlr_cursor_warp_absolute
};

use crate::{
    area::Area,
    compositor,
    cursor::xcursor,
    input::{self, pointer, tablet_tool, touch},
    output::{self, layout::Layout, Output},
    surface::Surface,
    utils::{HandleErr, HandleResult, Handleable}
};

#[derive(Debug)]
pub(crate) struct CursorState {
    output_layout: Option<output::layout::Handle>,
    /// A counter that will always have a strong count of 1.
    ///
    /// Once the cursor is destroyed, this will signal to the `cursor::Handle`s
    /// that they cannot be upgraded.
    counter: Rc<Cell<bool>>,
    /// A raw pointer to the Cursor on the heap
    cursor: *mut Cursor
}

// NOTE We can't use `utils::Handle` because we own the cursor.
// So this is special cased, just like `output::Layout`.
#[derive(Debug, Clone)]
pub struct Handle {
    cursor: *mut wlr_cursor,
    handle: Weak<Cell<bool>>
}

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that is triggered when the cursor moves.
    fn on_pointer_motion(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &pointer::event::Motion
    ) {
    }

    fn on_pointer_motion_absolute(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &pointer::event::AbsoluteMotion
    ) {
    }

    /// Callback that is triggered when the buttons on the pointer are pressed.
    fn on_pointer_button(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &pointer::event::Button
    ) {
    }

    fn on_pointer_axis(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &pointer::event::Axis
    ) {
    }

    fn on_touch_up(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &touch::event::Up
    ) {
    }

    fn on_touch_down(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &touch::event::Down
    ) {
    }

    fn on_touch_motion(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &touch::event::Motion
    ) {
    }

    fn on_touch_cancel(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &touch::event::Cancel
    ) {
    }

    fn on_tablet_tool_axis(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &tablet_tool::event::Axis
    ) {
    }

    fn on_tablet_tool_proximity(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &tablet_tool::event::Proximity
    ) {
    }

    fn on_tablet_tool_tip(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &tablet_tool::event::Tip
    ) {
    }

    fn on_tablet_tool_button(
        &mut self,
        compositor_handle: compositor::Handle,
        cursor_handle: Handle,
        event: &tablet_tool::event::Button
    ) {
    }
}

wayland_listener!(pub Cursor, (*mut wlr_cursor, Box<dyn Handler>, Option<output::layout::Handle>), [
    pointer_motion_listener => pointer_motion_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = pointer::event::Motion::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_pointer_motion(compositor,
                                         cursor.weak_reference(),
                                         &event);

        Box::into_raw(cursor);
    };
    pointer_motion_absolute_listener => pointer_motion_absolute_notify:
    |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let event = pointer::event::AbsoluteMotion::from_ptr(event as _);
        let cursor = Cursor::from_ptr(cursor_ptr);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_pointer_motion_absolute(compositor,
                                                  cursor.weak_reference(),
                                                  &event);

        Box::into_raw(cursor);
    };
    pointer_button_listener => pointer_button_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = pointer::event::Button::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_pointer_button(compositor,
                                         cursor.weak_reference(),
                                         &event);

        Box::into_raw(cursor);
    };
    pointer_axis_listener => pointer_axis_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = pointer::event::Axis::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_pointer_axis(compositor,
                                       cursor.weak_reference(),
                                       &event);

        Box::into_raw(cursor);
    };
    touch_up_listener => touch_up_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = touch::event::Up::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_touch_up(compositor,
                                   cursor.weak_reference(),
                                   &event);

        Box::into_raw(cursor);
    };
    touch_down_listener => touch_down_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = touch::event::Down::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_touch_down(compositor,
                                     cursor.weak_reference(),
                                     &event);

        Box::into_raw(cursor);
    };
    touch_motion_listener => touch_motion_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = touch::event::Motion::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_touch_motion(compositor,
                                       cursor.weak_reference(),
                                       &event);

        Box::into_raw(cursor);
    };
    touch_cancel_listener => touch_cancel_notify: |this: &mut Cursor, event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = touch::event::Cancel::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_touch_cancel(compositor,
                                       cursor.weak_reference(),
                                       &event);

        Box::into_raw(cursor);
    };
    tablet_tool_axis_listener => tablet_tool_axis_notify: |this: &mut Cursor,
                                                           event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = tablet_tool::event::Axis::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_tablet_tool_axis(compositor,
                                           cursor.weak_reference(),
                                           &event);

        Box::into_raw(cursor);
    };
    tablet_tool_proximity_listener => tablet_tool_proximity_notify: |this: &mut Cursor,
                                                                     event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = tablet_tool::event::Proximity::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_tablet_tool_proximity(compositor,
                                                cursor.weak_reference(),
                                                &event);

        Box::into_raw(cursor);
    };
    tablet_tool_tip_listener => tablet_tool_tip_notify: |this: &mut Cursor,
                                                         event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = tablet_tool::event::Tip::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_tablet_tool_tip(compositor,
                                          cursor.weak_reference(),
                                          &event);

        Box::into_raw(cursor);
    };
    tablet_tool_button_listener => tablet_tool_button_notify: |this: &mut Cursor,
                                                               event: *mut libc::c_void,|
    unsafe {
        let (cursor_ptr, ref mut cursor_handler, _) = this.data;
        let cursor = Cursor::from_ptr(cursor_ptr);
        let event = tablet_tool::event::Button::from_ptr(event as _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        cursor_handler.on_tablet_tool_button(compositor,
                                             cursor.weak_reference(),
                                             &event);

        Box::into_raw(cursor);
    };
]);

impl fmt::Debug for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", self.data.0)
    }
}

impl Cursor {
    pub fn create(cursor_handler: Box<dyn Handler>) -> Handle {
        unsafe {
            let cursor_ptr = wlr_cursor_create();
            if cursor_ptr.is_null() {
                panic!("Could not create wlr_cursor")
            }
            let mut cursor = Cursor::new((cursor_ptr, cursor_handler, None));
            wl_signal_add(
                &mut (*cursor_ptr).events.motion as *mut _ as _,
                cursor.pointer_motion_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.motion_absolute as *mut _ as _,
                cursor.pointer_motion_absolute_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.button as *mut _ as _,
                cursor.pointer_button_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.axis as *mut _ as _,
                cursor.pointer_axis_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.touch_up as *mut _ as _,
                cursor.touch_up_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.touch_down as *mut _ as _,
                cursor.touch_down_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.touch_motion as *mut _ as _,
                cursor.touch_motion_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.touch_cancel as *mut _ as _,
                cursor.touch_cancel_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.tablet_tool_axis as *mut _ as _,
                cursor.tablet_tool_axis_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.tablet_tool_proximity as *mut _ as _,
                cursor.tablet_tool_proximity_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.tablet_tool_tip as *mut _ as _,
                cursor.tablet_tool_tip_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*cursor_ptr).events.tablet_tool_button as *mut _ as _,
                cursor.tablet_tool_button_listener() as *mut _ as _
            );
            let counter = Rc::new(Cell::new(false));
            let handle = Rc::downgrade(&counter);
            let state = Box::new(CursorState {
                counter,
                cursor: Box::into_raw(cursor),
                output_layout: None
            });
            (*cursor_ptr).data = Box::into_raw(state) as *mut libc::c_void;
            Handle {
                cursor: cursor_ptr,
                handle
            }
        }
    }

    unsafe fn from_ptr(cursor: *mut wlr_cursor) -> Box<Cursor> {
        let data = (*cursor).data as *mut CursorState;
        if data.is_null() {
            panic!("Data pointer on the cursor was null!");
        }
        Box::from_raw((*data).cursor)
    }

    pub(crate) fn as_ptr(&self) -> *mut wlr_cursor {
        self.data.0
    }

    /// Get a weak reference to this `Cursor`.
    pub fn weak_reference(&self) -> Handle {
        unsafe {
            let handle = Rc::downgrade(&(*((*self.data.0).data as *mut CursorState)).counter);
            Handle {
                cursor: self.data.0,
                handle
            }
        }
    }

    /// Attach this cursor to an output layout.
    pub fn attach_output_layout(&mut self, output_layout: &mut Layout) {
        unsafe {
            let weak_reference = Some(output_layout.weak_reference().clone());
            self.data.2 = weak_reference.clone();
            let mut data = Box::from_raw((*self.data.0).data as *mut CursorState);
            data.output_layout = weak_reference;
            (*self.data.0).data = Box::into_raw(data) as *mut libc::c_void;
            wlr_cursor_attach_output_layout(self.data.0, output_layout.as_ptr());
        }
    }

    pub fn deattach_output_layout(&mut self) {
        unsafe {
            let weak_reference = None;
            self.data.2 = weak_reference.clone();
            let mut data = Box::from_raw((*self.data.0).data as *mut CursorState);
            data.output_layout = weak_reference;
            (*self.data.0).data = Box::into_raw(data) as *mut libc::c_void;
            wlr_cursor_attach_output_layout(self.data.0, ptr::null_mut());
        }
    }

    /// Get the coordinates the cursor is located at.
    pub fn coords(&self) -> (f64, f64) {
        unsafe { ((*self.data.0).x, (*self.data.0).y) }
    }

    /// Warp the cursor to the given x and y in layout coordinates. If x and y
    /// are out of the layout boundaries or constraints, no warp will
    /// happen.
    ///
    /// `dev` may be passed to respect device mapping constraints. If `dev` is
    /// None, device mapping constraints will be ignored.
    ///
    /// Returns true when the mouse warp was successful.
    pub fn warp<'this, O>(&'this mut self, dev: O, x: f64, y: f64) -> bool
    where
        O: Into<Option<&'this input::Device>>
    {
        self.assert_layout();
        unsafe {
            let dev_ptr = dev
                .into()
                .map(|input_device| input_device.as_ptr())
                .unwrap_or(ptr::null_mut());
            wlr_cursor_warp(self.data.0, dev_ptr, x, y)
        }
    }

    pub fn warp_absolute<'this, O>(&'this mut self, dev: O, x_mm: f64, y_mm: f64)
    where
        O: Into<Option<&'this input::Device>>
    {
        self.assert_layout();
        unsafe {
            let dev_ptr = dev
                .into()
                .map(|input_device| input_device.as_ptr())
                .unwrap_or(ptr::null_mut());
            wlr_cursor_warp_absolute(self.data.0, dev_ptr, x_mm, y_mm)
        }
    }

    /// Move the cursor in the direction of the given x and y coordinates.
    ///
    /// `dev` may be passed to respect device mapping constraints. If `dev` is
    /// None, device mapping constraints will be ignored.
    pub fn move_relative<'this, O>(&'this mut self, dev: O, delta_x: f64, delta_y: f64)
    where
        O: Into<Option<&'this input::Device>>
    {
        self.assert_layout();
        unsafe {
            let dev_ptr = dev.into().map(|dev| dev.as_ptr()).unwrap_or(ptr::null_mut());
            wlr_cursor_move(self.data.0, dev_ptr, delta_x, delta_y)
        }
    }

    // TODO USE IMAGE
    /// Sets the image of the cursor to the image.
    pub fn set_cursor_image(&mut self, image: &xcursor::Image) {
        unsafe {
            // NOTE Rationale for why lifetime isn't attached:
            //
            // wlr_cursor_set_image uses gl calls internally, which copies
            // the buffer and so it doesn't matter what happens to the
            // xcursor image after this call.
            wlr_cursor_set_image(
                self.data.0,
                image.buffer.as_ptr(),
                (image.width * 4) as i32,
                image.width,
                image.height,
                image.hotspot_x as _,
                image.hotspot_y as _,
                1.0
            )
        }
    }

    /// Set the cursor surface. The surface can be committed to update the
    /// cursor image. The surface position is substracted from the hotspot.
    ///
    /// A `None` surface commit hides the cursor.
    pub fn set_surface<'this, O>(&'this mut self, surface: O, hotspot_x: i32, hotspot_y: i32)
    where
        O: Into<Option<&'this Surface>>
    {
        unsafe {
            let surface_ptr = surface
                .into()
                .map(|surface| surface.as_ptr())
                .unwrap_or(ptr::null_mut());
            wlr_cursor_set_surface(self.data.0, surface_ptr, hotspot_x, hotspot_y)
        }
    }

    /// Attaches this input device to this cursor. The input device must be one
    /// of:
    ///
    /// - WLR_INPUT_DEVICE_POINTER
    /// - WLR_INPUT_DEVICE_TOUCH
    /// - WLR_INPUT_DEVICE_TABLET_TOOL
    ///
    /// TODO Make this impossible to mess up with using an enum
    /// Note that it's safe to use the wrong type.
    pub fn attach_input_device(&mut self, dev: &input::Device) {
        // NOTE Rationale for not storing handle:
        //
        // Internally, on the destroy event this will automatically
        // destroy the internal wlr_cursor_device used to refer to
        // this input::Device.
        unsafe { wlr_cursor_attach_input_device(self.data.0, dev.as_ptr()) }
    }

    /// Deattaches the input device from this cursor.
    pub fn deattach_input_device(&mut self, dev: &input::Device) {
        unsafe { wlr_cursor_detach_input_device(self.data.0, dev.as_ptr()) }
    }

    /// Attaches this cursor to the given output, which must be among the
    /// outputs in the current output_layout for this cursor.
    pub fn map_to_output<'a, T: Into<Option<&'a mut Output>>>(&mut self, output: T) {
        self.assert_layout();
        match output.into() {
            None => unsafe { wlr_cursor_map_to_output(self.data.0, ptr::null_mut()) },
            Some(output) => {
                if !self.output_in_output_layout(output.weak_reference()) {
                    wlr_log!(WLR_ERROR, "Tried to map output not in the Layout");
                    return;
                }
                unsafe { wlr_cursor_map_to_output(self.data.0, output.as_ptr()) }
            }
        }
    }

    /// Maps all input from a specific input device to a given output.
    ///
    /// The input device must be attached to this cursor
    /// and the output must be among the outputs in the attached output layout.
    pub fn map_input_to_output<'output, O>(&mut self, dev: &input::Device, output: O)
    where
        O: Into<Option<&'output Output>>
    {
        self.assert_layout();
        // NOTE Rationale for why we don't check input:
        //
        // If the input isn't found, then wlroots prints a diagnostic and
        // returns early (and thus does nothing unsafe).

        match output.into() {
            None => unsafe { wlr_cursor_map_input_to_output(self.data.0, dev.as_ptr(), ptr::null_mut()) },
            Some(output) => {
                if !self.output_in_output_layout(output.weak_reference()) {
                    wlr_log!(WLR_ERROR, "Tried to map input to an output not in the Layout");
                    return;
                }
                unsafe { wlr_cursor_map_input_to_output(self.data.0, dev.as_ptr(), output.as_ptr()) }
            }
        }
    }

    /// Maps this cursor to an arbitrary region on the associated
    /// wlr_output_layout.
    pub fn map_to_region(&mut self, area: Area) {
        self.assert_layout();
        unsafe { wlr_cursor_map_to_region(self.data.0, &mut area.into()) }
    }

    /// Maps inputs from this input device to an arbitrary region on the
    /// associated wlr_output_layout.
    ///
    /// The input device must be attached to this cursor.
    pub fn map_input_to_region(&mut self, dev: &input::Device, area: Area) {
        self.assert_layout();
        // NOTE Rationale for why we don't check input:
        //
        // If the input isn't found, then wlroots prints a diagnostic and
        // returns early (and thus does nothing unsafe).
        unsafe { wlr_cursor_map_input_to_region(self.data.0, dev.as_ptr(), &mut area.into()) }
    }

    /// Convert absolute coordinates to layout coordinates for the device.
    ///
    /// Coordinates are in (x, y).
    pub fn absolute_to_layout_coords(&mut self, dev: &input::Device, x_mm: f64, y_mm: f64) -> (f64, f64) {
        self.assert_layout();
        unsafe {
            let (mut lx, mut ly) = (0.0, 0.0);
            wlr_cursor_absolute_to_layout_coords(self.data.0, dev.as_ptr(), x_mm, y_mm, &mut lx, &mut ly);
            (lx, ly)
        }
    }

    /// Determines if we are within a valid layout.
    fn assert_layout(&self) {
        match self.data.2.clone().map(|layout| layout.run(|_| ())) {
            Some(Ok(())) | Some(Err(HandleErr::AlreadyBorrowed)) => {},
            None | Some(Err(_)) => panic!("Cursor was not attached to an output layout!")
        }
    }

    /// Checks if the output is in the Layout associated with this
    /// cursor.
    ///
    /// If it isn't, or the Layout has been dropped, this returns `false`.
    /// Otherwise it returns `true`.
    fn output_in_output_layout(&mut self, output: output::Handle) -> bool {
        self.assert_layout();
        match self.data.2.clone().unwrap().run(|output_layout| {
            for (cur_output, _) in output_layout.outputs() {
                if cur_output == output {
                    return true;
                }
            }
            false
        }) {
            Ok(res) => res,
            Err(HandleErr::AlreadyDropped) => false,
            Err(err) => panic!(err)
        }
    }
}

impl Drop for Cursor {
    fn drop(&mut self) {
        wlr_log!(WLR_DEBUG, "Dropped {:?}", self);
        let cursor_ptr = self.data.0;
        unsafe {
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.pointer_motion_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.pointer_motion_absolute_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.pointer_button_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.pointer_axis_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.touch_up_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.touch_down_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.touch_motion_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.touch_cancel_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.tablet_tool_axis_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.tablet_tool_proximity_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.tablet_tool_tip_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*self.tablet_tool_button_listener()).link as *mut _ as _
            );
            let data = Box::from_raw((*cursor_ptr).data as *mut CursorState);
            let _ = Box::from_raw(data.cursor);
            assert_eq!(
                Rc::strong_count(&data.counter),
                1,
                "Cursor had more than 1 reference count"
            );
            (*cursor_ptr).data = ptr::null_mut();
            wlr_cursor_destroy(self.data.0)
        }
    }
}

impl Handle {
    /// Constructs a `cursor::Handle` that is always invalid. Calling `run` on
    /// this will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the
    /// server, or for mocking/testing.
    pub fn new() -> Self {
        Handle {
            handle: Weak::new(),
            cursor: ptr::null_mut()
        }
    }
    /// Upgrades the cursor handle to a reference to the backing `Cursor`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound `Cursor`
    /// which may live forever..
    /// But a cursor could be destoryed else where.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Box<Cursor>> {
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
                Ok(Cursor::from_ptr(self.cursor))
            })
    }

    /// Run a function on the referenced Cursor, if it still exists
    ///
    /// If the Cursor is returned, then the Cursor is not deallocated.
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the Cursor
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Cursor does not live during a callback
    /// (at which point you would have aliased mutability).
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Cursor`.
    ///
    /// So don't nest `run` calls or call this in a Cursor callback
    /// and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
    where
        F: FnOnce(&mut Cursor) -> R
    {
        let mut cursor = unsafe { self.upgrade()? };
        let cursor_ptr = cursor.data.0;
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut cursor)));
        Box::into_raw(cursor);
        if let Some(check) = self.handle.upgrade() {
            // Sanity check that it hasn't been tampered with.
            if !check.get() {
                wlr_log!(
                    WLR_ERROR,
                    "After running cursor callback, mutable lock was \
                     false for {:p}",
                    cursor_ptr
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

    /// Destroy the cursor that this handle refers to.
    ///
    /// This will invalidate the other handles.
    ///
    /// If the seat was previously destroyed, does nothing
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
