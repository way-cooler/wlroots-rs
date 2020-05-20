//! Manager that is called when an output is created or destroyed.

use std::{marker::PhantomData, panic, ptr::NonNull};

use crate::libc;
use crate::wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::wlr_output;

use crate::{
    compositor,
    output::{self, Output, OutputState, UserOutput},
    utils::Handleable
};

/// Used to ensure the output sets the mode before doing any other
/// operation on the Output.
pub struct OutputBuilder<'output> {
    output: output::Handle,
    phantom: PhantomData<&'output Output>
}

/// Used to ensure that the builder is used to construct
/// the output::Handler instance.
pub struct BuilderResult<'output> {
    pub output: output::Handle,
    result: Box<dyn output::Handler>,
    phantom: PhantomData<&'output Output>
}

/// Wrapper around Output destruction so that you can't call
/// unsafe methods (e.g anything like setting the mode).
pub struct Destroyed(output::Handle);

impl<'output> OutputBuilder<'output> {
    /// Build the output with the best mode.
    ///
    /// To complete construction, return this in your implementation of
    /// `output::ManagerHandler::output_added`.
    pub fn build_best_mode<T: output::Handler + 'static>(mut self, data: T) -> BuilderResult<'output> {
        with_handles!([(output: {&mut self.output})] => {
            output.choose_best_mode();
        })
        .expect("Output was borrowed");
        BuilderResult {
            output: self.output,
            result: Box::new(data),
            phantom: PhantomData
        }
    }
}

impl Destroyed {
    // TODO Functions which are safe to use
}

pub type OutputAdded =
    fn(compositor_handle: compositor::Handle, output_builder: OutputBuilder) -> Option<BuilderResult>;

wayland_listener_static! {
    static mut MANAGER;
    (Manager, Builder): [
        (OutputAdded, add_listener, output_added) => (add_notify, add_callback):
        |manager: &mut Manager, data: *mut libc::c_void,| unsafe {
            let data = data as *mut wlr_output;
            let output = Output::new(data as *mut wlr_output);
            // NOTE
            // This clone is required because we pass it mutably to the output builder,
            // but due to lack of NLL there's no way to tell Rust it's safe to use it in
            // in the if branch.
            //
            // Thus, we need to clone it here and then drop the original once at the end.
            //
            // This is not a real clone, but an pub(crate) unsafe one we added, so it doesn't
            // break safety concerns in user code. Just an unfortunate hack we have to put here.
            let output_clone = output.clone();
            let builder = OutputBuilder { output: output.weak_reference(), phantom: PhantomData };
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            let res = panic::catch_unwind(
                panic::AssertUnwindSafe(|| manager.add_callback
                                        .map(|f| f(compositor, builder))
                                        .unwrap_or(None)));
            let build_result = match res {
                Ok(res) => res,
                // NOTE
                // Either Wayland or wlroots does not handle failure to set up output correctly.
                // Calling wl_display_terminate does not work if output is incorrectly set up.
                //
                // Instead, execution keeps going with an eventual segfault (if lucky).
                //
                // To fix this, we abort the process if there was a panic in output setup.
                Err(_) => ::std::process::abort()
            };
            if let Some(BuilderResult {result: output_ptr, .. }) = build_result {
                let mut output = UserOutput::new((output_clone, output_ptr));
                wl_signal_add(&mut (*data).events.frame as *mut _ as _,
                              output.frame_listener() as _);
                wl_signal_add(&mut (*data).events.mode as *mut _ as _,
                              output.mode_listener() as _);
                wl_signal_add(&mut (*data).events.enable as *mut _ as _,
                              output.enable_listener() as _);
                wl_signal_add(&mut (*data).events.scale as *mut _ as _,
                              output.scale_listener() as _);
                wl_signal_add(&mut (*data).events.transform as *mut _ as _,
                              output.transform_listener() as _);
                wl_signal_add(&mut (*data).events.precommit as *mut _ as _,
                              output.swap_buffers_listener() as _);
                wl_signal_add(&mut (*data).events.damage as *mut _ as _,
                              output.need_swap_listener() as _);
                wl_signal_add(&mut (*data).events.destroy as *mut _ as _,
                              output.on_destroy_listener() as _);
                let output_data = (*data).data as *mut OutputState;
                (*output_data).output = NonNull::new(Box::into_raw(output));
            }
        };
    ]
}
