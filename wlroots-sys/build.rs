extern crate bindgen;
#[cfg(feature = "static")]
extern crate meson;
extern crate pkg_config;
extern crate wayland_scanner;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, io};
use bindgen::EnumVariation;

fn main() {
    println!("cargo:rerun-if-changed=src/gen.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/wlroots.h");
    println!("cargo:rerun-if-changed=wlroots");

    meson();
    let protocol_header_path =
        generate_protocol_headers().expect("Could not generate header files for wayland protocols");
    let target_dir = env::var("OUT_DIR").expect("$OUT_DIR not set!");
    let mut builder = bindgen::builder()
        .derive_debug(true)
        .derive_default(true)
        .generate_comments(true)
        .header("src/wlroots.h")
        .whitelist_type(r"^wlr_.*$")
        .whitelist_type(r"^xkb_.*$")
        .whitelist_type(r"^XKB_.*$")
        .whitelist_function(r"^_?pixman_.*$")
        .whitelist_function(r"^_?wlr_.*$")
        .whitelist_function(r"^xkb_.*$")
        .ctypes_prefix("libc")
        .clang_arg("-Iwlroots/include")
        .clang_arg("-Iwlroots/include/wlr")

        // NOTE Necessary because they use the out directory to put
        // pragma information on what features are available in a header file
        // titled "config.h"
        .clang_arg(format!("-I{}{}", target_dir, "/include/"))
        .clang_arg(format!("-I{}", protocol_header_path.to_str().unwrap()))
        .clang_arg("-Iwlroots/include/xcursor")
        .clang_arg("-I/usr/include/pixman-1")
        // Work around bug https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .blacklist_type("FP_NAN")
        .blacklist_type("FP_INFINITE")
        .blacklist_type("FP_ZERO")
        .blacklist_type("FP_SUBNORMAL")
        .blacklist_type("FP_NORMAL")
        .default_enum_style(EnumVariation::Rust { non_exhaustive: false })
        .enable_function_attribute_detection()
        .enable_cxx_namespaces()
        .size_t_is_usize(true);

    if cfg!(feature = "unstable") {
        builder = builder.clang_arg("-DWLR_USE_UNSTABLE");
    }

    // config.h won't exist, so make a dummy file.
    // We don't need it because of the following -D defines.
    fs::create_dir_all(format!("{}{}", target_dir, "/include/wlr/"))
        .expect("Could not create <out>/include/wlr");
    fs::File::create(format!("{}{}", target_dir, "/include/wlr/config.h"))
        .expect("Could not create dummy config.h file");

    if !cfg!(feature = "static") {
        // meson automatically sets up variables, but if we are linking
        // dynamically bindgen will no longer have them.
        builder = builder.clang_args(
            [
                format!("-DWLR_HAS_LIBCAP={}", cfg!(feature = "libcap") as u8),
                format!("-DWLR_HAS_SYSTEMD={}", cfg!(feature = "systemd") as u8),
                format!("-DWLR_HAS_ELOGIND={}", cfg!(feature = "elogind") as u8),
                format!("-DWLR_HAS_X11_BACKEND={}", cfg!(feature = "x11_backend") as u8),
                format!("-DWLR_HAS_XWAYLAND={}", cfg!(feature = "xwayland") as u8),
                format!("-DWLR_HAS_XCB_ERRORS={}", cfg!(feature = "xcb_errors") as u8),
                format!("-DWLR_HAS_XCB_ICCCM={}", cfg!(feature = "xcb_icccm") as u8)
            ]
            .iter()
        )
    }
    
    let generated = builder.generate().unwrap();

    println!("cargo:rustc-link-lib=dylib=X11");
    println!("cargo:rustc-link-lib=dylib=X11-xcb");
    println!("cargo:rustc-link-lib=dylib=xkbcommon");
    println!("cargo:rustc-link-lib=dylib=xcb");
    println!("cargo:rustc-link-lib=dylib=xcb-composite");
    println!("cargo:rustc-link-lib=dylib=xcb-xfixes");
    println!("cargo:rustc-link-lib=dylib=xcb-image");
    println!("cargo:rustc-link-lib=dylib=xcb-render");
    println!("cargo:rustc-link-lib=dylib=xcb-shm");
    println!("cargo:rustc-link-lib=dylib=xcb-icccm");
    println!("cargo:rustc-link-lib=dylib=xcb-xkb");
    println!("cargo:rustc-link-lib=dylib=xcb-xinput");
    println!("cargo:rustc-link-lib=dylib=wayland-egl");
    println!("cargo:rustc-link-lib=dylib=wayland-client");
    println!("cargo:rustc-link-lib=dylib=wayland-server");
    println!("cargo:rustc-link-lib=dylib=EGL");
    println!("cargo:rustc-link-lib=dylib=GL");
    println!("cargo:rustc-link-lib=dylib=gbm");
    println!("cargo:rustc-link-lib=dylib=drm");
    println!("cargo:rustc-link-lib=dylib=input");
    println!("cargo:rustc-link-lib=dylib=udev");
    println!("cargo:rustc-link-lib=dylib=dbus-1");
    println!("cargo:rustc-link-lib=dylib=pixman-1");

    link_optional_libs();

    if !cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=dylib=wlroots");
        println!("cargo:rustc-link-search=native=/usr/local/lib");
    }

    // generate the bindings
    generated.write_to_file("src/gen.rs").unwrap();

    generate_protocols();
}

