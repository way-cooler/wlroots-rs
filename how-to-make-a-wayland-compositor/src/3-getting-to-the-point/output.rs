use wlroots::{wlroots_dehandle, compositor, output};

use CompositorState;

struct OutputHandler;

struct LayoutHandler;

#[wlroots_dehandle]
pub fn output_added<'output>(compositor: compositor::Handle,
                             builder: output::Builder<'output>)
                             -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(OutputHandler);
    {
        #[dehandle] let compositor = compositor;
        #[dehandle] let output = &result.output;
        let &mut CompositorState { output_layout: ref output_layout_handle,
                                   ref cursor_handle,
                                   ref mut xcursor_manager } =
            compositor.data.downcast_mut().unwrap();
        #[dehandle] let output_layout = output_layout_handle;
        #[dehandle] let cursor = cursor_handle;
        output_layout.add_auto(output);
        cursor.attach_output_layout(output_layout);
        xcursor_manager.load(1.0);
        xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
        let (x, y) = cursor.coords();
        cursor.warp(None, x, y);
    }
    Some(result)
}

pub fn create_output_layout() -> output::layout::Handle {
    wlroots::output::layout::Layout::create(Box::new(LayoutHandler))
}

impl output::Handler for OutputHandler {}

impl output::layout::Handler for LayoutHandler {}
