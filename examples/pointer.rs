#[macro_use]
extern crate wlroots;
extern crate log;

use log::LevelFilter;
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
    compositor,
    cursor::{self, xcursor, Cursor},
    input::{self, keyboard, pointer},
    output,
    utils::log::Logger
};

const MOUSE_AXIS_STEP_DIFF: f32 = 0.05;

struct CompositorState {
    color: [f32; 4],
    default_color: [f32; 4],
    xcursor_manager: xcursor::Manager,
    layout_handle: output::layout::Handle,
    cursor_handle: cursor::Handle
}
impl CompositorState {
    fn new(
        xcursor_manager: xcursor::Manager,
        layout_handle: output::layout::Handle,
        cursor_handle: cursor::Handle
    ) -> Self {
        CompositorState {
            color: [0.25, 0.25, 0.25, 1.0],
            default_color: [0.25, 0.25, 0.25, 1.0],
            xcursor_manager,
            layout_handle,
            cursor_handle
        }
    }
}

struct ExCursor;
impl cursor::Handler for ExCursor {}

struct ExOutputLayout;
impl output::layout::Handler for ExOutputLayout {}

fn output_added(
    compositor_handle: compositor::Handle,
    output_builder: output::Builder
) -> Option<output::BuilderResult> {
    let mut result = output_builder.build_best_mode(ExOutput);
    with_handles!([(compositor: {compositor_handle})] => {
        let compositor_state: &mut CompositorState = compositor.data.downcast_mut().unwrap();
        let layout_handle = &mut compositor_state.layout_handle;
        let cursor_handle = &mut compositor_state.cursor_handle;
        let xcursor_manager = &mut compositor_state.xcursor_manager;
        with_handles!([(layout: {layout_handle}),
                        (cursor: {cursor_handle}),
                        (output: {&mut result.output})] => {
            layout.add_auto(output);
            cursor.attach_output_layout(layout);
            xcursor_manager.load(output.scale());
            xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
            let (x, y) = cursor.coords();
            // https://en.wikipedia.org/wiki/Mouse_warping
            cursor.warp(None, x, y);
        }).unwrap();
        Some(result)
    })
    .unwrap()
}

struct ExKeyboardHandler;
impl keyboard::Handler for ExKeyboardHandler {
    fn on_key(
        &mut self,
        _compositor_handle: compositor::Handle,
        _keyboard_handle: keyboard::Handle,
        key_event: &keyboard::event::Key
    ) {
        for key in key_event.pressed_keys() {
            if let keysyms::KEY_Escape = key {
                wlroots::compositor::terminate()
            }
        }
    }
}

struct ExPointer;
impl pointer::Handler for ExPointer {
    fn on_motion_absolute(
        &mut self,
        compositor_handle: compositor::Handle,
        _pointer_handle: pointer::Handle,
        absolute_motion_event: &pointer::event::AbsoluteMotion
    ) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.downcast();
            let (x, y) = absolute_motion_event.pos();
            compositor_state.cursor_handle
                .run(|cursor| cursor.warp_absolute(absolute_motion_event.device(), x, y))
                .unwrap();
        })
        .unwrap();
    }

    fn on_motion(
        &mut self,
        compositor_handle: compositor::Handle,
        _pointer_handle: pointer::Handle,
        motion_event: &pointer::event::Motion
    ) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.downcast();
            let (delta_x, delta_y) = motion_event.delta();
            compositor_state.cursor_handle
                .run(|cursor| cursor.move_relative(None, delta_x, delta_y))
                .unwrap();
        })
        .unwrap();
    }

    fn on_button(
        &mut self,
        compositor_handle: compositor::Handle,
        _pointer_handle: pointer::Handle,
        button_event: &pointer::event::Button
    ) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.downcast();
            compositor_state.color =
                if button_event.state() == WLR_BUTTON_RELEASED {
                    compositor_state.default_color
                } else {
                    let mut mouse_button_color = [0.25, 0.25, 0.25, 1.0];
                    mouse_button_color[button_event.button() as usize % 3] = 1.0;
                    mouse_button_color
                };
        })
        .unwrap();
    }

    fn on_axis(
        &mut self,
        compositor_handle: compositor::Handle,
        _pointer_handle: pointer::Handle,
        axis_event: &pointer::event::Axis
    ) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.downcast();
            let color_diff = if axis_event.delta() > 0.0 {
                -MOUSE_AXIS_STEP_DIFF
            } else {
                MOUSE_AXIS_STEP_DIFF
            };
            for color_byte in &mut compositor_state.default_color[..3] {
                *color_byte += color_diff;
                if *color_byte > 1.0 {
                    *color_byte = 1.0;
                }
                if *color_byte < 0.0 {
                    *color_byte = 0.0;
                }
            }
            compositor_state.color = compositor_state.default_color
        })
        .unwrap();
    }
}

struct ExOutput;
impl output::Handler for ExOutput {
    fn on_frame(&mut self, compositor_handle: compositor::Handle, output_handle: output::Handle) {
        with_handles!([(compositor: {compositor_handle}), (output: {output_handle})] => {
            let compositor_state: &mut CompositorState = compositor.data.downcast_mut().unwrap();
            let renderer = compositor.renderer.as_mut()
                .expect("Compositor was not loaded with a renderer");
            let mut render_context = renderer.render(output, None);
            render_context.clear(compositor_state.color);
        })
        .unwrap();
    }
}

fn pointer_added(
    compositor_handle: compositor::Handle,
    pointer_handle: pointer::Handle
) -> Option<Box<dyn pointer::Handler>> {
    with_handles!([(compositor: {compositor_handle}), (pointer: {pointer_handle})] => {
        let compositor_state: &mut CompositorState = compositor.downcast();
        compositor_state.cursor_handle
            .run(|cursor| cursor.attach_input_device(pointer.input_device()))
            .unwrap();
    })
    .unwrap();
    Some(Box::new(ExPointer))
}

fn keyboard_added(
    _compositor_handle: compositor::Handle,
    _keyboard_handle: keyboard::Handle
) -> Option<Box<dyn keyboard::Handler>> {
    Some(Box::new(ExKeyboardHandler))
}

fn load_xcursor() -> (xcursor::Manager, cursor::Handle) {
    let cursor_handle = Cursor::create(Box::new(ExCursor));
    let mut xcursor_manager = xcursor::Manager::create("default".to_string(), 24).expect(
        "Could not create xcursor \
         manager"
    );
    xcursor_manager.load(1.0);
    cursor_handle
        .run(|c| xcursor_manager.set_cursor_image("left_ptr".to_string(), c))
        .unwrap();
    (xcursor_manager, cursor_handle)
}

fn main() {
    Logger::init(LevelFilter::Debug, None);
    let (xcursor_manager, cursor_handle) = load_xcursor();
    let layout_handle = output::layout::Layout::create(Box::new(ExOutputLayout));

    let output_builder = output::manager::Builder::default().output_added(output_added);
    let input_builder = input::manager::Builder::default()
        .pointer_added(pointer_added)
        .keyboard_added(keyboard_added);
    let compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(CompositorState::new(
            xcursor_manager,
            layout_handle,
            cursor_handle
        ));
    compositor.run();
}
