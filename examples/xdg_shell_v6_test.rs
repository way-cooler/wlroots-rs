extern crate log;
#[macro_use]
extern crate wlroots;

use std::{env, process::Command, thread};

use log::LevelFilter;
use wlroots::wlroots_dehandle;
use wlroots::wlroots_sys::wlr_key_state::WLR_KEY_PRESSED;
use wlroots::xkbcommon::xkb::keysyms;
use wlroots::{
    area::{Area, Origin, Size},
    compositor,
    cursor::{self, xcursor, Cursor},
    input::{self, keyboard, pointer},
    output::{self, layout::Layout},
    render::{matrix, Renderer},
    seat::{self, Seat},
    shell::xdg_shell_v6,
    surface,
    utils::{current_time, log::Logger, Handleable}
};
use wlroots::wlroots_sys::TransformOutput;

struct State {
    xcursor_manager: xcursor::Manager,
    keyboard: Option<keyboard::Handle>,
    layout: output::layout::Handle,
    cursor: cursor::Handle,
    shells: Vec<xdg_shell_v6::Handle>,
    seat_handle: Option<seat::Handle>
}

impl State {
    fn new(
        xcursor_manager: xcursor::Manager,
        layout: output::layout::Handle,
        cursor: cursor::Handle
    ) -> Self {
        State {
            xcursor_manager,
            layout,
            cursor,
            keyboard: None,
            seat_handle: None,
            shells: vec![]
        }
    }
}

struct SeatHandlerEx;

struct CursorEx;

impl cursor::Handler for CursorEx {}

impl seat::Handler for SeatHandlerEx {}

struct XdgV6ShellHandlerEx;
struct OutputLayoutEx;

impl output::layout::Handler for OutputLayoutEx {}

impl xdg_shell_v6::Handler for XdgV6ShellHandlerEx {
    fn destroyed(&mut self, compositor: compositor::Handle, shell: xdg_shell_v6::Handle) {
        with_handles!([(compositor: {compositor})] => {
            let state: &mut State = compositor.downcast();
            let weak = shell;
            if let Some(index) = state.shells.iter().position(|s| *s == weak) {
                state.shells.remove(index);
            }
        })
        .unwrap();
    }
}

struct SurfaceEx;

impl surface::Handler for SurfaceEx {
    fn on_commit(&mut self, _: compositor::Handle, surface: surface::Handle) {
        wlr_log!(WLR_DEBUG, "Commiting for surface {:?}", surface);
    }
}

#[wlroots_dehandle]
fn new_surface(
    compositor: compositor::Handle,
    shell: xdg_shell_v6::Handle
) -> xdg_shell_v6::NewSurfaceResult {
    {
        #[dehandle]
        let compositor = compositor;
        #[dehandle]
        let shell = shell;
        shell.ping();
        let state: &mut State = compositor.downcast();
        state.shells.push(shell.weak_reference());
        #[dehandle]
        let layout = &state.layout;
        for (output, _) in layout.outputs() {
            #[dehandle]
            let output = output;
            output.schedule_frame()
        }
    }
    (Some(Box::new(XdgV6ShellHandlerEx)), Some(Box::new(SurfaceEx)))
}

struct ExOutput;

struct ExPointer;

struct ExKeyboardHandler;

#[wlroots_dehandle]
fn output_added<'output>(
    compositor: compositor::Handle,
    builder: output::Builder<'output>
) -> Option<output::BuilderResult<'output>> {
    let result = builder.build_best_mode(ExOutput);
    {
        #[dehandle]
        let compositor = compositor;
        #[dehandle]
        let output = &result.output;
        let state: &mut State = compositor.data.downcast_mut().unwrap();
        // TODO use output config if present instead of auto
        #[dehandle]
        let layout = state.layout.clone();
        #[dehandle]
        let cursor = state.cursor.clone();
        layout.add_auto(output);
        cursor.attach_output_layout(layout);
        state.xcursor_manager.load(output.scale());
        state
            .xcursor_manager
            .set_cursor_image("left_ptr".to_string(), cursor);
        let (x, y) = cursor.coords();
        // https://en.wikipedia.org/wiki/Mouse_warping
        cursor.warp(None, x, y);
    }
    Some(result)
}

