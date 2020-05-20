//! Handler for tablet tools

use crate::libc;
use crate::wayland_sys::server::WAYLAND_SERVER_HANDLE;
use wlroots_sys::wlr_input_device;

use crate::{
    compositor,
    input::tablet_tool::{self, TabletTool},
    utils::Handleable
};

#[allow(unused_variables)]
pub trait Handler {
    /// Callback that is triggered when an axis event fires
    fn on_axis(
        &mut self,
        compositor_handle: compositor::Handle,
        tablet_tool_handle: tablet_tool::Handle,
        event: &tablet_tool::event::Axis
    ) {
    }

    /// Callback that is triggered when a table tool is brought close to the
    /// input source.
    fn on_proximity(
        &mut self,
        compositor_handle: compositor::Handle,
        tablet_tool_handle: tablet_tool::Handle,
        event: &tablet_tool::event::Proximity
    ) {
    }

    /// Callback that is triggered when a table tool's tip touches the input
    /// source.
    fn on_tip(
        &mut self,
        compositor_handle: compositor::Handle,
        tablet_tool_handle: tablet_tool::Handle,
        event: &tablet_tool::event::Tip
    ) {
    }

    /// Callback that is triggered when a button is pressed on the tablet tool.
    fn on_button(
        &mut self,
        compositor_handle: compositor::Handle,
        tablet_tool_handle: tablet_tool::Handle,
        event: &tablet_tool::event::Button
    ) {
    }

    /// Callback that is triggered when a tablet tool is destroyed.
    fn destroyed(&mut self, compositor_handle: compositor::Handle, tablet_tool_handle: tablet_tool::Handle) {}
}

wayland_listener!(pub(crate) TabletToolWrapper, (TabletTool, Box<dyn Handler>), [
    on_destroy_listener => on_destroy_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,|
    unsafe {
        let input_device_ptr = data as *mut wlr_input_device;
        {
            let (ref mut tool, ref mut tablet_tool_handler) = this.data;
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            tablet_tool_handler.destroyed(compositor, tool.weak_reference());
        }
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.on_destroy_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.axis_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.proximity_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.tip_listener()).link as *mut _ as _);
        ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                      wl_list_remove,
                      &mut (*this.button_listener()).link as *mut _ as _);
        Box::from_raw((*input_device_ptr).data as *mut TabletToolWrapper);
    };
    axis_listener => axis_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,| unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = tablet_tool::event::Axis::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_axis(compositor,
                        tool.weak_reference(),
                        &event);
    };
    proximity_listener => proximity_notify: |this: &mut TabletToolWrapper,
    data: *mut libc::c_void,|
    unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = tablet_tool::event::Proximity::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_proximity(compositor,
                             tool.weak_reference(),
                             &event);
    };
    tip_listener => tip_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,| unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = tablet_tool::event::Tip::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_tip(compositor,
                       tool.weak_reference(),
                       &event);
    };
    button_listener => button_notify: |this: &mut TabletToolWrapper, data: *mut libc::c_void,|
    unsafe {
        let (ref tool, ref mut handler) = this.data;
        let event = tablet_tool::event::Button::from_ptr(data as *mut _);
        let compositor = match compositor::handle() {
            Some(handle) => handle,
            None => return
        };

        handler.on_button(compositor,
                          tool.weak_reference(),
                          &event);
    };
]);
