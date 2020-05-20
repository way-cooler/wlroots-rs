//! TODO Documentation

use std::{
    cell::Cell,
    fmt,
    marker::PhantomData,
    panic, ptr,
    rc::{Rc, Weak}
};

use crate::libc::{self, c_double, c_int};
use crate::wayland_sys::server::{signal::wl_signal_add, WAYLAND_SERVER_HANDLE};
use wlroots_sys::{
    wlr_output_effective_resolution, wlr_output_layout, wlr_output_layout_add, wlr_output_layout_add_auto,
    wlr_output_layout_closest_point, wlr_output_layout_contains_point, wlr_output_layout_create,
    wlr_output_layout_destroy, wlr_output_layout_get, wlr_output_layout_get_box,
    wlr_output_layout_get_center_output, wlr_output_layout_intersects, wlr_output_layout_move,
    wlr_output_layout_output, wlr_output_layout_output_at, wlr_output_layout_output_coords,
    wlr_output_layout_remove
};

use crate::{
    area::{Area, Origin},
    compositor, output,
    utils::{HandleErr, HandleResult, Handleable}
};

struct OutputLayoutState {
    /// A counter that will always have a strong count of 1.
    ///
    /// Once the output layout is destroyed, this will signal to the
    /// `output::Handle`s that they cannot be upgraded.
    counter: Rc<Cell<bool>>,
    /// A raw pointer to the `output::layout::Layout` on the heap.
    layout: *mut Layout
}

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that's triggered when an output is added to the output layout.
    fn output_added<'this>(
        &'this mut self,
        compositor_handle: compositor::Handle,
        layout_handle: Handle,
        output: Output<'this>
    ) {
    }

    /// Callback that's triggered when an output is removed from the output
    /// layout.
    fn output_removed<'this>(
        &'this mut self,
        compositor_handle: compositor::Handle,
        layout_handle: Handle,
        output: Output<'this>
    ) {
    }

    /// Callback that's triggered when the layout changes.
    fn on_change<'this>(
        &mut self,
        compositor_handle: compositor::Handle,
        layout_handle: Handle,
        output: Output<'this>
    ) {
    }
}

wayland_listener!(pub Layout, (*mut wlr_output_layout, Box<dyn Handler>), [
    output_add_listener => output_add_notify: |this: &mut Layout, data: *mut libc::c_void,|
    unsafe {
        let (output_ptr, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let layout_output = data as *mut wlr_output_layout_output;
        let layout_output = Output{layout_output, phantom: PhantomData};
        let output_layout = Layout::from_ptr(output_ptr);

        manager.output_added(compositor,
                             output_layout.weak_reference(),
                             layout_output);

        Box::into_raw(output_layout);
    };
    output_remove_listener => output_remove_notify: |this: &mut Layout,
                                                     data: *mut libc::c_void,|
    unsafe {
        let (output_ptr, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let layout_output = data as *mut wlr_output_layout_output;
        let layout_output = Output { layout_output, phantom: PhantomData};
        let output_layout = Layout::from_ptr(output_ptr);

        manager.output_removed(compositor,
                               output_layout.weak_reference(),
                               layout_output);

        Box::into_raw(output_layout);
    };
    change_listener => change_notify: |this: &mut Layout, data: *mut libc::c_void,|
    unsafe {
        let (output_ptr, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };
        let layout_output = data as *mut wlr_output_layout_output;
        let layout_output = Output { layout_output, phantom: PhantomData};
        let output_layout = Layout::from_ptr(output_ptr);

        manager.on_change(compositor,
                          output_layout.weak_reference(),
                          layout_output);

        Box::into_raw(output_layout);
    };
]);

impl fmt::Debug for Layout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "")
    }
}

// NOTE We can't use `utils::Handle` because we own the cursor.
// So this is special cased, just like `output::Layout`.
/// A handle to an `output::layout::Layout`.
#[derive(Debug, Clone)]
pub struct Handle {
    /// The Rc that ensures that this handle is still alive.
    ///
    /// When wlroots deallocates the pointer associated with this handle,
    /// this can no longer be used.
    handle: Weak<Cell<bool>>,
    /// The output_layout ptr that refers to this `output::layout::Layout`
    layout: *mut wlr_output_layout
}

