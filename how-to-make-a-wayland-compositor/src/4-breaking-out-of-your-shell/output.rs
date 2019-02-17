use wlroots::{wlroots_dehandle, compositor, output};

use crate::CompositorState;

pub struct LayoutHandler;

impl output::layout::Handler for LayoutHandler {}

struct OutputHandler;

impl output::Handler for OutputHandler {
    #[wlroots_dehandle]
    fn on_frame(&mut self,
                compositor_handle: compositor::Handle,
                output_handle: output::Handle) {
        #[dehandle] let compositor = compositor_handle;
        #[dehandle] let output = output_handle;
        let renderer = compositor.renderer.as_mut().unwrap();
        let mut render_context = renderer.render(output, None);
        render_context.clear([0.25, 0.25, 0.25, 1.0]);
    }
}
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
