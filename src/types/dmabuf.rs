//! Support for the DMABuf type

use wlroots_sys::{wl_display, wlr_linux_dmabuf_v1, wlr_linux_dmabuf_v1_create, 
                wlr_linux_dmabuf_v1_destroy};
use {crate::compositor::Compositor,
    crate::render::GenericRenderer};


#[derive(Debug)]
pub struct Dmabuf {
    dmabuf: *mut wlr_linux_dmabuf_v1
}

impl Dmabuf {
    pub fn new(compositor: &Compositor) -> Option<Self> {
        unsafe {
            // Get the renderer from compositor
            let renderer: Option<&GenericRenderer> = compositor.get_renderer().as_ref();
            match renderer {
                Some(ref renderer) => {
                    let dmabuf_raw = wlr_linux_dmabuf_v1_create(compositor.display as *mut wl_display, renderer.as_ptr());
                    if !dmabuf_raw.is_null() {
                        Some(Dmabuf { dmabuf: dmabuf_raw })
                    } else {
                        None
                    }
                }
                None => None
            }
        }
    }
}

impl Drop for Dmabuf {
    fn drop(&mut self) {
        unsafe { wlr_linux_dmabuf_v1_destroy(self.dmabuf) }
    }
}
