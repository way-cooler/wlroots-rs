extern crate wlroots;

use wlroots::{compositor,
              input::{self, keyboard},
              utils::log::{WLR_DEBUG, init_logging},
              xkbcommon::xkb::keysyms
};

fn main() {
    init_logging(WLR_DEBUG, None);
    compositor::Builder::new()
        .input_manager(Box::new(InputManager))
        .build_auto(())
        .run()
}

struct InputManager;

impl input::ManagerHandler for InputManager {
    fn keyboard_added(&mut self,
                      _compositor_handle: compositor::Handle,
                      _keyboard_handle: keyboard::Handle)
                      -> Option<Box<keyboard::Handler>> {
        Some(Box::new(KeyboardHandler))
    }
}

struct KeyboardHandler;

impl keyboard::Handler for KeyboardHandler {
    fn on_key(&mut self,
              compositor_handle: compositor::Handle,
              _keyboard_handle: keyboard::Handle,
              key_event: &keyboard::event::Key) {
        for key in key_event.pressed_keys() {
            match key {
                keysyms::KEY_Escape => wlroots::compositor::terminate(),
                keysyms::KEY_XF86Switch_VT_1 ..= keysyms::KEY_XF86Switch_VT_12 => {
                    compositor_handle.run(|compositor| {
                        let backend = compositor.backend_mut();
                        if let Some(mut session) = backend.get_session() {
                            session.change_vt(key - keysyms::KEY_XF86Switch_VT_1 + 1);
                        }
                    }).unwrap();
                }
                _ => { /* Do nothing */ }
            }
        }
    }
}
