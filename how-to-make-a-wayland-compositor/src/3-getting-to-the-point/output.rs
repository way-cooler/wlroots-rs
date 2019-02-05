use wlroots::{wlroots_dehandle, compositor, output};

use CompositorState;

struct OutputHandler;

#[wlroots_dehandle]
pub fn output_added<'output>(compositor: compositor::Handle,
                             builder: output::Builder<'output>)
                             -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(OutputHandler);
    {
        #[dehandle] let compositor = compositor;
        #[dehandle] let output = &result.output;
        let CompositorState { ref mut theme, ref mut cursor } =
            compositor.downcast();
        *cursor = output::Cursor::new(output).map(|mut cursor| {
            let xcursor = theme.get_cursor("left_ptr".into())
                .expect("Could not load default cursor set");
            let image: wlroots::render::Image = xcursor.image(0).expect("xcursor had no images").into();
            cursor.set_image(&image)
                .expect("Could not set cursor image");
            cursor
        });
    }
    Some(result)
}

impl output::Handler for OutputHandler {}
