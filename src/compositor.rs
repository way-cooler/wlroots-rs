//! Main entry point to the library.
//! See examples for documentation on how to use this struct.

use libc;
use std::{env, panic, ptr, any::Any, cell::{Cell, UnsafeCell}, ffi::CStr, rc::{Rc, Weak}};

use {DataDeviceManager, Surface, SurfaceHandle, XWaylandManagerHandler, XWaylandServer};
use errors::{HandleErr, HandleResult};
use types::surface::{InternalSurface, InternalSurfaceState};
use extensions::server_decoration::ServerDecorationManager;
use manager::{InputManager, InputManagerHandler, OutputManager, OutputManagerHandler,
              XdgShellManager, XdgShellManagerHandler, XdgV6ShellManager,
              XdgV6ShellManagerHandler, LayerShellManagerHandler, LayerShellManager};
use render::GenericRenderer;

use wayland_sys::server::{wl_display, wl_event_loop, signal::wl_signal_add, WAYLAND_SERVER_HANDLE};
use wlroots_sys::{wlr_backend, wlr_backend_autocreate, wlr_backend_destroy, wlr_backend_start,
                  wlr_compositor, wlr_compositor_create, wlr_compositor_destroy, wlr_wl_shell,
                  wlr_xdg_shell_v6, wlr_xdg_shell_v6_create,
                  wlr_xdg_shell, wlr_xdg_shell_create, wlr_layer_shell_create, wlr_layer_shell};
use wlroots_sys::wayland_server::sys::wl_display_init_shm;

/// Global compositor pointer, used to refer to the compositor state unsafely.
pub(crate) static mut COMPOSITOR_PTR: *mut Compositor = 0 as *mut _;

pub trait CompositorHandler {
    /// Callback that's triggered when a surface is provided to the compositor.
    fn new_surface(&mut self, CompositorHandle, SurfaceHandle) {}

    /// Callback that's triggered during shutdown.
    fn on_shutdown(&mut self) {}
}

impl CompositorHandler for () {}

wayland_listener!(InternalCompositor, Box<CompositorHandler>, [
    new_surface_listener => new_surface_notify: |this: &mut InternalCompositor,
                                                 surface_ptr: *mut libc::c_void,|
    unsafe {
        let handler = &mut this.data;
        let surface_ptr = surface_ptr as _;
        let compositor = (&mut *COMPOSITOR_PTR).weak_reference();
        let surface = Surface::new(surface_ptr);
        handler.new_surface(compositor.clone(), surface.weak_reference());
        let mut internal_surface = InternalSurface::new((surface, Box::new(())));
        wl_signal_add(&mut (*surface_ptr).events.commit as *mut _ as _,
                      internal_surface.on_commit_listener() as _);
        wl_signal_add(&mut (*surface_ptr).events.new_subsurface as *mut _ as _,
                      internal_surface.new_subsurface_listener() as _);
        wl_signal_add(&mut (*surface_ptr).events.destroy as *mut _ as _,
                        internal_surface.on_destroy_listener() as _);
        let surface_data = (*surface_ptr).data as *mut InternalSurfaceState;
        (*surface_data).surface = Box::into_raw(internal_surface);
    };
    shutdown_listener => shutdown_notify: |this: &mut InternalCompositor,
                                           _data: *mut libc::c_void,|
    unsafe {
        let handler = &mut this.data;
        handler.on_shutdown();
    };
]);

#[derive(Debug, Clone)]
pub struct CompositorHandle {
    /// This ensures that this handle is still alive and not already borrowed.
    handle: Weak<Cell<bool>>
}

