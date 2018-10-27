#[macro_use]
extern crate wlroots;
extern crate libc;

use wlroots::{CompositorBuilder, CompositorHandle, Cursor, CursorHandle, CursorHandler,
              InputManagerHandler, KeyboardHandle, KeyboardHandler, OutputBuilder,
              OutputBuilderResult, OutputHandle, OutputHandler, OutputLayout, OutputLayoutHandle,
              OutputLayoutHandler, OutputManagerHandler, PointerHandle, PointerHandler,
              XCursorManager};
use wlroots::key_events::KeyEvent;
use wlroots::pointer_events::{AbsoluteMotionEvent, AxisEvent, ButtonEvent, MotionEvent};
use wlroots::utils::{init_logging, WLR_DEBUG};
use wlroots::wlroots_sys::wlr_button_state::WLR_BUTTON_RELEASED;
use wlroots::xkbcommon::xkb::keysyms;

const MOUSE_AXIS_DELTA_STEP: f32 = 0.05;

struct CompositorState {
    color: [f32; 4],
    default_color: [f32; 4],
    xcursor_manager: XCursorManager,
    layout_handle: OutputLayoutHandle,
    cursor_handle: CursorHandle,
}
impl CompositorState {
    fn new(xcursor_manager: XCursorManager,
           layout_handle: OutputLayoutHandle,
           cursor_handle: CursorHandle)
           -> Self {
        CompositorState { color: [0.25, 0.25, 0.25, 1.0],
                          default_color: [0.25, 0.25, 0.25, 1.0],
                          xcursor_manager,
                          layout_handle,
                          cursor_handle,
        }
    }
}

compositor_data!(CompositorState);

struct ExCursor;
impl CursorHandler for ExCursor {}

struct ExOutputLayout;
impl OutputLayoutHandler for ExOutputLayout {}

struct OutputManager;
impl OutputManagerHandler for OutputManager {
    fn output_added<'output>(&mut self,
                             compositor_handle: CompositorHandle,
                             output_builder: OutputBuilder<'output>)
                             -> Option<OutputBuilderResult<'output>> {
        let mut result = output_builder.build_best_mode(ExOutput);
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.data.downcast_mut().unwrap();
            let layout_handle = &mut compositor_state.layout_handle;
            let cursor_handle = &mut compositor_state.cursor_handle;
            let xcursor_manager = &mut compositor_state.xcursor_manager;
            // TODO use output config if present instead of auto
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
        }).unwrap()
    }
}

struct ExKeyboardHandler;
impl KeyboardHandler for ExKeyboardHandler {
    fn on_key(&mut self, _compositor_handle: CompositorHandle, _keyboard_handle: KeyboardHandle, key_event: &KeyEvent) {
        for key in key_event.pressed_keys() {
            match key {
                keysyms::KEY_Escape => wlroots::terminate(),
                _ => {}
            }
        }
    }
}

struct ExPointer;
impl PointerHandler for ExPointer {
    fn on_motion_absolute(&mut self,
                          compositor_handle: CompositorHandle,
                          _pointer_handle: PointerHandle,
                          absolute_motion_event: &AbsoluteMotionEvent) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.into();
            let (x, y) = absolute_motion_event.pos();
            compositor_state.cursor_handle
                .run(|cursor| cursor.warp_absolute(absolute_motion_event.device(), x, y))
                .unwrap();
        }).unwrap();
    }

    fn on_motion(&mut self, compositor_handle: CompositorHandle, _pointer_handle: PointerHandle, motion_event: &MotionEvent) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.into();
            let (delta_x, delta_y) = motion_event.delta();
            compositor_state.cursor_handle
                .run(|cursor| cursor.move_to(None, delta_x, delta_y))
                .unwrap();
        }).unwrap();
    }

    fn on_button(&mut self, compositor_handle: CompositorHandle, _pointer_handle: PointerHandle, button_event: &ButtonEvent) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.into();
            compositor_state.color =
                if button_event.state() == WLR_BUTTON_RELEASED {
                    compositor_state.default_color
                } else {
                    let mut mouse_button_color = [0.25, 0.25, 0.25, 1.0];
                    mouse_button_color[button_event.button() as usize % 3] = 1.0;
                    mouse_button_color
                };
        }).unwrap();
    }

    fn on_axis(&mut self, compositor_handle: CompositorHandle, _pointer_handle: PointerHandle, axis_event: &AxisEvent) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = compositor.into();
            let color_diff = if axis_event.delta() > 0.0 { -MOUSE_AXIS_DELTA_STEP } else { MOUSE_AXIS_DELTA_STEP };
            for color_byte in &mut compositor_state.default_color[..3] {
                *color_byte += color_diff;
                if *color_byte > 1.0 {
                    *color_byte = 1.0;
                }
                if *color_byte < 0.0 {
                    *color_byte = 0.0;
                }
            }
            compositor_state.color = compositor_state.default_color.clone()
        }).unwrap();
    }
}

struct ExOutput;
impl OutputHandler for ExOutput {
    fn on_frame(&mut self, compositor_handle: CompositorHandle, output_handle: OutputHandle) {
        with_handles!([(compositor: {compositor_handle}), (output: {output_handle})] => {
            let compositor_state: &mut CompositorState = compositor.data.downcast_mut().unwrap();
            let renderer = compositor.renderer.as_mut()
                .expect("Compositor was not loaded with a renderer");
            let mut render_context = renderer.render(output, None);
            render_context.clear(compositor_state.color);
        }).unwrap();
    }
}

struct InputManager;
impl InputManagerHandler for InputManager {
    fn pointer_added(&mut self,
                     compositor_handle: CompositorHandle,
                     pointer_handle: PointerHandle)
                     -> Option<Box<PointerHandler>> {
        with_handles!([(compositor: {compositor_handle}), (pointer: {pointer_handle})] => {
            let compositor_state: &mut CompositorState = compositor.into();
            compositor_state.cursor_handle
                .run(|cursor| cursor.attach_input_device(pointer.input_device()))
                .unwrap();
        }).unwrap();
        Some(Box::new(ExPointer))
    }

    fn keyboard_added(&mut self,
                      _compositor_handle: CompositorHandle,
                      _keyboard_handle: KeyboardHandle)
                      -> Option<Box<KeyboardHandler>> {
        Some(Box::new(ExKeyboardHandler))
    }
}

fn load_xcursor() -> (XCursorManager, CursorHandle) {
    let cursor_handle = Cursor::create(Box::new(ExCursor));
    let mut xcursor_manager =
        XCursorManager::create("default".to_string(), 24).expect("Could not create xcursor \
                                                                  manager");
    xcursor_manager.load(1.0);
    cursor_handle.run(|c| xcursor_manager.set_cursor_image("left_ptr".to_string(), c))
          .unwrap();
    (xcursor_manager, cursor_handle)
}

fn main() {
    init_logging(WLR_DEBUG, None);
    let (xcursor_manager, cursor_handle) = load_xcursor();
    let layout_handle = OutputLayout::create(Box::new(ExOutputLayout));

    let compositor =
        CompositorBuilder::new().gles2(true)
                                .input_manager(Box::new(InputManager))
                                .output_manager(Box::new(OutputManager))
                                .build_auto(CompositorState::new(xcursor_manager, layout_handle, cursor_handle));
    compositor.run();
}
