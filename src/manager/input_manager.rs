//! Manager that is called when a seat is created or destroyed.
//! Pass a struct that implements this trait to the `Compositor` during
//! initialization.

use crate::libc;

use std::{env, panic, process::abort};

use crate::wayland_sys::server::signal::wl_signal_add;
use wlroots_sys::{
    wlr_input_device, wlr_input_device_type, wlr_keyboard_set_keymap, wlr_keyboard_set_repeat_info,
    xkb_context_flags::*, xkb_context_new, xkb_context_unref, xkb_keymap_compile_flags::*,
    xkb_keymap_new_from_names, xkb_keymap_unref, xkb_rule_names
};

use crate::{
    compositor,
    input::{
        self,
        keyboard::{self, Keyboard, KeyboardWrapper},
        pointer::{self, Pointer, PointerWrapper},
        switch::{self, Switch, SwitchWrapper},
        tablet_pad::{self, TabletPad, TabletPadWrapper},
        tablet_tool::{self, TabletTool, TabletToolWrapper},
        touch::{self, Touch, TouchWrapper}
    },
    utils::{safe_as_cstring, Handleable}
};

/// Callback triggered when an input device is added.
///
/// # Panics
/// Any panic in this function will cause the process to abort.
pub type InputAdded = fn(compositor_handle: compositor::Handle, device: &mut input::Device);

/// Callback triggered when a keyboard device is added.
///
/// # Panics
/// Any panic in this function will cause the process to abort.
pub type KeyboardAdded = fn(
    compositor_handle: compositor::Handle,
    keyboard_handle: keyboard::Handle
) -> Option<Box<dyn keyboard::Handler>>;

/// Callback triggered when a pointer device is added.
///
/// # Panics
/// Any panic in this function will cause the process to abort.
pub type PointerAdded = fn(
    compositor_handle: compositor::Handle,
    pointer_handle: pointer::Handle
) -> Option<Box<dyn pointer::Handler>>;

/// Callback triggered when a touch device is added.
///
/// # Panics
/// Any panic in this function will cause the process to abort.
pub type TouchAdded =
    fn(compositor_handle: compositor::Handle, touch_handle: touch::Handle) -> Option<Box<dyn touch::Handler>>;

/// Callback triggered when a tablet tool is added.
///
///
/// # Panics
/// Any panic in this function will cause the process to abort.
pub type TabletToolAdded = fn(
    compositor_handle: compositor::Handle,
    tablet_tool_handle: tablet_tool::Handle
) -> Option<Box<dyn tablet_tool::Handler>>;

/// Callback triggered when a tablet pad is added.
///
///
/// # Panics
/// Any panic in this function will cause the process to abort.
pub type TabletPadAdded = fn(
    compositor_handle: compositor::Handle,
    tablet_pad_handle: tablet_pad::Handle
) -> Option<Box<dyn tablet_pad::Handler>>;

pub type SwitchAdded =
    fn(compositor_handle: compositor::Handle, switch_handle: switch::Handle) -> Option<Box<dyn switch::Handler>>;

