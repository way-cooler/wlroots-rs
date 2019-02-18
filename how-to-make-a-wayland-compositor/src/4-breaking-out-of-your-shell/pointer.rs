use wlroots::{wlroots_dehandle,
              compositor,
              input::pointer,
              seat::Capability,
              cursor};

use crate::{CompositorState, Inputs, Shells};

pub struct CursorHandler;

impl cursor::Handler for CursorHandler {}

pub struct PointerHandler;

impl pointer::Handler for PointerHandler {
    #[wlroots_dehandle]
    fn on_button(&mut self,
                 compositor: compositor::Handle,
                 _pointer_handle: pointer::Handle,
                 event: &pointer::event::Button) {
        use wlroots::WLR_BUTTON_RELEASED;
        #[dehandle] let compositor = compositor;
        let CompositorState { shells: Shells { ref mut xdg_shells },
                              ref mut inputs,
                              ref seat_handle,
                              .. } = compositor.downcast();
        inputs.clicked = event.state() == WLR_BUTTON_RELEASED;
        if !inputs.clicked {
            return
        }
        if let Some(shell_handle) = xdg_shells.back() {
            #[dehandle] let shell = shell_handle;
            #[dehandle] let surface = shell.surface();
            #[dehandle] let seat = seat_handle;
            let (mut keycodes, mut modifier_masks) = inputs.get_keyboard_info();
            seat.keyboard_notify_enter(surface,
                                       &mut keycodes,
                                       &mut modifier_masks);
        }
    }

    /// Triggered when the pointer is moved on the Wayland and X11 backends.
    #[wlroots_dehandle]
    fn on_motion_absolute(&mut self,
                          compositor_handle: compositor::Handle,
                          _pointer_handle: pointer::Handle,
                          absolute_motion_event: &pointer::event::AbsoluteMotion) {
        #[dehandle] let compositor = compositor_handle;
        let &mut CompositorState { ref cursor_handle, .. } = compositor.downcast();
        #[dehandle] let cursor = cursor_handle;
        let (x, y) = absolute_motion_event.pos();
        cursor.warp_absolute(absolute_motion_event.device(), x, y);
    }

    #[wlroots_dehandle]
    /// Triggered when the pointer is moved in the DRM backend.
    fn on_motion(&mut self,
                 compositor_handle: compositor::Handle,
                 _pointer_handle: pointer::Handle,
                 motion_event: &pointer::event::Motion) {
        #[dehandle] let compositor = compositor_handle;
        let &mut CompositorState { ref cursor_handle, .. } = compositor.downcast();
        #[dehandle] let cursor = cursor_handle;
        let (delta_x, delta_y) = motion_event.delta();
        cursor.move_relative(None, delta_x, delta_y);
    }

    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 pointer_handle: pointer::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let &mut CompositorState { ref seat_handle,
                                   inputs: Inputs { ref mut pointers, .. },
                                   .. } = compositor.downcast();
        pointers.remove(&pointer_handle);
        if pointers.len() == 0 {
            #[dehandle] let seat = seat_handle;
            let mut cap = seat.capabilities();
            cap.remove(Capability::Pointer);
            seat.set_capabilities(cap)
        }
    }
}

#[wlroots_dehandle]
pub fn pointer_added(compositor_handle: compositor::Handle,
                     pointer_handle: pointer::Handle)
                     -> Option<Box<pointer::Handler>> {
    #[dehandle] let compositor = compositor_handle;
    #[dehandle] let pointer = pointer_handle;
    let CompositorState { ref cursor_handle,
                          ref seat_handle,
                          ref mut xcursor_manager,
                          inputs: Inputs { ref mut pointers, ..  },
                          .. } = compositor.downcast();
    pointers.insert(pointer_handle.clone());
    if pointers.len() == 1 {
        #[dehandle] let seat = seat_handle;
        let mut cap = seat.capabilities();
        cap.insert(Capability::Pointer);
        seat.set_capabilities(cap)
    }
    #[dehandle] let cursor = cursor_handle;
    xcursor_manager.set_cursor_image("left_ptr".to_string(), cursor);
    cursor.attach_input_device(pointer.input_device());
    Some(Box::new(PointerHandler) as Box<pointer::Handler>)
}