impl keyboard::Handler for ExKeyboardHandler {
    #[wlroots_dehandle]
    fn on_key(
        &mut self,
        compositor_handle: compositor::Handle,
        keyboard_handle: keyboard::Handle,
        key_event: &keyboard::event::Key
    ) {
        if key_event.key_state() == WLR_KEY_PRESSED {
            for key in key_event.pressed_keys() {
                match key {
                    keysyms::KEY_Escape => {
                        compositor::terminate();
                        return;
                    },
                    keysyms::KEY_F1 => {
                        thread::spawn(move || {
                            Command::new("weston-terminal").output().unwrap();
                        });
                        return;
                    },
                    keysyms::KEY_XF86Switch_VT_1...keysyms::KEY_XF86Switch_VT_12 => {
                        compositor_handle
                            .run(|compositor| {
                                if let Some(mut session) = compositor.backend.get_session() {
                                    session.change_vt(key - keysyms::KEY_XF86Switch_VT_1 + 1);
                                }
                            })
                            .unwrap();
                        return;
                    },
                    _ => { /*do nothing*/ }
                }
            }
        }

        #[dehandle]
        let compositor = compositor_handle;
        let state: &mut State = compositor.downcast();
        #[dehandle]
        let seat = state.seat_handle.clone().unwrap();
        #[dehandle]
        let keyboard = keyboard_handle;

        seat.set_keyboard(keyboard.input_device());
        seat.keyboard_notify_key(
            key_event.time_msec(),
            key_event.keycode(),
            key_event.key_state() as u32
        );
    }

    #[wlroots_dehandle]
    fn modifiers(&mut self, compositor_handle: compositor::Handle, keyboard_handle: keyboard::Handle) {
        #[dehandle]
        let compositor = compositor_handle;
        let state: &mut State = compositor.downcast();
        #[dehandle]
        let seat = state.seat_handle.clone().unwrap();
        #[dehandle]
        let keyboard = keyboard_handle;

        seat.set_keyboard(keyboard.input_device());
        seat.keyboard_notify_modifiers(&mut keyboard.get_modifier_masks());
    }
}

impl pointer::Handler for ExPointer {
    #[wlroots_dehandle]
    fn on_motion_absolute(
        &mut self,
        compositor: compositor::Handle,
        _: pointer::Handle,
        event: &pointer::event::AbsoluteMotion
    ) {
        #[dehandle]
        let compositor = compositor;
        let state: &mut State = compositor.downcast();
        let (x, y) = event.pos();
        #[dehandle]
        let cursor = &state.cursor;
        cursor.warp_absolute(event.device(), x, y)
    }

    #[wlroots_dehandle]
    fn on_motion(
        &mut self,
        compositor: compositor::Handle,
        _: pointer::Handle,
        event: &pointer::event::Motion
    ) {
        #[dehandle]
        let compositor = compositor;
        let state: &mut State = compositor.downcast();
        let (delta_x, delta_y) = event.delta();
        #[dehandle]
        let cursor = &state.cursor;
        cursor.move_relative(event.device(), delta_x, delta_y)
    }

    #[wlroots_dehandle]
    fn on_button(&mut self, compositor: compositor::Handle, _: pointer::Handle, _: &pointer::event::Button) {
        #[dehandle]
        let compositor = compositor;
        let state: &mut State = compositor.downcast();
        #[dehandle]
        let shell = &state.shells[0];
        if let Some(&mut xdg_shell_v6::ShellState::TopLevel(ref mut toplevel)) = shell.state() {
            toplevel.set_activated(true);
        }
        #[dehandle]
        let seat = state.seat_handle.clone().unwrap();
        #[dehandle]
        let keyboard = state.keyboard.clone().unwrap();
        #[dehandle]
        let surface = shell.surface();
        seat.set_keyboard(keyboard.input_device());
        seat.keyboard_notify_enter(
            surface,
            &mut keyboard.keycodes(),
            &mut keyboard.get_modifier_masks()
        )
    }
}

