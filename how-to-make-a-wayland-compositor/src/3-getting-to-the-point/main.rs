extern crate wlroots;

mod keyboard;
mod pointer;
mod output;

use pointer::{pointer_added, init_cursor};
use keyboard::keyboard_added;
use output::{output_added, create_output_layout};

use wlroots::{compositor,
              cursor::{self, xcursor},
              utils::log::{WLR_DEBUG, init_logging}};

pub struct CompositorState {
    xcursor_manager: xcursor::Manager,
    cursor_handle: cursor::Handle,
    output_layout: wlroots::output::layout::Handle
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let (xcursor_manager, cursor_handle) = init_cursor();
    let output_builder = wlroots::output::manager::Builder::default()
        .output_added(output_added);
    let output_layout = create_output_layout();
    let input_builder = wlroots::input::manager::Builder::default()
        .pointer_added(pointer_added)
        .keyboard_added(keyboard_added);
    let compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(CompositorState { xcursor_manager,
                                      cursor_handle,
                                      output_layout }
        );
    compositor.run();
}