#[cfg(not(feature = "static"))]
fn meson() {}

#[cfg(feature = "static")]
fn meson() {
    if !Path::new("wlroots").exists(){
        panic!("The `wlroots` submodule does not exist");
    }

    let build_path = PathBuf::from(env::var("OUT_DIR")
        .expect("Could not get OUT_DIR env variable"))
        .join("build");

    let build_path_str = build_path
        .to_str()
        .expect("Could not turn build path into a string");
    
    println!("cargo:rustc-link-search=native=wlroots");
    println!("cargo:rustc-link-search=native={}/lib", build_path_str);
    println!("cargo:rustc-link-search=native={}/lib64", build_path_str);
    println!("cargo:rustc-link-search=native={}/build/", build_path_str);

    let mut meson_config_status = None;

    if cfg!(feature = "static") {
        println!("cargo:rustc-link-lib=static=wlroots");
        println!("cargo:rustc-link-search=native={}/", build_path_str);

        meson_config_status = Some(
            Command::new("meson")
                .current_dir("wlroots")
                .arg(".")
                .arg(build_path_str)
                .arg("-Ddefault_library=static")
                .spawn()
                .expect("Static compilation failed: Is meson installed?")
                .wait()
        );
    }

    match meson_config_status
    {
        None | Some(Ok(_)) => meson::build("wlroots", build_path_str),
        Some(Err(exit_status)) => println!(
            "Static compilation failed: Meson configuration failed with {}",
            exit_status
        )
    }
}

/// Gets the unstable and stable protocols in /usr/share-wayland-protocols and
/// generates server headers for them.
///
/// The path to the folder with the generated headers is returned. It will
/// have two directories, `stable`, and `unstable`.
fn generate_protocol_headers() -> io::Result<PathBuf> {
    let output_dir_str = env::var("OUT_DIR").unwrap();
    let out_path: PathBuf = format!("{}/wayland-protocols", output_dir_str).into();
    fs::create_dir(&out_path).ok();
    let protocols_prefix = pkg_config::get_variable("wayland-protocols", "prefix").unwrap();
    let protocols = fs::read_dir(format!("{}/share/wayland-protocols/stable", protocols_prefix))?.chain(
        fs::read_dir(format!("{}/share/wayland-protocols/unstable", protocols_prefix))?
    );
    for entry in protocols {
        let entry = entry?;
        for entry in fs::read_dir(entry.path())? {
            let entry = entry?;
            let path = entry.path();
            let mut filename = entry.file_name().into_string().unwrap();
            if filename.ends_with(".xml") {
                let new_length = filename.len() - 4;
                filename.truncate(new_length);
            }
            filename.push_str("-protocol");
            Command::new("wayland-scanner")
                .arg("server-header")
                .arg(path.clone())
                .arg(format!("{}/{}.h", out_path.to_str().unwrap(), filename))
                .status()
                .unwrap();
        }
    }
    Ok(out_path)
}

fn generate_protocols() {
    let output_dir_str = env::var("OUT_DIR").unwrap();

    let output_dir = Path::new(&output_dir_str);

    let protocols = &[
        ("./wlroots/protocol/server-decoration.xml", "server_decoration"),
        (
            "./wlroots/protocol/wlr-gamma-control-unstable-v1.xml",
            "gamma_control"
        ),
        ("./wlroots/protocol/wlr-screencopy-unstable-v1.xml", "screencopy"),
        ("./wlroots/protocol/idle.xml", "idle")
    ];

    for protocol in protocols {
        wayland_scanner::generate_code(
            protocol.0,
            output_dir.join(format!("{}_server_api.rs", protocol.1)),
            wayland_scanner::Side::Server
        );
        wayland_scanner::generate_code(
            protocol.0,
            output_dir.join(format!("{}_client_api.rs", protocol.1)),
            wayland_scanner::Side::Client
        );
        wayland_scanner::generate_code(
            protocol.0,
            output_dir.join(format!("{}_interfaces.rs", protocol.1)),
            wayland_scanner::Side::Server
        );
    }
}

fn link_optional_libs() {
    if cfg!(feature = "libcap") && pkg_config::probe_library("libcap").is_ok() {
        println!("cargo:rustc-link-lib=dylib=cap");
    }
    if cfg!(feature = "systemd") && pkg_config::probe_library("libsystemd").is_ok() {
        println!("cargo:rustc-link-lib=dylib=systemd");
    }
    if cfg!(feature = "elogind") && pkg_config::probe_library("elogind").is_ok() {
        println!("cargo:rustc-link-lib=dylib=elogind");
    }
    if pkg_config::probe_library("xcb-errors").is_ok() {
        println!("cargo:rustc-link-lib=dylib=xcb-errors");
    }
}
