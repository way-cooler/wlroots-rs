extern crate log;
#[macro_use]
extern crate wlroots;

use log::LevelFilter;
use wlroots::utils::log::Logger;
use wlroots::xkbcommon::xkb::keysyms::KEY_Escape;
use wlroots::{
    compositor,
    input::{self, keyboard, touch},
    output,
    render::{Texture, TextureFormat}
};

const CAT_WIDTH: u32 = 128;
const CAT_HEIGHT: u32 = 128;
const CAT_DATA: &[u8] = include_bytes!("cat.data");

#[derive(Debug, Clone)]
struct TouchPoint {
    touch_id: i32,
    x: f64,
    y: f64
}

struct State {
    cat_texture: Option<Texture<'static>>,
    touch_points: Vec<TouchPoint>
}

impl State {
    fn new() -> Self {
        State {
            cat_texture: None,
            touch_points: Vec::new()
        }
    }
}

struct TouchHandlerEx;

struct ExOutput;

struct ExKeyboardHandler;

fn output_added(_: compositor::Handle, builder: output::Builder) -> Option<output::BuilderResult> {
    Some(builder.build_best_mode(ExOutput))
}

impl keyboard::Handler for ExKeyboardHandler {
    fn on_key(&mut self, _: compositor::Handle, _: keyboard::Handle, key_event: &keyboard::event::Key) {
        for key in key_event.pressed_keys() {
            if key == KEY_Escape {
                compositor::terminate()
            }
        }
    }
}

impl output::Handler for ExOutput {
    fn on_frame(&mut self, compositor: compositor::Handle, output: output::Handle) {
        with_handles!([(compositor: {compositor}), (output: {output})] => {
            let renderer = compositor.renderer.as_mut().unwrap();
            let state: &mut State = (&mut compositor.data).downcast_mut().unwrap();
            // NOTE gl functions will probably always be unsafe.
            let (width, height) = output.effective_resolution();
            let transform_matrix = output.transform_matrix();
            let mut renderer = renderer.render(output, None);
            renderer.clear([0.25, 0.25, 0.25, 1.0]);
            let cat_texture = state.cat_texture.as_mut().unwrap();
            let (cat_width, cat_height) = cat_texture.size();
            for touch_point in &mut state.touch_points {
                let x = (touch_point.x * f64::from(width)) as i32 - (cat_width / 2);
                let y = (touch_point.y * f64::from(height)) as i32 - (cat_height / 2);
                renderer.render_texture(cat_texture, transform_matrix, x, y, 1.0);
            }
        })
        .unwrap();
    }
}

impl touch::Handler for TouchHandlerEx {
    fn on_down(&mut self, compositor: compositor::Handle, _: touch::Handle, event: &touch::event::Down) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.downcast();
            let (x, y) = event.location();
            let point = TouchPoint { touch_id: event.touch_id(),
                                    x: x,
                                    y: y };
            wlr_log!(WLR_ERROR, "New touch point at {:?}", point);
            state.touch_points.push(point)
        })
        .unwrap();
    }

    fn on_up(&mut self, compositor: compositor::Handle, _: touch::Handle, event: &touch::event::Up) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.downcast();
            wlr_log!(WLR_ERROR,
                    "Removing {:?} from {:#?}",
                    event.touch_id(),
                    state.touch_points);
            let touch_id = event.touch_id();
            if let Some(index) = state.touch_points
                                    .iter()
                                    .position(|touch_point| touch_point.touch_id == touch_id)
            {
                state.touch_points.remove(index);
            }
        })
        .unwrap();
    }

    fn on_motion(&mut self, compositor: compositor::Handle, _: touch::Handle, event: &touch::event::Motion) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.downcast();
            let (x, y) = event.location();
            wlr_log!(WLR_ERROR, "New location: {:?}", (x, y));
            for touch_point in &mut state.touch_points {
                if touch_point.touch_id == event.touch_id() {
                    touch_point.x = x;
                    touch_point.y = y;
                }
            }
        })
        .unwrap();
    }
}

fn touch_added(_: compositor::Handle, _: touch::Handle) -> Option<Box<dyn touch::Handler>> {
    Some(Box::new(TouchHandlerEx))
}

fn keyboard_added(_: compositor::Handle, _: keyboard::Handle) -> Option<Box<dyn keyboard::Handler>> {
    Some(Box::new(ExKeyboardHandler))
}

fn main() {
    Logger::init(LevelFilter::Debug, None);
    let output_builder = output::manager::Builder::default().output_added(output_added);
    let input_builder = input::manager::Builder::default()
        .keyboard_added(keyboard_added)
        .touch_added(touch_added);
    let mut compositor = compositor::Builder::new()
        .gles2(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .build_auto(State::new());
    {
        let gles2 = &mut compositor.renderer.as_mut().unwrap();
        let compositor_data: &mut State = (&mut compositor.data).downcast_mut().unwrap();
        compositor_data.cat_texture = gles2.create_texture_from_pixels(
            TextureFormat::ABGR8888.into(),
            CAT_WIDTH * 4,
            CAT_WIDTH,
            CAT_HEIGHT,
            CAT_DATA
        );
    }
    compositor.run();
}
