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

    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 output_handle: output::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { ref mut outputs,
                              ref output_layout_handle,
                              .. } = compositor.downcast();
        #[dehandle] let layout = output_layout_handle;
        #[dehandle] let output = output_handle.clone();
        // NOTE Not necessary to remove the output from the layout,
        // wlroots-rs takes care of it for you.
        outputs.remove(&output_handle);
    }
}
#[wlroots_dehandle]
pub fn output_added<'output>(compositor: compositor::Handle,
                             builder: output::Builder<'output>)
                             -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(OutputHandler);
    #[dehandle] let compositor = compositor;
    let CompositorState { ref output_layout_handle,
                          ref mut outputs,
                          ref cursor_handle,
                          ref mut xcursor_manager,
                          .. } = compositor.downcast();
    #[dehandle] let output = result.output.clone();
    #[dehandle] let cursor = cursor_handle;
    #[dehandle] let layout = output_layout_handle;
    layout.add_auto(output);
    // NOTE You _must_ attach the cursor to the layout before
    // doing xcursor related with it. Otherwise if you hotplug outputs
    // then the cursor will stop rendering correctly.
    cursor.attach_output_layout(layout);
    xcursor_manager.load(output.scale());
    xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
    outputs.insert(result.output.clone());
    let (x, y) = cursor.coords();
    cursor.warp(None, x, y);
    Some(result)
}
