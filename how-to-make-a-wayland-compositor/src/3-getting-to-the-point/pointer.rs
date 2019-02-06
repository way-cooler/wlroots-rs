use wlroots::{wlroots_dehandle,
              compositor,
              input::pointer,
              cursor};

use CompositorState;

pub struct CursorHandler;

impl cursor::Handler for CursorHandler {}

pub struct PointerHandler;

impl pointer::Handler for PointerHandler {
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
    fn on_motion(&mut self,
                 compositor_handle: compositor::Handle,
                 _pointer_handle: pointer::Handle,
                 motion_event: &pointer::event::Motion) {
        #[dehandle] let compositor = compositor_handle;
        let &mut CompositorState { ref cursor_handle, .. } = compositor.downcast();
        #[dehandle] let cursor = cursor_handle;
        let (delta_x, delta_y) = motion_event.delta();
        cursor.move_to(None, delta_x, delta_y);
    }
}

#[wlroots_dehandle]
pub fn pointer_added(compositor_handle: compositor::Handle,
                     pointer_handle: pointer::Handle)
                     -> Option<Box<pointer::Handler>> {
    #[dehandle] let compositor = compositor_handle;
    #[dehandle] let pointer = pointer_handle;
    let CompositorState { ref cursor_handle, .. } = compositor.downcast();
    #[dehandle] let cursor = cursor_handle;
    cursor.attach_input_device(pointer.input_device());
    Some(Box::new(PointerHandler) as Box<pointer::Handler>)
}