#[allow(dead_code)]
pub struct Compositor {
    /// User data.
    pub data: Box<Any>,
    /// Internal compositor handler
    compositor_handler: Option<Box<InternalCompositor>>,
    /// Manager for the inputs.
    input_manager: Option<Box<InputManager>>,
    /// Manager for the outputs.
    output_manager: Option<Box<OutputManager>>,
    /// Manager for stable XDG shells.
    xdg_shell_manager: Option<Box<XdgShellManager>>,
    /// Manager for XDG shells v6.
    xdg_v6_shell_manager: Option<Box<XdgV6ShellManager>>,
    /// Manager for layer shells.
    layer_shell_manager: Option<Box<LayerShellManager>>,
    /// Pointer to the xdg_shell global.
    /// If xdg_shell_manager is `None`, this value will be `NULL`.
    xdg_shell_global: *mut wlr_xdg_shell,
    /// Pointer to the xdg_shell_v6 global.
    /// If xdg_v6_shell_manager is `None`, this value will be `NULL`.
    xdg_v6_shell_global: *mut wlr_xdg_shell_v6,
    /// Pointer to the layer_shell global.
    /// If layer_shell_manager is `None`, this value will be `NULL`.
    layer_shell_global: *mut wlr_layer_shell,
    /// Pointer to the wlr_compositor.
    compositor: *mut wlr_compositor,
    /// Pointer to the wlroots backend in use.
    backend: *mut wlr_backend,
    /// Pointer to the wayland display.
    display: *mut wl_display,
    /// Pointer to the event loop.
    event_loop: *mut wl_event_loop,
    /// Shared memory buffer file descriptor.
    shm_fd: i32,
    /// Name of the Wayland socket that we are binding to.
    socket_name: String,
    /// Optional decoration manager extension.
    pub server_decoration_manager: Option<ServerDecorationManager>,
    /// The renderer used to draw things to the screen.
    pub renderer: Option<GenericRenderer>,
    /// XWayland server, only Some if it is enabled
    pub xwayland: Option<XWaylandServer>,
    /// The DnD manager
    data_device_manager: Option<DataDeviceManager>,
    /// The error from the panic, if there was one.
    panic_error: Option<Box<Any + Send>>,
    /// Custom function to run at shutdown (or when a panic occurs).
    user_terminate: Option<fn()>,
    /// Lock used to borrow the compositor globally.
    /// Should always be set before passing a reference to the compositor
    /// in a callback.
    pub(crate) lock: Rc<Cell<bool>>
}

pub struct CompositorBuilder {
    compositor_handler: Option<Box<CompositorHandler>>,
    input_manager_handler: Option<Box<InputManagerHandler>>,
    output_manager_handler: Option<Box<OutputManagerHandler>>,
    xdg_shell_manager_handler: Option<Box<XdgShellManagerHandler>>,
    xdg_v6_shell_manager_handler: Option<Box<XdgV6ShellManagerHandler>>,
    layer_shell_manager_handler: Option<Box<LayerShellManagerHandler>>,
    gles2: bool,
    server_decoration_manager: bool,
    data_device_manager: bool,
    xwayland: Option<Box<XWaylandManagerHandler>>,
    user_terminate: Option<fn()>
}

impl CompositorBuilder {
    /// Make a new compositor builder.
    ///
    /// Unless otherwise noted, each option is `false`/`None`.
    pub fn new() -> Self {
        CompositorBuilder { gles2: false,
                            server_decoration_manager: false,
                            data_device_manager: false,
                            compositor_handler: None,
                            input_manager_handler: None,
                            output_manager_handler: None,
                            xdg_shell_manager_handler: None,
                            xdg_v6_shell_manager_handler: None,
                            layer_shell_manager_handler: None,
                            xwayland: None,
                            user_terminate: None }
    }

    /// Set the handler for global compositor callbacks.
    pub fn compositor_handler(mut self, compositor_handler: Box<CompositorHandler>) -> Self {
        self.compositor_handler = Some(compositor_handler);
        self
    }

    /// Set the handler for inputs.
    pub fn input_manager(mut self, input_manager_handler: Box<InputManagerHandler>) -> Self {
        self.input_manager_handler = Some(input_manager_handler);
        self
    }

    /// Set the handler for outputs.
    pub fn output_manager(mut self, output_manager_handler: Box<OutputManagerHandler>) -> Self {
        self.output_manager_handler = Some(output_manager_handler);
        self
    }

    pub fn xdg_shell_manager(mut self,
                             xdg_shell_manager_handler: Box<XdgShellManagerHandler>)
                             -> Self {
        self.xdg_shell_manager_handler = Some(xdg_shell_manager_handler);
        self
    }

    /// Set the handler for xdg v6 shells.
    pub fn xdg_shell_v6_manager(mut self,
                                xdg_v6_shell_manager_handler: Box<XdgV6ShellManagerHandler>)
                                -> Self {
        self.xdg_v6_shell_manager_handler = Some(xdg_v6_shell_manager_handler);
        self
    }

    /// Set the handler for the layer shells.
    pub fn layer_shell_manager(mut self,
                               layer_shell_manager_handler: Box<LayerShellManagerHandler>)
                               -> Self {
        self.layer_shell_manager_handler = Some(layer_shell_manager_handler);
        self
    }

