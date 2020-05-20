#![allow(non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::all)]

pub extern crate libc;
pub extern crate wayland_commons;
pub extern crate wayland_server;
pub extern crate wayland_client;
pub extern crate wayland_sys;

pub use wayland_sys::{
    *, gid_t,
    pid_t,
    server::{self, WAYLAND_SERVER_HANDLE}, uid_t
};

pub use self::generated::root::*;
pub use self::generated::protocols as protocols;

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod generated {
    include!("gen.rs");

    // XXX: If you add another protocols, take a look at wayland_protocol! macro
    // from `wayland-rs/wayland-protocols/src/protocol_macro.rs`.
    pub mod protocols {

        pub mod server_decoration {
            #![allow(unused)]

            mod c_interfaces {
                use wayland_commons::wire::*;
                use wayland_commons::map::*;
                use wayland_commons::smallvec;
                use wayland_server::*;
                use wayland_server::protocol::wl_surface;
                use wayland_server::protocol::wl_seat as wl_seat;
                use wayland_client::AnonymousObject;
                use wayland_sys as sys;

                include!(concat!(env!("OUT_DIR"), "/server_decoration_interfaces.rs"));
            }

            pub mod server {
                #![allow(unused)]

                use wayland_commons::wire::*;
                use wayland_commons::map::*;
                use wayland_commons::smallvec;
                use wayland_server::*;
                use wayland_server::protocol::wl_surface;
                use wayland_server::protocol::wl_seat as wl_seat;
                use wayland_client::AnonymousObject;
                use wayland_sys as sys;

                include!(concat!(env!("OUT_DIR"), "/server_decoration_server_api.rs"));
            }
        }
        pub mod idle {
            mod c_interfaces {
                #![allow(unused)]

                use wayland_commons::wire::*;
                use wayland_commons::map::*;
                use wayland_commons::smallvec;
                use wayland_server::*;
                use wayland_server::protocol::wl_surface;
                use wayland_server::protocol::wl_seat as wl_seat;
                use wayland_client::AnonymousObject;
                use wayland_sys as sys;

                include!(concat!(env!("OUT_DIR"), "/idle_interfaces.rs"));
            }

            pub mod server {
                #![allow(unused)]

                use wayland_commons::wire::*;
                use wayland_commons::map::*;
                use wayland_commons::smallvec;
                use wayland_server::*;
                use wayland_server::protocol::wl_surface;
                use wayland_server::protocol::wl_seat as wl_seat;
                use wayland_client::AnonymousObject;
                use wayland_sys as sys;

                include!(concat!(env!("OUT_DIR"), "/idle_server_api.rs"));
            }
        }

    }
}

#[cfg(feature = "unstable")]
pub type wlr_output_events = self::generated::root::wlr_output__bindgen_ty_1;
#[cfg(feature = "unstable")]
pub type wlr_input_device_pointer = self::generated::root::wlr_input_device__bindgen_ty_1;

pub trait TransformOutput {
    fn invert(self) -> Self;
    fn compose(self, other: Self) -> Self;
}

#[cfg(feature = "unstable")]
impl TransformOutput for generated::root::wl_output_transform {
    /// Returns the transform that, when composed with `self`, gives
    /// `WL_OUTPUT_TRANSFORM_NORMAL`.
    fn invert(self) -> Self {
        unsafe { generated::root::wlr_output_transform_invert(self) }
    }

    /// Returns a transform that, when applied, has the same effect as applying
    /// sequentially `self` and `other`.
    fn compose(self, other: Self) -> Self {
        unsafe { generated::root::wlr_output_transform_compose(self, other) }
    }
}
