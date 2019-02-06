use wlroots::{wlroots_dehandle, compositor, output};

use crate::CompositorState;

struct OutputHandler;

impl output::Handler for OutputHandler {}

pub struct LayoutHandler;

impl output::layout::Handler for LayoutHandler {}

#[wlroots_dehandle]
pub fn output_added<'output>(compositor: compositor::Handle,
                             builder: output::Builder<'output>)
                             -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(OutputHandler);
    #[dehandle] let compositor = compositor;
    #[dehandle] let output = result.output.clone();
    let CompositorState { ref output_layout_handle,
                            ref cursor_handle,
                            ref mut xcursor_manager } =
        compositor.downcast();
    #[dehandle] let output_layout = output_layout_handle;
    #[dehandle] let cursor = cursor_handle;
    output_layout.add_auto(output);
    xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
    let (x, y) = cursor.coords();
    cursor.warp(None, x, y);
    output.schedule_frame();
    Some(result)
}