impl output::Handler for ExOutput {
    #[wlroots_dehandle]
    fn on_frame(&mut self, compositor: compositor::Handle, output: output::Handle) {
        #[dehandle]
        let compositor = compositor;
        #[dehandle]
        let output = output;
        let state: &mut State = compositor.data.downcast_mut().unwrap();
        let renderer = compositor
            .renderer
            .as_mut()
            .expect("Compositor was not loaded with a renderer");
        let mut render_context = renderer.render(output, None);
        render_context.clear([0.25, 0.25, 0.25, 1.0]);
        render_shells(state, &mut render_context)
    }
}

fn pointer_added(_: compositor::Handle, _: pointer::Handle) -> Option<Box<dyn pointer::Handler>> {
    Some(Box::new(ExPointer))
}

#[wlroots_dehandle]
fn keyboard_added(
    compositor: compositor::Handle,
    keyboard: keyboard::Handle
) -> Option<Box<dyn keyboard::Handler>> {
    {
        #[dehandle]
        let compositor = compositor;
        #[dehandle]
        let keyboard = keyboard;
        let state: &mut State = compositor.downcast();
        state.keyboard = Some(keyboard.weak_reference());
        #[dehandle]
        let seat = state.seat_handle.as_ref().unwrap();
        seat.set_keyboard(keyboard.input_device());
    }
    Some(Box::new(ExKeyboardHandler))
}

fn main() {
    Logger::init(LevelFilter::Debug, None);
    let cursor = Cursor::create(Box::new(CursorEx));
    let mut xcursor_manager = xcursor::Manager::create("default".to_string(), 24).expect(
        "Could not create xcursor \
         manager"
    );
    xcursor_manager.load(1.0);
    cursor
        .run(|c| xcursor_manager.set_cursor_image("left_ptr".to_string(), c))
        .unwrap();
    let layout = Layout::create(Box::new(OutputLayoutEx));

    let output_builder = output::manager::Builder::default().output_added(output_added);
    let input_builder = input::manager::Builder::default()
        .keyboard_added(keyboard_added)
        .pointer_added(pointer_added);
    let xdg_shell_v6_builder = xdg_shell_v6::manager::Builder::default().surface_added(new_surface);
    let mut compositor = compositor::Builder::new()
        .gles2(true)
        .wl_shm(true)
        .input_manager(input_builder)
        .output_manager(output_builder)
        .xdg_shell_v6_manager(xdg_shell_v6_builder)
        .build_auto(State::new(xcursor_manager, layout, cursor));

    {
        let seat_handle = Seat::create(&mut compositor, "seat0".into(), Box::new(SeatHandlerEx));
        seat_handle
            .run(|seat| {
                seat.set_capabilities(seat::Capability::all());
            })
            .unwrap();
        let state: &mut State = compositor.downcast();
        state.seat_handle = Some(seat_handle);
    }
    env::set_var("WAYLAND_DISPLAY", compositor.socket_name());
    compositor.run();
}

/// Render the shells in the current compositor state on the given output.
#[wlroots_dehandle]
fn render_shells(state: &mut State, renderer: &mut Renderer) {
    let shells = state.shells.clone();
    for shell in shells {
        #[dehandle]
        let shell = shell;
        #[dehandle]
        let surface = shell.surface();
        #[dehandle]
        let layout = &state.layout;
        let (width, height) = surface.current_state().size();
        let (render_width, render_height) = (
            width * renderer.output.scale() as i32,
            height * renderer.output.scale() as i32
        );
        let (lx, ly) = (0.0, 0.0);
        let render_box = Area::new(
            Origin::new(lx as i32, ly as i32),
            Size::new(render_width, render_height)
        );
        if layout.intersects(renderer.output, render_box) {
            let transform = renderer.output.get_transform().invert();
            let matrix = matrix::project_box(render_box, transform, 0.0, renderer.output.transform_matrix());
            if let Some(texture) = surface.texture().as_ref() {
                renderer.render_texture_with_matrix(texture, matrix);
            }
            surface.send_frame_done(current_time());
        }
    }
}
