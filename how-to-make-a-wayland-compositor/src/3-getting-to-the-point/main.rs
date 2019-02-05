extern crate wlroots;

mod keyboard;
mod pointer;
mod output;

use pointer::pointer_added;
use keyboard::keyboard_added;
use output::output_added;

use wlroots::{compositor,
              cursor::xcursor,
              utils::log::{WLR_DEBUG, init_logging}};

pub struct CompositorState {
    theme: xcursor::Theme,
    cursor: Option<wlroots::output::Cursor>
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let theme = xcursor::Theme::load_theme(None, 16)
        .expect("Could not create xcursor manager");
    let output_builder = wlroots::output::manager::Builder::default()
        .output_added(output_added);
    let input_builder = wlroots::input::manager::Builder::default()
        .pointer_added(pointer_added)
        .keyboard_added(keyboard_added);
    let compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(CompositorState { theme, cursor: None });
    compositor.run();
}