/// The coordinate information of an `output::Output` within an
/// `output::layout::Layout`.
#[derive(Debug)]
pub struct Output<'output> {
    layout_output: *mut wlr_output_layout_output,
    phantom: PhantomData<&'output Layout>
}

impl Layout {
    /// Construct a new OuputLayout.
    pub fn create(handler: Box<dyn Handler>) -> Handle {
        unsafe {
            let layout = wlr_output_layout_create();
            if layout.is_null() {
                panic!("Could not allocate a wlr_output_layout")
            }
            let mut output_layout = Layout::new((layout, handler));
            wl_signal_add(
                &mut (*layout).events.add as *mut _ as _,
                output_layout.output_add_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*layout).events.destroy as *mut _ as _,
                output_layout.output_remove_listener() as *mut _ as _
            );
            wl_signal_add(
                &mut (*layout).events.change as *mut _ as _,
                output_layout.change_listener() as *mut _ as _
            );
            let counter = Rc::new(Cell::new(false));
            let handle = Rc::downgrade(&counter);
            let state = Box::new(OutputLayoutState {
                counter,
                layout: Box::into_raw(output_layout)
            });
            (*layout).data = Box::into_raw(state) as *mut libc::c_void;
            Handle { layout, handle }
        }
    }

    pub(crate) unsafe fn as_ptr(&self) -> *mut wlr_output_layout {
        self.data.0
    }

    /// Reconstruct the box from the wlr_output_layout.
    unsafe fn from_ptr(layout: *mut wlr_output_layout) -> Box<Layout> {
        let data = (*layout).data as *mut OutputLayoutState;
        if data.is_null() {
            panic!("Data pointer on the output layout was null!");
        }
        Box::from_raw((*data).layout)
    }

    /// Get the outputs associated with this output::layout::Layout.
    ///
    /// Also returns their absolute position within the layout.
    pub fn outputs(&mut self) -> Vec<(output::Handle, Origin)> {
        unsafe {
            let mut result = vec![];
            wl_list_for_each!((*self.data.0).outputs, link, (pos: wlr_output_layout_output) => {
                result.push((output::Handle::from_ptr((*pos).output),
                             Origin::new((*pos).x, (*pos).y)))
            });
            result
        }
    }

    /// Get the Outputs in the output::layout::Layout coupled with their output
    /// information.
    ///
    /// For a version that isn't bound by lifetimes, see `outputs`.
    pub fn outputs_layouts<'output>(&'output mut self) -> Vec<Output<'output>> {
        unsafe {
            let mut result = vec![];
            wl_list_for_each!((*self.data.0).outputs, link,
            (layout_output: wlr_output_layout_output) => {
                result.push(Output { layout_output,
                                                 phantom: PhantomData
                })
            });
            result
        }
    }

    /// Adds an output to the layout at the given coordinates.
    pub fn add(&mut self, output: &mut output::Output, origin: Origin) {
        let (x, y) = (origin.x, origin.y);
        unsafe { wlr_output_layout_add(self.data.0, output.as_ptr(), x, y) }
    }

    /// Adds an output to the layout, automatically positioning it with
    /// the others that are already there.
    pub fn add_auto(&mut self, output: &mut output::Output) {
        unsafe {
            let layout_handle = self.weak_reference();
            output.set_output_layout(Some(layout_handle));
            wlr_output_layout_add_auto(self.data.0, output.as_ptr());
            wlr_log!(WLR_DEBUG, "Added {:?} to {:?}", output, self);
        }
    }

    /// Moves the output to the given coordinates.
    ///
    /// If the output is not part of this layout this does nothing.
    pub fn move_output(&mut self, output: &mut output::Output, origin: Origin) {
        let (x, y) = (origin.x, origin.y);
        unsafe { wlr_output_layout_move(self.data.0, output.as_ptr(), x, y) }
    }

    /// Get the closest point on this layout from the given point from the
    /// reference output.
    ///
    /// If reference is None, gets the closest point from the entire layout.
    ///
    /// Returns the closest point in the format (x, y)
    pub fn closest_point<'this, O>(&mut self, reference: O, x: f64, y: f64) -> (f64, f64)
    where
        O: Into<Option<&'this mut output::Output>>
    {
        unsafe {
            let output_ptr = reference
                .into()
                .map(|output| output.as_ptr())
                .unwrap_or(ptr::null_mut());
            let (ref mut out_x, ref mut out_y) = (0.0, 0.0);
            wlr_output_layout_closest_point(self.data.0, output_ptr, x, y, out_x, out_y);
            (*out_x, *out_y)
        }
    }

    /// Determines if the `output::layout::Layout` contains the `output::Output`
    /// at the given point.
    pub fn contains_point(&mut self, output: &mut output::Output, origin: Origin) -> bool {
        unsafe { wlr_output_layout_contains_point(self.data.0, output.as_ptr(), origin.x, origin.y) }
    }

    /// Get the box of the layout for the given reference output.
    ///
    /// If `reference` is None, the box will be for the extents of the entire
    /// layout.
    pub fn get_box<'this, O>(&mut self, reference: O) -> Area
    where
        O: Into<Option<&'this mut output::Output>>
    {
        unsafe {
            let output_ptr = reference
                .into()
                .map(|output| output.as_ptr())
                .unwrap_or(ptr::null_mut());
            Area::from_box(*wlr_output_layout_get_box(self.data.0, output_ptr))
        }
    }

    /// Get the output closest to the center of the layout extents, if one
    /// exists.
    pub fn get_center_output(&mut self) -> Option<output::Handle> {
        unsafe {
            let output = wlr_output_layout_get_center_output(self.data.0);
            if output.is_null() {
                None
            } else {
                Some(output::Handle::from_ptr(output))
            }
        }
    }

    /// Determines if the `output::Output` in the `output::layout::Layout`
    /// intersects with the provided `Area`.
    pub fn intersects(&mut self, output: &mut output::Output, area: Area) -> bool {
        unsafe { wlr_output_layout_intersects(self.data.0, output.as_ptr(), &area.into()) }
    }

    /// Given x and y as pointers to global coordinates, adjusts them to local
    /// output coordinates relative to the given reference output.
    pub fn output_coords(&mut self, output: &mut output::Output, x: &mut f64, y: &mut f64) {
        unsafe { wlr_output_layout_output_coords(self.data.0, output.as_ptr(), x, y) }
    }

    /// Remove an output from this layout.
    ///
    /// If the output was not in the layout, does nothing.
    pub fn remove(&mut self, output: &mut output::Output) {
        wlr_log!(WLR_DEBUG, "Removing {:?} from {:?}", output, self);
        unsafe {
            output.clear_output_layout_data();
            wlr_output_layout_remove(self.data.0, output.as_ptr());
        };
    }

    /// Get an output's information about its place in the
    /// `output::layout::Layout`, if it's present.
    pub fn get_output_info<'output>(
        &mut self,
        output: &'output mut output::Output
    ) -> Option<Output<'output>> {
        unsafe {
            let layout_output = wlr_output_layout_get(self.data.0, output.as_ptr());
            if layout_output.is_null() {
                None
            } else {
                Some(Output {
                    layout_output,
                    phantom: PhantomData
                })
            }
        }
    }

    /// Get the output at the given output layout coordinate location, if there
    /// is one there.
    pub fn output_at(&mut self, lx: c_double, ly: c_double) -> Option<output::Handle> {
        unsafe {
            let output = wlr_output_layout_output_at(self.data.0, lx, ly);
            if output.is_null() {
                None
            } else {
                Some(output::Handle::from_ptr(output))
            }
        }
    }

    /// Creates a weak reference to an `output::layout::Layout`.
    ///
    /// # Panics
    /// If this `output::layout::Layout` is a previously upgraded
    /// `output::layout::Handle`, then this function will panic.
    pub fn weak_reference(&self) -> Handle {
        unsafe {
            let handle = Rc::downgrade(&(*((*self.data.0).data as *mut OutputLayoutState)).counter);
            Handle {
                layout: self.data.0,
                handle
            }
        }
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        let layout_ptr = self.data.0;
        unsafe {
            let data = Box::from_raw((*layout_ptr).data as *mut OutputLayoutState);
            let mut manager = Box::from_raw(data.layout);
            assert_eq!(
                Rc::strong_count(&data.counter),
                1,
                "output::layout::Layout had more than 1 reference count"
            );
            (*layout_ptr).data = ptr::null_mut();
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.output_add_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.output_remove_listener()).link as *mut _ as _
            );
            ffi_dispatch!(
                WAYLAND_SERVER_HANDLE,
                wl_list_remove,
                &mut (*manager.change_listener()).link as *mut _ as _
            );
            wlr_output_layout_destroy(self.data.0)
        }
    }
}

