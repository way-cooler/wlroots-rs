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
    fn map_request(&mut self,
                   compositor_handle: compositor::Handle,
                   _surface_handle: surface::Handle,
                   shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut xdg_shells }, ..} =
            compositor.downcast();
        xdg_shells.push_back(shell_handle.clone());
    }

    #[wlroots_dehandle]
    fn unmap_request(&mut self,
                     compositor_handle: compositor::Handle,
                     _surface_handle: surface::Handle,
                     shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut xdg_shells }, ..} =
            compositor.downcast();
        xdg_shells.retain(|shell| *shell != shell_handle);
    }

    #[wlroots_dehandle]
    fn destroyed(&mut self,
                 compositor_handle: compositor::Handle,
                 shell_handle: xdg_shell::Handle) {
        #[dehandle] let compositor = compositor_handle;
        let CompositorState { shells: Shells { ref mut xdg_shells }, ..} =
            compositor.downcast();
        xdg_shells.retain(|shell| *shell != shell_handle);
    }
}


#[wlroots_dehandle]
pub fn new_surface(_compositor_handle: compositor::Handle,
                   _shell_handle: xdg_shell::Handle)
                   -> (Option<Box<xdg_shell::Handler>>, Option<Box<surface::Handler>>) {
    (Some(Box::new(XdgShellHandler) as _), Some(Box::new(SurfaceHandler) as _))
}