wayland_listener_static! {
    static mut MANAGER;
    (Manager, Builder): [
        // NOTE
        // This is a macro hack to add these as arguments to the builder.
        // The callbacks will be storted in the manager, but they'll have no
        // listener to wait for since this is the only event on this interface
        // (and why we need this hack).
        [
            keyboard_added: KeyboardAdded,
            pointer_added: PointerAdded,
            touch_added: TouchAdded,
            tablet_tool_added: TabletToolAdded,
            tablet_pad_added: TabletPadAdded,
            switch_added: SwitchAdded
        ]
        (InputAdded, add_listener, input_added) => (add_notify, input_added):
        |manager: &mut Manager, data: *mut libc::c_void,| unsafe {
            let compositor = match compositor::handle() {
                Some(handle) => handle,
                None => return
            };
            let data = data as *mut wlr_input_device;
            use self::wlr_input_device_type::*;
            let mut dev = input::Device::from_ptr(data);
            let res = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                match dev.dev_type() {
                    WLR_INPUT_DEVICE_KEYBOARD => {
                        // Boring setup that we won't make the user do
                        add_keyboard(&mut dev);
                        let keyboard = match Keyboard::new_from_input_device(data) {
                            Some(dev) => dev,
                            None => {
                                wlr_log!(WLR_ERROR, "Device {:#?} was not a keyboard!", dev);
                                abort()
                            }
                        };
                        let keyboard_handle = keyboard.weak_reference();
                        let res = manager.keyboard_added.and_then(|f| f(compositor.clone(), keyboard_handle));
                        if let Some(keyboard_handler) = res {
                            let mut keyboard = KeyboardWrapper::new((keyboard,
                                                                     keyboard_handler));
                            wl_signal_add(&mut (*dev.dev_union().keyboard).events.key as *mut _ as _,
                                          keyboard.key_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().keyboard).events.modifiers
                                          as *mut _ as _,
                                          keyboard.modifiers_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().keyboard).events.keymap as *mut _ as _,
                                          keyboard.keymap_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().keyboard).events.repeat_info
                                          as *mut _ as _,
                                          keyboard.repeat_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                                          keyboard.on_destroy_listener() as _);
                            (*data).data = Box::into_raw(keyboard) as _;
                        }
                    },
                    WLR_INPUT_DEVICE_POINTER => {
                        let pointer = match Pointer::new_from_input_device(data) {
                            Some(dev) => dev,
                            None => {
                                wlr_log!(WLR_ERROR, "Device {:#?} was not a pointer!", dev);
                                abort()
                            }
                        };
                        let pointer_handle = pointer.weak_reference();
                        let res = manager.pointer_added.and_then(|f| f(compositor.clone(), pointer_handle));
                        if let Some(pointer_handler) = res {
                            let mut pointer = PointerWrapper::new((pointer, pointer_handler));
                            wl_signal_add(&mut (*dev.dev_union().pointer).events.motion as *mut _ as _,
                                          pointer.motion_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().pointer)
                                          .events.motion_absolute as *mut _ as _,
                                          pointer.motion_absolute_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().pointer).events.button as *mut _ as _,
                                          pointer.button_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().pointer).events.axis as *mut _ as _,
                                          pointer.axis_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                                          pointer.on_destroy_listener() as _);
                            (*data).data = Box::into_raw(pointer) as _;
                        }
                    },
                    WLR_INPUT_DEVICE_TOUCH => {
                        let touch = match Touch::new_from_input_device(data) {
                            Some(dev) => dev,
                            None => {
                                wlr_log!(WLR_ERROR, "Device {:#?} was not a touch", dev);
                                abort()
                            }
                        };
                        let touch_handle = touch.weak_reference();
                        let res = manager.touch_added.and_then(|f| f(compositor.clone(), touch_handle));
                        if let Some(touch_handler) = res {
                            let mut touch = TouchWrapper::new((touch, touch_handler));
                            wl_signal_add(&mut (*dev.dev_union().touch).events.down as *mut _ as _,
                                          touch.down_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().touch).events.up as *mut _ as _,
                                          touch.up_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().touch).events.motion as *mut _ as _,
                                          touch.motion_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.dev_union().touch).events.cancel as *mut _ as _,
                                          touch.cancel_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                                          touch.on_destroy_listener() as _);
                            (*data).data = Box::into_raw(touch) as _;
                        }
                    },
                    WLR_INPUT_DEVICE_TABLET_TOOL => {
                        let tablet_tool = match TabletTool::new_from_input_device(data) {
                            Some(dev) => dev,
                            None => {
                                wlr_log!(WLR_ERROR, "Device {:#?}, was not a tablet tool", dev);
                                abort()
                            }
                        };
                        let tablet_tool_handle = tablet_tool.weak_reference();
                        let res = manager.tablet_tool_added.and_then(|f| f(compositor.clone(),
                                                                           tablet_tool_handle));
                        if let Some(tablet_tool_handler) = res {
                            let mut tablet_tool = TabletToolWrapper::new((tablet_tool,
                                                                          tablet_tool_handler));
                            let tool_ptr = &mut (*dev.dev_union().tablet);
                            wl_signal_add(&mut tool_ptr.events.axis as *mut _ as _,
                                          tablet_tool.axis_listener() as *mut _ as _);
                            wl_signal_add(&mut tool_ptr.events.proximity as *mut _ as _,
                                          tablet_tool.proximity_listener() as *mut _ as _);
                            wl_signal_add(&mut tool_ptr.events.tip as *mut _ as _,
                                          tablet_tool.tip_listener() as *mut _ as _);
                            wl_signal_add(&mut tool_ptr.events.button as *mut _ as _,
                                          tablet_tool.button_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                                          tablet_tool.on_destroy_listener() as _);
                            (*data).data = Box::into_raw(tablet_tool) as _;
                        }
                    },
                    WLR_INPUT_DEVICE_TABLET_PAD => {
                        let tablet_pad = match TabletPad::new_from_input_device(data) {
                            Some(dev) => dev,
                            None => {
                                wlr_log!(WLR_ERROR, "Device {:#?}, was not a tablet pad", dev);
                                abort()
                            }
                        };
                        let tablet_pad_handle = tablet_pad.weak_reference();
                        let res = manager.tablet_pad_added.and_then(|f| f(compositor.clone(),
                                                                          tablet_pad_handle));
                        if let Some(tablet_pad_handler) = res {
                            let mut tablet_pad = TabletPadWrapper::new((tablet_pad,
                                                                        tablet_pad_handler));
                            let pad_ptr = &mut (*dev.dev_union().tablet_pad);
                            wl_signal_add(&mut pad_ptr.events.button as *mut _ as _,
                                          tablet_pad.button_listener() as *mut _ as _);
                            wl_signal_add(&mut pad_ptr.events.ring as *mut _ as _,
                                          tablet_pad.ring_listener() as *mut _ as _);
                            wl_signal_add(&mut pad_ptr.events.strip as *mut _ as _,
                                          tablet_pad.strip_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                                          tablet_pad.on_destroy_listener() as _);
                            (*data).data = Box::into_raw(tablet_pad) as _;
                        }
                    }
                    WLR_INPUT_DEVICE_SWITCH => {
                        let switch = match Switch::new_from_input_device(data) {
                            Some(dev) => dev,
                            None => {
                                wlr_log!(WLR_ERROR, "Device {:#?} was not a switch", dev);
                                abort();
                            }
                        };
                        let switch_handle = switch.weak_reference();
                        let res = manager.switch_added.and_then(|f| f(compositor.clone(), switch_handle));
                        if let Some(switch_handler) = res {
                            let mut switch = SwitchWrapper::new((switch, switch_handler));
                            let switch_ptr = &mut (*dev.dev_union().switch_device);
                            wl_signal_add(&mut switch_ptr.events.toggle as *mut _ as _,
                                          switch.on_toggle_listener() as *mut _ as _);
                            wl_signal_add(&mut (*dev.as_ptr()).events.destroy as *mut _ as _,
                                          switch.on_destroy_listener() as _);
                            (*data).data = Box::into_raw(switch) as _;
                        }
                    }
                }
                manager.input_added.map(|f| f(compositor, &mut dev))
            }));
            match res {
                Ok(_) => {},
                // NOTE
                // Either Wayland or wlroots does not handle failure to set up input correctly.
                // Calling wl_display_terminate does not work if input is incorrectly set up.
                //
                // Instead, execution keeps going with an eventual segfault (if lucky).
                //
                // To fix this, we abort the process if there was a panic in input setup.
                Err(_) => abort()
            }
    };
    ]
}

