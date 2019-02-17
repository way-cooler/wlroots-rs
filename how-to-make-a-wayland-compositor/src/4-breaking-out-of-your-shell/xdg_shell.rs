use crate::{CompositorState, Shells};

use wlroots::{wlroots_dehandle,
              compositor,
              surface,
              shell::xdg_shell::{self}};

struct SurfaceHandler;

impl surface::Handler for SurfaceHandler {}

struct XdgShellHandler;

impl xdg_shell::Handler for XdgShellHandler {
    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut xdg_shells }, ..} =
            compositor.downcast();
        xdg_shells.remove(&shell_handle);
    }
}


#[wlroots_dehandle]
pub fn new_surface(compositor_handle: compositor::Handle,
                   shell_handle: xdg_shell::Handle)
                   -> (Option<Box<xdg_shell::Handler>>, Option<Box<surface::Handler>>) {
    #[dehandle] let compositor = compositor_handle;
    let CompositorState { shells: Shells { ref mut xdg_shells }, ..} =
        compositor.downcast();
    xdg_shells.insert(shell_handle.clone());
    (Some(Box::new(XdgShellHandler) as _), Some(Box::new(SurfaceHandler) as _))
}