    /// Decide whether or not to enable the data device manager.
    ///
    /// This is used to do DnD, or "drag 'n drop" copy paste.
    pub fn data_device(mut self, data_device_manager: bool) -> Self {
        self.data_device_manager = data_device_manager;
        self
    }

    /// Decide whether or not to enable the GLES2 extension.
    pub fn gles2(mut self, gles2_renderer: bool) -> Self {
        self.gles2 = gles2_renderer;
        self
    }

    /// Decide whether or not to enable the server decoration manager protocol
    /// extension.
    pub fn server_decoration_manager(mut self, server_decoration_manager: bool) -> Self {
        self.server_decoration_manager = server_decoration_manager;
        self
    }

    /// Add a handler for xwayland.
    ///
    /// If you do not provide a handler then the xwayland server does not run.
    pub fn xwayland(mut self, xwayland: Box<XWaylandManagerHandler>) -> Self {
        self.xwayland = Some(xwayland);
        self
    }

    /// Add a custom function to run when shutting down the compositor
    /// or whenever a function in a callback panics.
    pub fn custom_terminate(mut self, terminate: fn()) -> Self {
        self.user_terminate = Some(terminate);
        self
    }

    /// Makes a new compositor that handles the setup of the graphical backend
    /// (e.g, Wayland, X11, or DRM).
    ///
    /// Also automatically opens the socket for clients to communicate to the
    /// compositor with.
    pub fn build_auto<D>(self, data: D) -> Compositor
        where D: Any + 'static
    {
        unsafe {
            let display =
                ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_create,) as *mut wl_display;
            let event_loop =
                ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_get_event_loop, display);
            // TODO Make optional
            let backend = wlr_backend_autocreate(display as *mut _, None);
            if backend.is_null() {
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Could not auto-create backend");
            }
            // Set up shared memory buffer for Wayland clients.
            let shm_fd = wl_display_init_shm(display as *mut _);
            // Create optional extensions.
            let server_decoration_manager = if self.server_decoration_manager {
                ServerDecorationManager::new(display)
            } else {
                None
            };
            let data_device_manager = if self.data_device_manager {
                DataDeviceManager::new(display as _)
            } else {
                None
            };
            let compositor;
            let renderer = if self.gles2 {
                let gles2 = GenericRenderer::gles2_renderer(backend);
                // Set up wlr_compositor
                let gles2_ptr = gles2.as_ptr();
                compositor = wlr_compositor_create(display as *mut _, gles2_ptr);
                Some(gles2)
            } else {
                compositor = wlr_compositor_create(display as *mut _, ptr::null_mut());
                None
            };

            // Set up compositor handler, if the user provided it.
            let compositor_handler = self.compositor_handler.or_else(|| Some(Box::new(())));
            let compositor_handler = compositor_handler.map(|handler| {
                let mut compositor_handler = InternalCompositor::new(handler);
                wl_signal_add(&mut (*compositor).events.new_surface as *mut _ as _,
                              compositor_handler.new_surface_listener() as *mut _ as _);
                wl_signal_add(&mut (*compositor).events.destroy as *mut _ as _,
                              compositor_handler.shutdown_listener() as *mut _ as _);
                compositor_handler
            });

            // Set up input manager, if the user provided it.
            let input_manager = self.input_manager_handler.map(|handler| {
                let mut input_manager = InputManager::new(handler);
                wl_signal_add(&mut (*backend).events.new_input as *mut _ as _,
                              input_manager.add_listener() as *mut _ as _);
                input_manager
            });

            // Set up output manager, if the user provided it.
            let output_manager = self.output_manager_handler.map(|handler| {
                let mut output_manager = OutputManager::new(handler);
                wl_signal_add(&mut (*backend).events.new_output as *mut _ as _,
                              output_manager.add_listener() as *mut _ as _);
                output_manager
            });

            // Set up the xdg_shell handler and associated Wayland global,
            // if user provided a manager for it.
            let mut xdg_shell_global = ptr::null_mut();
            let xdg_shell_manager = self.xdg_shell_manager_handler.map(|handler| {
                xdg_shell_global = wlr_xdg_shell_create(display as *mut _);
                let mut xdg_shell_manager = XdgShellManager::new(handler);
                wl_signal_add(&mut (*xdg_shell_global).events.new_surface as *mut _ as _,
                              xdg_shell_manager.add_listener() as *mut _ as _);
                xdg_shell_manager
            });

