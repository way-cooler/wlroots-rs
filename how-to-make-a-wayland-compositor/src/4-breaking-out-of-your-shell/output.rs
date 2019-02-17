use wlroots::{wlroots_dehandle,
              area::{Area, Origin, Size},
              compositor,
              render::{matrix, Renderer},
              output,
              utils::current_time};

use crate::{CompositorState, Shells};

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
        let state: &mut CompositorState = compositor.data
            .downcast_mut().unwrap();
        let renderer = compositor.renderer.as_mut().unwrap();
        let mut render_context = renderer.render(output, None);
        render_context.clear([0.0, 0.0, 0.0, 1.0]);
        render_shells(state, &mut render_context)
    }

    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 output_handle: output::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { ref mut outputs, .. } = compositor.downcast();
        // NOTE Not necessary to remove the output from the layout,
        // wlroots-rs takes care of it for you.
        outputs.remove(&output_handle);
    }
}

/// Render the shells in the current compositor state on the output attached
/// to the `Renderer`.
#[wlroots_dehandle]
fn render_shells(state: &mut CompositorState, renderer: &mut Renderer) {
    let CompositorState { ref output_layout_handle,
                          shells: Shells { ref xdg_shells }, .. } = state;
    for shell in xdg_shells {
        #[dehandle] let shell = &shell;
        #[dehandle] let surface = shell.surface();
        #[dehandle] let layout = output_layout_handle;
        let (width, height) = surface.current_state().size();
        // The size of the surface depends on the output scale.
        let output_scale = renderer.output.scale() as i32;
        let (render_width, render_height) = (width * output_scale,
                                             height * output_scale);
        let (ox, oy) = match layout.get_output_info(renderer.output) {
            Some(output_layout) => {
                let (mut ox, mut oy) = output_layout.coords();
                ox *= output_scale;
                oy *= output_scale;
                (ox, oy)
            }
            None => return
        };
        let render_area = Area::new(Origin::new(ox as i32, oy as i32),
                                    Size::new(render_width, render_height));
        // Only render the view if it is in the output area.
        if layout.intersects(renderer.output, render_area) {
            let transform = renderer.output.get_transform().invert();
            let matrix = matrix::project_box(render_area,
                                             transform,
                                             0.0,
                                             renderer.output
                                             .transform_matrix());
            if let Some(texture) = surface.texture().as_ref() {
                renderer.render_texture_with_matrix(texture, matrix);
            }
            surface.send_frame_done(current_time());
        }
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
