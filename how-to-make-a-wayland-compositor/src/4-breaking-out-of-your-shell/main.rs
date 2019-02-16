extern crate wlroots;

mod keyboard;
mod output;
mod pointer;
mod seat;

use wlroots::{compositor,
              utils::log::{WLR_DEBUG, init_logging},
              wlroots_dehandle};

use crate::{pointer::pointer_added,
            keyboard::keyboard_added,
            output::output_added};

pub struct CompositorState {
    xcursor_manager: wlroots::cursor::xcursor::Manager,
    seat_handle: wlroots::seat::Handle,
    cursor_handle: wlroots::cursor::Handle,
    output_layout_handle: wlroots::output::layout::Handle
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let compositor_state = setup_compositor_state();
    let output_builder = wlroots::output::manager::Builder::default()
        .output_added(output_added);
    let input_builder = wlroots::input::manager::Builder::default()
        .pointer_added(pointer_added)
        .keyboard_added(keyboard_added);
    let mut compositor = compositor::Builder::new()
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(compositor_state);
    setup_seat(&mut compositor);
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
                      output_layout_handle }
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