            // Set up the xdg_shell_v6 handler and associated Wayland global,
            // if user provided a manager for it.
            let mut xdg_v6_shell_global = ptr::null_mut();
            let xdg_v6_shell_manager = self.xdg_v6_shell_manager_handler.map(|handler| {
                xdg_v6_shell_global = wlr_xdg_shell_v6_create(display as *mut _);
                let mut xdg_v6_shell_manager = XdgV6ShellManager::new(handler);
                wl_signal_add(&mut (*xdg_v6_shell_global).events.new_surface as *mut _ as _,
                              xdg_v6_shell_manager.add_listener() as *mut _ as _);
                xdg_v6_shell_manager
            });

            // Set up the layer shell handler and associated Wayland global,
            // if user provided a manager for it.
            let mut layer_shell_global = ptr::null_mut();
            let layer_shell_manager = self.layer_shell_manager_handler.map(|handler| {
                layer_shell_global = wlr_layer_shell_create(display as *mut _);
                let mut layer_shell_manager = LayerShellManager::new((vec![], handler));
                wl_signal_add(&mut (*layer_shell_global).events.new_surface as *mut _ as _,
                              layer_shell_manager.add_listener() as *mut _ as _);
                layer_shell_manager
            });

            // Set up the XWayland server, if the user wants it.
            let xwayland = self.xwayland.and_then(|manager| {
                                                      Some(XWaylandServer::new(display as _,
                                                                               compositor,
                                                                               manager,
                                                                               false))
                                                  });

            let user_terminate = self.user_terminate;

            // Open the socket to the Wayland server.
            let socket = ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_add_socket_auto, display);
            if socket.is_null() {
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Unable to open wayland socket");
            }
            let socket_name = CStr::from_ptr(socket).to_string_lossy().into_owned();
            wlr_log!(L_DEBUG,
                     "Running compositor on wayland display {}",
                     socket_name);
            env::set_var("_WAYLAND_DISPLAY", socket_name.clone());
            let compositor = Compositor { data: Box::new(data),
                                          compositor_handler,
                                          socket_name,
                                          input_manager,
                                          output_manager,
                                          xdg_shell_manager,
                                          xdg_shell_global,
                                          xdg_v6_shell_manager,
                                          xdg_v6_shell_global,
                                          layer_shell_manager,
                                          layer_shell_global,
                                          data_device_manager,
                                          compositor,
                                          backend,
                                          display,
                                          event_loop,
                                          shm_fd,
                                          server_decoration_manager,
                                          renderer,
                                          xwayland,
                                          user_terminate,
                                          panic_error: None,
                                          lock: Rc::new(Cell::new(false)) };
            compositor.set_lock(true);
            compositor
        }
    }
}

impl Compositor {
    /// Creates a weak reference to the `Compositor`.
    pub fn weak_reference(&self) -> CompositorHandle {
        let handle = Rc::downgrade(&self.lock);
        CompositorHandle { handle }
    }

    /// Enters the wayland event loop. Won't return until the compositor is
    /// shut off
    pub fn run(self) {
        self.run_with(|_| unsafe {
                          ffi_dispatch!(WAYLAND_SERVER_HANDLE,
                                        wl_display_run,
                                        (*COMPOSITOR_PTR).display);
                      })
    }

    /// Prepare to enter the wayland event loop. Instead of calling
    /// `wl_display_run`, the provided callback function is invoked. Allows
    /// integration with a different event loop.
    pub fn run_with<F>(self, runner: F)
        where F: FnOnce(&Compositor)
    {
        unsafe {
            self.set_lock(false);
            let compositor = UnsafeCell::new(self);
            if COMPOSITOR_PTR != 0 as _ {
                // NOTE Rationale for panicking:
                // * Nicer than an abort
                // * Not yet in C land
                panic!("A compositor is already running!")
            }
            COMPOSITOR_PTR = compositor.get();
            wlr_log!(L_INFO, "Starting compositor");
            if !wlr_backend_start((*compositor.get()).backend) {
                wlr_backend_destroy((*compositor.get()).backend);
                // NOTE Rationale for panicking:
                // * Won't be in C land just yet, so it's safe to panic
                // * Can always be returned in a Result instead, but for now
                //   if you auto create it's assumed you can't recover.
                panic!("Failed to start backend");
            }
            env::set_var("WAYLAND_DISPLAY", (*COMPOSITOR_PTR).socket_name.clone());
            runner(&*COMPOSITOR_PTR);
            match (*compositor.get()).panic_error.take() {
                None => {}
                Some(err) => {
                    // A panic occured, now we can re-throw it safely.
                    ::std::panic::resume_unwind(err)
                }
            }
        }
    }

