extern crate log;
#[macro_use]
extern crate wlroots;

use std::{env, time::Instant};

use log::LevelFilter;

use wlroots::wlroots_sys::wl_output_transform;
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
    compositor,
    input::{self, keyboard},
    output,
    render::{Texture, TextureFormat},
    utils::log::Logger
};

const CAT_TEXTURE_WIDTH: u32 = 128;
const CAT_TEXTURE_HEIGHT: u32 = 128;
const CAT_TEXTURE_DATA: &[u8] = include_bytes!("cat.data");
const VELOCITY_STEP_DIFF: f32 = 16.0;

struct Vector2 {
    x: f32,
    y: f32
}
impl Vector2 {
    pub fn increment(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;
    }
}

struct CompositorState {
    cat_texture: Option<Texture<'static>>,
    rotation_transform: wl_output_transform,
    last_frame: Instant,
    offset: Vector2,
    velocity: Vector2
}
impl CompositorState {
    fn new(rotation_transform: wl_output_transform) -> Self {
        CompositorState {
            cat_texture: None,
            rotation_transform,
            last_frame: Instant::now(),
            offset: Vector2 { x: 0.0, y: 0.0 },
            velocity: Vector2 { x: 128.0, y: 128.0 }
        }
    }

    /// Registers `now` as the last frame and returns the calculated delta time
    /// since the previous last frame in seconds.
    pub fn register_frame(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        let seconds_delta = delta.as_secs() as f32;
        let nano_delta = u64::from(delta.subsec_nanos());
        let ms = (seconds_delta * 1000.0) + nano_delta as f32 / 1_000_000.0;
        ms / 1000.0
    }
}

fn output_added(
    compositor_handle: compositor::Handle,
    output_builder: output::Builder
) -> Option<output::BuilderResult> {
    let ex_output = ExOutput;
    let mut result = output_builder.build_best_mode(ex_output);
    with_handles!([(compositor: {compositor_handle}), (output: {&mut result.output})] => {
        let compositor_state: &mut CompositorState = compositor.downcast();
        output.transform(compositor_state.rotation_transform);
    })
    .unwrap();
    Some(result)
}

struct ExOutput;
impl output::Handler for ExOutput {
    fn on_frame(&mut self, mut compositor_handle: compositor::Handle, mut output_handle: output::Handle) {
        with_handles!([(compositor: {&mut compositor_handle}), (output: {&mut output_handle})] => {
            let (output_width, output_height) = output.effective_resolution();
            let renderer = compositor.renderer
                                    .as_mut()
                                    .expect("Compositor was not loaded with gles2 renderer");
            let compositor_state: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
            let delta_time_in_seconds = compositor_state.register_frame();
            let &mut CompositorState { ref mut offset, ref cat_texture, .. } = compositor_state;
            let transform_matrix = output.transform_matrix();
            let mut renderer = renderer.render(output, None);
            let cat_texture = cat_texture.as_ref().unwrap();
            let (max_width, max_height) = (CAT_TEXTURE_WIDTH as i32, CAT_TEXTURE_HEIGHT as i32);
            for y in (-max_height + offset.y as i32..output_height).step_by(max_height as usize) {
                for x in (-max_width + offset.x as i32..output_width).step_by(max_width as usize) {
                    renderer.render_texture(&cat_texture, transform_matrix, x, y, 1.0);
                }
            }
            offset.increment(
                compositor_state.velocity.x * delta_time_in_seconds,
                compositor_state.velocity.y * delta_time_in_seconds
            );
            if offset.x > max_width as f32 {
                offset.x = 0.0
            }
            if offset.y > max_height as f32 {
                offset.y = 0.0
            }
        })
        .unwrap();
    }
}

fn keyboard_added(
    _compositor_handle: compositor::Handle,
    _keyboard_handle: keyboard::Handle
) -> Option<Box<dyn keyboard::Handler>> {
    Some(Box::new(KeyboardManager))
}

struct KeyboardManager;
impl keyboard::Handler for KeyboardManager {
    fn on_key(
        &mut self,
        compositor_handle: compositor::Handle,
        _keyboard_handle: keyboard::Handle,
        key_event: &keyboard::event::Key
    ) {
        with_handles!([(compositor: {compositor_handle})] => {
            let compositor_state: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
            for key in key_event.pressed_keys() {
                match key {
                    keysyms::KEY_Escape => compositor::terminate(),
                    keysyms::KEY_Left => compositor_state.velocity.increment(-VELOCITY_STEP_DIFF, 0.0),
                    keysyms::KEY_Right => compositor_state.velocity.increment(VELOCITY_STEP_DIFF, 0.0),
                    keysyms::KEY_Up => compositor_state.velocity.increment(0.0, -VELOCITY_STEP_DIFF),
                    keysyms::KEY_Down => compositor_state.velocity.increment(0.0, VELOCITY_STEP_DIFF),
                    _ => {}
                }
            }
        })
        .unwrap();
    }
}

fn rotation_transform_from_str(rotation_str: &str) -> wl_output_transform {
    use crate::wl_output_transform::*;
    match rotation_str {
        "90" => WL_OUTPUT_TRANSFORM_90,
        "180" => WL_OUTPUT_TRANSFORM_180,
        "270" => WL_OUTPUT_TRANSFORM_270,
        "flipped" => WL_OUTPUT_TRANSFORM_FLIPPED,
        "flipped_90" => WL_OUTPUT_TRANSFORM_FLIPPED_90,
        "flipped_180" => WL_OUTPUT_TRANSFORM_FLIPPED_180,
        "flipped_270" => WL_OUTPUT_TRANSFORM_FLIPPED_270,
        _ => WL_OUTPUT_TRANSFORM_NORMAL
    }
}

fn main() {
    Logger::init(LevelFilter::Debug, None);
    let mut args = env::args();
    let rotation_argument_string = args.nth(1).unwrap_or_else(|| "".to_string());
    let rotation_transform = rotation_transform_from_str(&rotation_argument_string);
    let compositor_state = CompositorState::new(rotation_transform);
    let output_builder = output::manager::Builder::default().output_added(output_added);
    let input_builder = input::manager::Builder::default().keyboard_added(keyboard_added);
    let mut compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(compositor_state);
    {
        let gles2 = &mut compositor.renderer.as_mut().unwrap();
        let compositor_state: &mut CompositorState = (&mut compositor.data).downcast_mut().unwrap();
        compositor_state.cat_texture = gles2.create_texture_from_pixels(
            TextureFormat::ABGR8888.into(),
            CAT_TEXTURE_WIDTH * 4,
            CAT_TEXTURE_WIDTH,
            CAT_TEXTURE_HEIGHT,
            CAT_TEXTURE_DATA
        );
    }
    compositor.run();
}
