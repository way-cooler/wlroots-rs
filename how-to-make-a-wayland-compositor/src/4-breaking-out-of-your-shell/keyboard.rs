use wlroots::{wlroots_dehandle, compositor,
              input::keyboard,
              seat::Capability,
              xkbcommon::xkb::keysyms,
              wlr_key_state::WLR_KEY_PRESSED};

use crate::{CompositorState, Inputs};

#[wlroots_dehandle]
pub fn keyboard_added(compositor_handle: compositor::Handle,
                      keyboard_handle: keyboard::Handle)
                      -> Option<Box<keyboard::Handler>> {
    #[dehandle] let compositor = compositor_handle;
    let CompositorState { ref seat_handle,
                          inputs: Inputs { ref mut keyboards, ..  },
                          .. } = compositor.downcast();
    keyboards.insert(keyboard_handle.clone());
    if keyboards.len() == 1 {
        #[dehandle] let keyboard = keyboard_handle;
        #[dehandle] let seat = seat_handle;
        let mut cap = seat.capabilities();
        cap.insert(Capability::Keyboard);
        seat.set_capabilities(cap);
        seat.set_keyboard(keyboard.input_device());
    }
    Some(Box::new(KeyboardHandler) as _)
}

struct KeyboardHandler;

impl keyboard::Handler for KeyboardHandler {
    #[wlroots_dehandle]
    fn on_key(&mut self,
              compositor_handle: compositor::Handle,
              _keyboard_handle: keyboard::Handle,
              key_event: &keyboard::event::Key) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { inputs: Inputs { ref mut ctrl_pressed,
                                               ref mut shift_pressed, .. },
                              ref seat_handle, .. } =
            compositor.data.downcast_mut().unwrap();
        for key in key_event.pressed_keys() {
            match key {
                keysyms::KEY_Control_L | keysyms::KEY_Control_R =>
                    *ctrl_pressed = key_event.key_state() == WLR_KEY_PRESSED,
                keysyms::KEY_Shift_L | keysyms::KEY_Shift_R =>
                    *shift_pressed = key_event.key_state() == WLR_KEY_PRESSED,
                keysyms::KEY_Escape => {
                    if *shift_pressed && *ctrl_pressed {
                        wlroots::compositor::terminate()
                    }
                },
                keysyms::KEY_XF86Switch_VT_1 ..= keysyms::KEY_XF86Switch_VT_12 => {
                    if let Some(mut session) = compositor.backend.get_session() {
                        session.change_vt(key - keysyms::KEY_XF86Switch_VT_1 + 1);
                    }
                }
                _ => { /* Do nothing */ }
            }
        }
        #[dehandle] let seat = seat_handle;
        seat.keyboard_notify_key(key_event.time_msec(),
                                 key_event.keycode(),
                                 key_event.key_state() as u32)
    }

    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 keyboard_handle: keyboard::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { ref seat_handle,
                              inputs: Inputs { ref mut keyboards, ..  },
                              .. } = compositor.downcast();
        keyboards.remove(&keyboard_handle);
        if keyboards.len() == 0 {
            #[dehandle] let seat = seat_handle;
            let mut cap = seat.capabilities();
            cap.remove(Capability::Keyboard);
            seat.set_capabilities(cap)
        }
    }
}