pub(crate) unsafe fn add_keyboard(dev: &mut input::Device) {
    // Set the XKB settings
    let rules = safe_as_cstring(env::var("XKB_DEFAULT_RULES").unwrap_or_else(|_| "".into()));
    let model = safe_as_cstring(env::var("XKB_DEFAULT_MODEL").unwrap_or_else(|_| "".into()));
    let layout = safe_as_cstring(env::var("XKB_DEFAULT_LAYOUT").unwrap_or_else(|_| "".into()));
    let variant = safe_as_cstring(env::var("XKB_DEFAULT_VARIANT").unwrap_or_else(|_| "".into()));
    let options = safe_as_cstring(env::var("XKB_DEFAULT_OPTIONS").unwrap_or_else(|_| "".into()));
    wlr_log!(WLR_DEBUG, "Using xkb rules: {:?}", rules);
    wlr_log!(WLR_DEBUG, "Using xkb model: {:?}", model);
    wlr_log!(WLR_DEBUG, "Using xkb layout: {:?}", layout);
    wlr_log!(WLR_DEBUG, "Using xkb variant: {:?}", variant);
    wlr_log!(WLR_DEBUG, "Using xkb options: {:?}", options);
    let rules = xkb_rule_names {
        rules: rules.into_raw(),
        model: model.into_raw(),
        layout: layout.into_raw(),
        variant: variant.into_raw(),
        options: options.into_raw()
    };
    let context = xkb_context_new(XKB_CONTEXT_NO_FLAGS);
    if context.is_null() {
        panic!("Failed to create XKB context");
    }
    let xkb_map = xkb_keymap_new_from_names(context, &rules, XKB_KEYMAP_COMPILE_NO_FLAGS);
    if xkb_map.is_null() {
        panic!("Could not create xkb map");
    }
    wlr_keyboard_set_keymap(dev.dev_union().keyboard, xkb_map);
    xkb_keymap_unref(xkb_map);
    xkb_context_unref(context);
    wlr_keyboard_set_repeat_info(dev.dev_union().keyboard, 25, 600);
}