impl Handle {
    /// Constructs a new Handle that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the
    /// server, or for mocking/testing.
    pub fn new() -> Self {
        unsafe {
            Handle {
                handle: Weak::new(),
                layout: ptr::null_mut()
            }
        }
    }

    /// Upgrades the `Handle` to a reference
    /// to the backing `output::layout::Layout`.
    ///
    /// # Unsafety
    /// This function is unsafe, because it creates an unbound
    /// `output::layout::Layout` which may live forever..
    /// But the actual lifetime of `output::layout::Layout` is determined by the
    /// user.
    pub(crate) unsafe fn upgrade(&self) -> HandleResult<Box<Layout>> {
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
                Ok(Layout::from_ptr(self.layout))
            })
    }

    /// Run a function on the referenced output::layout::Layout, if it still
    /// exists
    ///
    /// Returns the result of the function, if successful.
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the
    /// output::layout::Layout to a short lived scope of an anonymous
    /// function, this function ensures the output::layout::Layout does not
    /// live longer than it exists (because the lifetime is controlled by
    /// the user).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
    where
        F: FnOnce(&mut Layout) -> R
    {
        let mut output_layout = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(&mut output_layout)));
        if let Some(check) = self.handle.upgrade() {
            // Sanity check that it hasn't been tampered with.
            if !check.get() {
                wlr_log!(
                    WLR_ERROR,
                    "After running output::layout::Layout callback, \
                     mutable lock was false for: {:?}",
                    output_layout
                );
                panic!("Lock in incorrect state!");
            }
            check.set(false);
        };
        Box::into_raw(output_layout);
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }

    /// Destroy the output layout that this handle refers to.
    ///
    /// This will invalidate the other handles, including the ones held by
    /// `output::Output`s.
    ///
    /// If the output layout was previously destroyed, does nothing.
    pub fn destroy(self) {
        unsafe {
            self.upgrade().ok();
        }
    }
}