    /// Shutdown the wayland server
    fn terminate(&mut self) {
        unsafe {
            ffi_dispatch!(WAYLAND_SERVER_HANDLE, wl_display_terminate, self.display);
        }
    }

    pub unsafe fn display(&self) -> *mut wl_display {
        self.display
    }

    pub unsafe fn event_loop(&self) -> *mut wl_event_loop {
        self.event_loop
    }

    /// Saves the panic error information in the compositor, to be re-thrown
    /// later when we are out of the C callback stack.
    pub(crate) fn save_panic_error(&mut self, error: Box<Any + Send>) {
        self.panic_error = Some(error);
    }

    /// Manually set hte lock used to determine if a double-borrow is occuring on this structure.
    ///
    /// # Panics
    /// Panics when trying to set the lock on an upgraded handle.
    unsafe fn set_lock(&self, val: bool) {
        self.lock.set(val)
    }
}

impl Drop for Compositor {
    fn drop(&mut self) {
        unsafe { wlr_compositor_destroy(self.compositor) }
    }
}

impl CompositorHandle {
    /// Constructs a new `CompositorHandle` that is always invalid. Calling `run` on this
    /// will always fail.
    ///
    /// This is useful for pre-filling a value before it's provided by the server, or
    /// for mocking/testing.
    pub fn new() -> Self {
        CompositorHandle { handle: Weak::new() }
    }

    /// Upgrades the compositor handle to a reference to the backing `Compositor`.
    ///
    /// # Unsafety
    /// To be honest this function is probably safe.
    ///
    /// However, the CompositorHandle will behave like the other handles in order
    /// to reduce confusion.
    unsafe fn upgrade(&self) -> HandleResult<&mut Compositor> {
        self.handle.upgrade()
            .ok_or(HandleErr::AlreadyDropped)
            // NOTE
            // We drop the Rc here because having two would allow a dangling
            // pointer to exist!
            .and_then(|check| {
                if check.get() {
                    return Err(HandleErr::AlreadyBorrowed)
                }
                if COMPOSITOR_PTR.is_null() {
                    return Err(HandleErr::AlreadyDropped)
                }
                check.set(true);
                Ok(&mut *COMPOSITOR_PTR)
            })
    }

    /// Run a function on the referenced `Compositor`, if it still exists.
    ///
    /// Returns the result of the function, if successful.
    ///
    /// # Safety
    /// By enforcing a rather harsh limit on the lifetime of the Compositor
    /// to a short lived scope of an anonymous function,
    /// this function ensures the Compositor does not live longer
    /// than it exists.
    ///
    /// # Panics
    /// This function will panic if multiple mutable borrows are detected.
    /// This will happen if you call `upgrade` directly within this callback,
    /// or if you run this function within the another run to the same `Output`.
    ///
    /// So don't nest `run` calls and everything will be ok :).
    pub fn run<F, R>(&self, runner: F) -> HandleResult<R>
        where F: FnOnce(&mut Compositor) -> R
    {
        let compositor = unsafe { self.upgrade()? };
        let res = panic::catch_unwind(panic::AssertUnwindSafe(|| runner(compositor)));
        self.handle.upgrade().map(|check| {
                                      // Sanity check that it hasn't been tampered with.
                                      if !check.get() {
                                          wlr_log!(L_ERROR,
                                                   "After running compositor callback, mutable \
                                                    lock was false");
                                          panic!("Compositor lock in incorrect state!");
                                      }
                                      check.set(false)
                                  });
        match res {
            Ok(res) => Ok(res),
            Err(err) => panic::resume_unwind(err)
        }
    }
}

/// Terminates the compositor and execute any user clean up code.
pub fn terminate() {
    unsafe {
        if COMPOSITOR_PTR != 0 as _ {
            let compositor = &mut *COMPOSITOR_PTR;
            compositor.terminate();
            compositor.user_terminate.map(|f| f());
        }
    }
}

/// Gets a handle to the compositor.
///
/// If the compositor has not started running yet, or if it has stopped,
/// then this function will return None.
pub fn compositor_handle() -> Option<CompositorHandle> {
    unsafe {
        if COMPOSITOR_PTR.is_null() {
            None
        } else {
            Some((&mut *COMPOSITOR_PTR).weak_reference())
        }
    }
}
