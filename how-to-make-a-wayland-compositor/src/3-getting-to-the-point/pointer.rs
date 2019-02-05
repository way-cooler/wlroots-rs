use wlroots::{wlroots_dehandle,
              compositor,
              input::pointer};

use CompositorState;

pub struct PointerHandler;

pub fn pointer_added(_compositor_handle: compositor::Handle,
                     _pointer_handle: pointer::Handle)
                     -> Option<Box<pointer::Handler>> {
    Some(Box::new(PointerHandler))
}

impl pointer::Handler for PointerHandler {
    #[wlroots_dehandle]
    fn on_motion(&mut self,
                 compositor_handle: compositor::Handle,
                 _pointer_handle: pointer::Handle,
                 motion_event: &pointer::event::Motion) {
        #[dehandle] let compositor = compositor_handle;
        let &mut CompositorState { ref mut cursor, .. } = compositor.downcast();
        if let Some(cursor) = cursor.as_mut() {
            let (delta_x, delta_y) = motion_event.delta();
            let (cur_x, cur_y) = cursor.coords();
            cursor.move_to(cur_x + delta_x, cur_y + delta_y)
                .expect("Could not move cursor");
        }
    }
}
