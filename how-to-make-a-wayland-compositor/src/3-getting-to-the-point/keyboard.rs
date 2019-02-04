use wlroots::{compositor,
              input::keyboard,
              xkbcommon::xkb::keysyms,
              wlr_key_state::WLR_KEY_PRESSED};

pub fn keyboard_added(_compositor_handle: compositor::Handle,
                  _keyboard_handle: keyboard::Handle)
                  -> Option<Box<keyboard::Handler>> {
    Some(Box::new(KeyboardHandler::default()))
}

#[derive(Default)]
struct KeyboardHandler {
    shift_pressed: bool,
    ctrl_pressed: bool
}

impl keyboard::Handler for KeyboardHandler {
    fn on_key(&mut self,
              compositor_handle: compositor::Handle,
              _keyboard_handle: keyboard::Handle,
              key_event: &keyboard::event::Key) {
        for key in key_event.pressed_keys() {
            match key {
                keysyms::KEY_Control_L | keysyms::KEY_Control_R =>
                    self.ctrl_pressed = key_event.key_state() == WLR_KEY_PRESSED,
                keysyms::KEY_Shift_L | keysyms::KEY_Shift_R =>
                    self.shift_pressed = key_event.key_state() == WLR_KEY_PRESSED,
                keysyms::KEY_Escape => {
                    if self.shift_pressed && self.ctrl_pressed {
                        wlroots::compositor::terminate()
                    }
                },
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
