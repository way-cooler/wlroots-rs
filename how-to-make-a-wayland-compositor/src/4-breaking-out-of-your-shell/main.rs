extern crate wlroots;

mod keyboard;
mod output;
mod pointer;
mod seat;
mod xdg_shell;

use std::{collections::HashSet, env, process::{Command, Stdio}};

use wlroots::{compositor,
              utils::log::{WLR_DEBUG, init_logging},
              wlroots_dehandle};

use crate::{pointer::pointer_added,
            keyboard::keyboard_added,
            output::output_added};

#[derive(Default)]
pub struct Shells {
    xdg_shells: HashSet<wlroots::shell::xdg_shell::Handle>
}

#[derive(Default)]
pub struct Inputs {
    pointers: HashSet<wlroots::input::pointer::Handle>,
    keyboards: HashSet<wlroots::input::keyboard::Handle>,
}

pub struct CompositorState {
    xcursor_manager: wlroots::cursor::xcursor::Manager,
    seat_handle: wlroots::seat::Handle,
    cursor_handle: wlroots::cursor::Handle,
    output_layout_handle: wlroots::output::layout::Handle,
    outputs: HashSet<wlroots::output::Handle>,
    shells: Shells,
    inputs: Inputs
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let compositor_state = setup_compositor_state();
    let output_builder = wlroots::output::manager::Builder::default()
        .output_added(output_added);
    let input_builder = wlroots::input::manager::Builder::default()
        .pointer_added(pointer_added)
        .keyboard_added(keyboard_added);
    let xdg_shell_builder = wlroots::shell::xdg_shell::manager::Builder::default()
        .surface_added(xdg_shell::new_surface);
    let mut compositor = compositor::Builder::new()
        .gles2(true)
        .wl_shm(true)
        .data_device(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .xdg_shell_manager(xdg_shell_builder)
        .build_auto(compositor_state);
    setup_seat(&mut compositor);
    spawn_startup_command(&compositor);
    compositor.run();
}

#[wlroots_dehandle]
fn setup_compositor_state() -> CompositorState {
    use wlroots::{cursor::{Cursor, xcursor},
                  seat,
                  output::layout::Layout};
    use crate::{pointer::CursorHandler, output::LayoutHandler};
    let output_layout_handle = Layout::create(Box::new(LayoutHandler));
    // Make a sentinel seat to be filled in after the compositor is created.
    let seat_handle = seat::Handle::new();
    let cursor_handle = Cursor::create(Box::new(CursorHandler));
    let xcursor_manager = xcursor::Manager::create("default".to_string(), 24)
        .expect("Could not create xcursor manager");
    xcursor_manager.load(1.0);
    #[dehandle] let output_layout = output_layout_handle.clone();
    #[dehandle] let cursor = cursor_handle.clone();
    cursor.attach_output_layout(output_layout);
    CompositorState { xcursor_manager,
                      cursor_handle,
                      seat_handle,
                      output_layout_handle,
                      shells: Shells::default(),
                      outputs: HashSet::default(),
                      inputs: Inputs::default() }
}

/// Set up the seat for the compositor.
/// Needs to be done after the compositor is already created.
fn setup_seat(compositor: &mut compositor::Compositor) {
    use wlroots::seat::Seat;
    let seat_handle = Seat::create(compositor,
                                   "default".into(),
                                   Box::new(seat::SeatHandler));
    let state: &mut CompositorState = compositor.data.downcast_mut().unwrap();
    state.seat_handle = seat_handle;
}

/// Spawns a startup command, if one was provided on the command line.
fn spawn_startup_command(compositor: &compositor::Compositor) {
    let args = env::args().skip(1).collect::<Vec<_>>();
    env::set_var("WAYLAND_DISPLAY", compositor.socket_name());
    if args.len() > 0 {
        let command = args.join(" ");
        Command::new("/bin/sh")
            .arg("-c")
            .arg(command.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect(&format!("Could not spawn {}", command));
    }
}
