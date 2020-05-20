//! Handler for outputs

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::wlr_output;

use crate::{
    compositor,
    output::{self, Output, OutputState},
    utils::{HandleErr, Handleable}
};

#[allow(unused_variables)]
pub trait Handler {
    /// Called every time the output frame is updated.
    fn on_frame(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called every time the output mode changes.
    fn on_mode_change(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called every time the output is enabled.
    fn on_enable(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called every time the output scale changes.
    fn on_scale_change(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called every time the output transforms.
    fn on_transform(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called every time the buffers are swapped on an output.
    fn on_buffers_swapped(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called every time the buffers need to be swapped on an output.
    fn needs_swap(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}

    /// Called when an output is destroyed (e.g. unplugged).
    fn destroyed(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {}
}

wayland_listener!(pub(crate) UserOutput, (Output, Box<dyn Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut UserOutput, data: *mut libc::c_void,|
    unsafe {
        let output_ptr = data as *mut wlr_output;
        {
            let (ref mut output, ref mut manager) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            manager.destroyed(compositor, output.weak_reference());
            // NOTE Remove the output from the output if there is one.
            if let Some(layout) = output.layout() {
                match with_handles!([(layout: {layout})] => {
                    layout.remove(output)
                }) {
                    Ok(_) | Err(HandleErr::AlreadyDropped) => {},
                    Err(HandleErr::AlreadyBorrowed) => {
                        panic!("Tried to remove layout from output, but the output layout is already borrowed!");
                    }
                }
            }
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.frame_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.mode_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.enable_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.scale_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.transform_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.swap_buffers_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.need_swap_listener()).link as *mut _ as _);
        let output_data = (*output_ptr).data as *mut OutputState;
        if let Some(output_ptr) = (*output_data).output {
            Box::from_raw(output_ptr.as_ptr());
        }
    };
    frame_listener => frame_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_frame(compositor, output.weak_reference());
    };
    mode_listener => mode_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_mode_change(compositor, output.weak_reference());
    };
    enable_listener => enable_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_enable(compositor, output.weak_reference());
    };
    scale_listener => scale_notify: |this: &mut UserOutput, _output: *mut libc::c_void,| unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_scale_change(compositor, output.weak_reference());
    };
    transform_listener => transform_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_transform(compositor, output.weak_reference());
    };
    swap_buffers_listener => swap_buffers_notify: |this: &mut UserOutput,
                                                   _output: *mut libc::c_void,|
    unsafe {

        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.on_buffers_swapped(compositor, output.weak_reference());
    };
    need_swap_listener => need_swap_notify: |this: &mut UserOutput, _output: *mut libc::c_void,|
    unsafe {
        let (ref output, ref mut manager) = this.data;
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        manager.needs_swap(compositor, output.weak_reference());
    };
]);
