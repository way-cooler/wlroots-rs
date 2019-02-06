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
    let CompositorState { ref output_layout_handle, .. } = compositor.downcast();
    #[dehandle] let output = result.output.clone();
    #[dehandle] let output_layout = output_layout_handle;
    output_layout.add_auto(output);
    Some(result)
}