impl<'output> Output<'output> {
    /// Get a handle to the output that this structure describes.
    pub fn output(&self) -> output::Handle {
        unsafe { output::Handle::from_ptr((*self.layout_output).output) }
    }

    /// Get the coordinates of this output in the layout output.
    pub fn coords(&self) -> (c_int, c_int) {
        unsafe { ((*self.layout_output).x, (*self.layout_output).y) }
    }

    /// Get the absolute top left edge coordinate of this output in the output
    /// layout.
    pub fn top_left_edge(&self) -> Origin {
        unsafe { Origin::new((*self.layout_output).x, (*self.layout_output).y) }
    }

    /// Get the absolute top right edge coordinate of this output in the output
    /// layout.
    pub fn top_right_edge(&self) -> Origin {
        unsafe {
            let (mut width, mut _height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut width, &mut _height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x + width, y)
        }
    }

    pub fn bottom_left_edge(&self) -> Origin {
        unsafe {
            let (mut _width, mut height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut _width, &mut height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x, y + height)
        }
    }

    pub fn bottom_right_edge(&self) -> Origin {
        unsafe {
            let (mut width, mut height) = (0, 0);
            wlr_output_effective_resolution((*self.layout_output).output, &mut width, &mut height);
            let (x, y) = ((*self.layout_output).x, (*self.layout_output).y);
            Origin::new(x + height, y + height)
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
        self.layout == other.layout
    }
}

impl Eq for Handle {}
