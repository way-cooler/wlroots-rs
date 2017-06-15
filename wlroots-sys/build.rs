extern crate bindgen;
#[cfg(feature = "static")]
extern crate cmake;
extern crate gl_generator;

use gl_generator::{Registry, Api, Profile, Fallbacks, StaticGenerator};
use std::env;
use std::fs::File;
use std::path::Path;

static LIBRARIES: &'static [&'static str] =
    &["wlr-common", "wlr-backend", "wlr-session", "wlr-types"];

fn main() {
    let generated = bindgen::builder()
        .header("src/wlroots.h")
        .whitelisted_type(r"^wlr_.*$")
        .whitelisted_function(r"^wlr_.*$")
        .no_unstable_rust()
        .ctypes_prefix("libc")
        .clang_arg("-I")
        .clang_arg("wlroots/include")
        .generate().unwrap();

    if cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=dylib=wayland-server");
        println!("cargo:rustc-link-lib=dylib=EGL");
        println!("cargo:rustc-link-lib=dylib=GL");
        println!("cargo:rustc-link-lib=dylib=gbm");
        println!("cargo:rustc-link-lib=dylib=drm");
        println!("cargo:rustc-link-lib=dylib=input");
        println!("cargo:rustc-link-lib=dylib=udev");
        println!("cargo:rustc-link-lib=dylib=systemd");
        println!("cargo:rustc-link-lib=dylib=dbus-1");
    } else {
        for library in LIBRARIES {
            println!("cargo:rustc-link-lib=dylib={}", library);
        }
    }

    // generate the bindings
    generated.write_to_file("src/gen.rs").unwrap();

    cmake();

    // Example Khronos building stuff.
    // TODO Put behind feature flag.
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("bindings.rs")).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();

}

#[cfg(not(feature = "static"))]
fn cmake() {}

#[cfg(feature = "static")]
fn cmake() {
    use cmake::Config;

    let dst = Config::new("wlroots")
                // TODO Eventually change to Release, once the warnings stop
                .define("CMAKE_BUILD_TYPE", "Debug")
                // TODO Remove "all" once "install" is valid
                .build_target("all")
                .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    println!("cargo:rustc-link-search=native={}/build/", dst.display());
    // TODO May not be needed to specify the directiories directly,
    // wait until the library output stabilizes and look into it later
    println!("cargo:rustc-link-search=native={}/build/types", dst.display());
    println!("cargo:rustc-link-search=native={}/build/session", dst.display());
    println!("cargo:rustc-link-search=native={}/build/common", dst.display());
    println!("cargo:rustc-link-search=native={}/build/wayland", dst.display());
    println!("cargo:rustc-link-search=native={}/build/backend", dst.display());

    for library in LIBRARIES {
        println!("cargo:rustc-link-lib=static={}", library);
    }
}
