extern crate wlroots_sys;
#[macro_use] extern crate wayland_sys;
extern crate wayland_server;

use std::ptr::null_mut;
use std::env;
use std::time::Instant;
use std::os::raw::{c_void, c_int};

use wlroots_sys::{wl_list, wlr_session_start, wlr_output, wlr_output_enable,
                  wlr_output_set_mode, wlr_backend_init, wlr_backend_destroy,
                  wl_signal, wlr_backend_autocreate};
use wayland_sys::server::{WAYLAND_SERVER_HANDLE, wl_listener,
                          wl_notify_func_t};

// For graphical functions
// TODO Move into real library
mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

#[repr(C)]
struct wl_listener_wrapper {
    link: wl_list,
    notify: wl_notify_func_t
}

/// State of the example program.
struct State {
    /// The color on the screen.
    color: [f32; 3],
    dec: i32,
    /// How long since the last frame was renderend.
    last_frame: Instant,
    /// An opaque pointer to the wl_listener for when an output is added.
    output_add: wl_listener_wrapper,
    /// An opaque pointer to the wl_listener for when an output is removed.
    output_remove: wl_listener_wrapper,
    /// List of outputs
    outputs: Vec<OutputState>
}

#[repr(C)]
struct OutputState {
    output: *mut wlr_output,
    state: *mut State,
    frame: wl_listener_wrapper
}

macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}


macro_rules! wl_container_of {
    ($ptr:ident, $ty:ty, $field:ident) => {
        ($ptr as usize - offset_of!($ty, $field)) as *const $ty
    }
}

unsafe extern "C" fn timer_done(data: *mut c_void) -> c_int {
    let done: &mut bool = &mut *(data as *mut _);
    *done = true;
    1
}

unsafe extern "C" fn enable_outputs(data: *mut c_void) -> c_int {
    let state: &mut State = &mut *(data as *mut _);
    for output_state in &mut state.outputs {
        wlr_output_enable(output_state.output, true);
    }
    1
}
unsafe extern "C" fn disable_outputs(data: *mut c_void) -> c_int {
    let state: &mut State = &mut *(data as *mut _);
    for output_state in &mut state.outputs {
        wlr_output_enable(output_state.output, false);
    }
    1
}

unsafe extern "C" fn output_frame(listener: *mut wl_listener,
                                  _data: *mut c_void) {
    let output_state: *mut OutputState =
        wl_container_of!(listener, OutputState, frame) as *mut _;
    let state: &mut State = &mut *(*output_state).state;
    let now = Instant::now();
    let delta = now.duration_since(state.last_frame);
    //let delta = state.last_frame.duration_since(now);
    let seconds_delta= delta.as_secs();
    let nano_delta = delta.subsec_nanos() as u64;
    let ms = (seconds_delta * 1000) + nano_delta / 1000000;
    let inc = (state.dec + 1) % 3;
    state.color[inc as usize] += ms as f32 / 2000.0;
    state.color[state.dec as usize] -= ms as f32 / 2000.0;

    if state.color[state.dec as usize] < 0.0 {
        state.color[inc as usize] = 1.0;
        state.color[state.dec as usize] = 0.0;
        state.dec = inc;
    }

    state.last_frame = now;
    gl::ClearColor(state.color[0], state.color[1], state.color[2], 1.0);
    gl::Clear(gl::COLOR_BUFFER_BIT);
}

unsafe extern "C" fn output_add(listener: *mut wl_listener, data: *mut c_void) {
    let output: &mut wlr_output = &mut *(data as *mut _);
    let state: *mut State = wl_container_of!(listener, State, output_add) as *mut _;
    println!("Adding output");
    let cur_mode = (*(*output.modes).items) as *mut _;
    wlr_output_set_mode(output, cur_mode);
    let new_output_state = OutputState {
        output,
        state,
        frame: wl_listener_wrapper {
            link: wl_list {
                prev: null_mut(),
                next: null_mut()
            },
            notify: output_frame
        }
    };
    (*state).outputs.push(new_output_state);
    let mut new_output_state = &mut (*state).outputs.last_mut().unwrap();
    wl_signal_add(&mut output.events.frame,
                  &mut new_output_state.frame);
}

unsafe extern "C" fn output_remove(_listener: *mut wl_listener,
                                   _data: *mut c_void) {
    println!("Removing output");
    // RAII will take care of this for the example.
    // However, in a real application it should search through the list and remove it.
    // To avoid moving the data, a linked list should be used, but again for the example
    // it doesn't really matter which we use.
}

fn main() {
    if env::var("DISPLAY").is_ok() {
        panic!("Detected that X is running. Run this in its own virtual terminal.")
    } else if env::var("WAYLAND_DISPLAY").is_ok() {
        panic!("Detected that Wayland is running. Run this in its own virtual terminal")
    }
    let mut state = State {
        color: [1.0, 0.0, 0.0],
        dec: 0,
        last_frame: Instant::now(),
        output_add: wl_listener_wrapper {
            link: wl_list {
                prev: null_mut(),
                next: null_mut()
            },
            notify: output_add
        },
        output_remove: wl_listener_wrapper {
            link: wl_list {
                prev: null_mut(),
                next: null_mut()
            },
            notify: output_remove
        },
        // High capacity just so I can avoid reallocations
        // while I have random pointers to the contents...
        outputs: Vec::with_capacity(128)
    };
    unsafe {
        let display = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
            wl_display_create,
        );
        let wlr_display = display as *mut wlroots_sys::wl_display;
        let event_loop = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
            wl_display_get_event_loop,
            display
        );
        let wlr_session = wlr_session_start(wlr_display);
        if wlr_session.is_null() {
            panic!("Could not initialize wlr session!")
        }
	      let wlr_backend = wlr_backend_autocreate(wlr_display, wlr_session);
        if wlr_backend.is_null() {
            panic!("wlr_backend_autocreate returned null ptr!");
        }
        wl_signal_add(&mut (*wlr_backend).events.output_add,
                      &mut state.output_add);
        wl_signal_add(&mut (*wlr_backend).events.output_remove,
                      &mut state.output_remove);
        if !wlr_backend_init(wlr_backend) {
            panic!("Failed to initialize wlr output backend");
        }

        let mut done = false;
        let timer = ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                  wl_event_loop_add_timer,
                                  event_loop,
                                  timer_done,
                                  &mut done as *mut _ as *mut _);
        let timer_disable_outputs =
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_event_loop_add_timer,
                          event_loop,
                          disable_outputs,
                          &mut state as *mut _ as *mut _);
        let timer_enable_outputs =
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_event_loop_add_timer,
                          event_loop,
                          enable_outputs,
                          &mut state as *mut _ as *mut _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_event_source_timer_update,
                      timer,
                      20000);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_event_source_timer_update,
                      timer_disable_outputs,
                      5000);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_event_source_timer_update,
                      timer_enable_outputs,
                      10000);
        while !done {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                          wl_event_loop_dispatch,
                          event_loop,
                          0);
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_event_source_remove,
                      timer);
        wlr_backend_destroy(wlr_backend);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_display_destroy,
                      display);
    }
}


/// NOTE This is what wl_signal_add is suppose to be
/// but it's not defined in wayland-rs for some reason..
unsafe fn wl_signal_add(signal: &mut wl_signal, listener: &mut wl_listener_wrapper) {
    ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                  wl_list_insert,
                  signal.listener_list.prev as *mut _,
                  &mut listener.link as *mut _ as *mut _
    )
}
