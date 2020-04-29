extern crate bindgen;
extern crate llvm_config;
#[cfg(feature = "static")]
extern crate meson;
extern crate pkg_config;
extern crate wayland_scanner;

#[cfg(feature = "static")]
extern crate expat_sys;


use std::path::{Path, PathBuf};
use std::process::{Command, exit};
use std::{env, fs, io};

fn main() {

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/wlroots.h");
    println!("cargo:rerun-if-changed=wlroots");


    find_pkg_config_clang();

    #[cfg(feature = "unstable")]
    package_error_unstable();


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
        // Work around bug https://github.com/rust-lang-nursery/rust-bindgen/issues/687
        .blacklist_type("FP_NAN")
        .blacklist_type("FP_INFINITE")
        .blacklist_type("FP_ZERO")
        .blacklist_type("FP_SUBNORMAL")
        .blacklist_type("FP_NORMAL");

    if cfg!(feature = "pixman") || cfg!(feature = "unstable") {
        builder = builder.whitelist_function(r"^_?pixman_.*$");
        builder = builder.clang_arg("-I/usr/include/pixman-1");
        builder = builder.clang_arg("-DWLR_USE_PIXMAN");
    }

    if cfg!(feature = "unstable") {
        builder = builder.clang_arg("-DWLR_USE_UNSTABLE");
    }

    if !cfg!(feature = "static") {
        // config.h won't exist, so make a dummy file.
        // We don't need it because of the following -D defines.
        fs::create_dir_all(format!("{}{}", target_dir, "/include/wlr/"))
            .expect("Could not create <out>/include/wlr");
        fs::File::create(format!("{}{}", target_dir, "/include/wlr/config.h"))
            .expect("Could not create dummy config.h file");
        // meson automatically sets up variables, but if we are linking
        // dynamically bindgen will no longer have them.
        builder = builder.clang_args(
            [
                format!("-DWLR_HAS_LIBCAP={}", cfg!(feature = "libcap") as u8),
                format!("-DWLR_HAS_SYSTEMD={}", cfg!(feature = "systemd") as u8),
                format!("-DWLR_HAS_ELOGIND={}", cfg!(feature = "elogind") as u8),
                format!(
                    "-DWLR_HAS_X11_BACKEND={}",
                    cfg!(feature = "x11_backend") as u8
                ),
                format!("-DWLR_HAS_XWAYLAND={}", cfg!(feature = "xwayland") as u8),
                format!(
                    "-DWLR_HAS_XCB_ERRORS={}",
                    cfg!(feature = "xcb_errors") as u8
                ),
                format!("-DWLR_HAS_XCB_ICCCM={}", cfg!(feature = "xcb_icccm") as u8),
            ]
                .iter(),
        )
    }
    let generated = builder.generate().expect(package_error_common_unstable().to_string().as_ref());

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
    #[cfg(feature = "pixman")]
    println!("cargo:rustc-link-lib=dylib=pixman-1");
    #[cfg(feature = "unstable")]
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
/// prints helpful package installation instructions or error to the user.
fn package_error(command: String) -> String {
    if check_version("wayland-protocols","--version","0",true)
        && "wayland-protocols" == command
    {
        println!("wayland-protocols found");
    } else {
        println!("WRONG version of wayland-protocols or not installed.");

        println!("\nInstallation instructions, install with packet manager or:");
        println!("git clone https://github.com/wayland-project/wayland-protocols.git");
    }

    return "".to_string();
}

fn package_error_common_unstable() -> String {

    if cfg!(feature = "unstable") {
        println!("Are the following unstable dependencies installed?");
        println!("libwayland-dev");
        println!("libudev-dev");
        println!("libgles2-mesa-dev");
        println!("libpixman-1-dev");
        println!("llvm");
        println!("libxkbcommon-dev");
        println!("libxkbcommon-dev");
        println!("libinput-dev");
        println!("libinput-bin");
    }

    return "".to_string();
}


// STATIC BUILD CHECK
#[cfg(feature = "static")]
fn package_error_static() {
    if check_version("pkg-config","--version","0",false)
    {
        println!("pkg-config found");
    } else {
        println!("WRONG version of pkg-config or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        println!("Installation cannot rely on Cargo pkg-config ");
        exit(2);
    }

    if check_version("ninja","--version","1.9.0",true)
    {
        println!("ninja found");
    } else {
        println!("WRONG version of ninja or not installed on system.");

        println!("\nInstallation instructions, install with packet manager or python3-pip if available");
        exit(2);
    }

    if check_version("meson","--version","0.54.0",true)
    {
        println!("meson found");
    } else {
        println!("WRONG version of meson or not installed on system.");

        println!("\nInstallation instructions, install with packet manager or python3-pip if available");
        println!("STATIC installation cannot rely on Cargo meson ");
        exit(2);
    }

    if check_version("cmake","--version","3.0",true)
    {
        println!("cmake found");
    } else {
        println!("WRONG version of cmake or not installed on system.");

        println!("\nInstallation instructions, install with packet manager or if available");
        println!("STATIC installation cannot rely on Cargo cmake ");
        exit(2);
    }

    if check_version("clang","--version","6.0",true)
    {
        println!("clang found");
    } else {
        println!("WRONG version of clang or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        println!("STATIC installation cannot rely on Cargo clang ");
        exit(2);
    }

    if check_version("pip3","--version","9.0",true)
    {
        println!("pip3 found");
    } else {
        println!("WRONG version of pip3 or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        exit(2);
    }

    if check_version("xml2-config","--version","0.0",true)
    {
        println!("libxml2-dev found");
    } else {
        println!("WRONG version of libxml2-dev or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        exit(2);
    }

    if check_version("dot","-V","0.0",true)
    {
        println!("graphviz found");
    } else {
        println!("WRONG version of graphviz or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        exit(2);
    }


    if check_version("wayland-scanner","--version","0.0",false)
    {
        println!("libwayland-bin found");
    } else {
        println!("WRONG version of libwayland-bin or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        exit(2);
    }

    println!("other dependencies needed: ");
    println!("libxml2: ");
    println!("libwayland-bin: ");
    println!("wayland-protocols ");
    println!("libffi-dev: ");
    println!("expat ");
    println!("graphviz: ");
    println!("doxygen: ");
    println!("llvm ");
    println!("libwayland-bin: ")

}

/// Checks if a specific package is installed in system PATH or pkg-config
/// if no min_version is needed use "0" as arg.
fn check_version(command: &str, arg: &str, min_version: &str, first_check: bool) -> bool {

    if first_check {
        if let Ok(_lib_details) = pkg_config::Config::new()
            .atleast_version(&min_version.clone())
            .probe(&command.clone())
        {
            if min_version == "0".to_string() {
                println!("{:?} was found located in pkg-config", command);
            } else {
                println!(
                    "{:?} min version {:?} was not found located in pkg-config",
                    command, min_version
                );
            }

            return true;
        } else {
            if min_version == "0".to_string() {
                println!("{:?} was not found located in pkg-config", command);
            } else {
                println!(
                    "{:?} min version {:?} was found located in pkg-config",
                    command, min_version
                );
            }

            println!("if it is installed, try export PKG_CONFIG_PATH=/usr/lib/PATH_TO_PC/lib/:$PKG_CONFIG_PATH where .pc file is located");
        }
    }
    //let mut minvec = Vec::new();
    //let mut command_vector = Vec::new();

    let minvec = min_version
        .split(|c| c == ' ' || c == '.')
        .filter_map(|s| s.parse::<i32>().ok())
        .collect::<Vec<_>>();

    return if !is_in_path("PATH".to_string(), command) {
        println!("\n{:?} was not found in PATH, try export it by:", command);
        println!("export PATH=/usr/PATH_TO_BIN/bin/:$PATH \n");
        false
    } else {
        let output = Command::new(command.clone())
            .arg(arg)
            .output()
            .unwrap_or_else(|e| panic!("failed to execute process: {}", e));

        if output.status.success() {
            let mut command_output = String::from_utf8_lossy(&output.stdout);

            let mut command_vector = command_output
                .split(|c| c == ' ' || c == '.')
                .filter_map(|s| s.parse::<i32>().ok())
                .collect::<Vec<_>>();

            if command_vector.is_empty() {
                command_output = String::from_utf8_lossy(&output.stderr);
                command_vector = command_output
                    .split(|c| c == ' ' || c == '.')
                    .filter_map(|s| s.parse::<i32>().ok())
                    .collect::<Vec<_>>();
                if command_vector.is_empty() {
                    println!("Unable to get version number with --version");
                    return false;
                }
            }

            while command_vector.len() > minvec.len() {
                command_vector.pop();
            }

            while minvec.len() > command_vector.len() {
                command_vector.push(0);
            }

            //compares version vector and return true if larger
            let mut counter_compare = 1;
            for (comval, minval) in command_vector.iter().zip(minvec.iter()) {
                if counter_compare < minvec.len() {
                    if comval > minval {
                        println!("local installation of {:?} {:?} was found, >= version {:?} was not found located in pkg-config", command, command_vector, min_version);
                        return true;
                    }
                    if comval < minval {
                        return false;
                    }
                    //else equal continue....
                } else {
                    return if comval >= minval { true } else { false };
                }
                counter_compare += 1;
            }
        } else {
            let s = String::from_utf8_lossy(&output.stderr);
            print!("rustc failed and stderr was:\n{}", s);
        }

        false
    };
}

fn find_pkg_config_clang(){
    if check_version("pkg-config","--version","0",false)
    {
        println!("pkg-config found");
    } else {
        println!("WRONG version of pkg-config or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        println!("Installation cannot rely on Cargo pkg-config ");
        exit(2);
    }

    if check_version("clang","--version","0.0",true)
    {
        println!("clang found");
    } else {
        println!("WRONG version of clang or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        println!("STATIC installation cannot rely on Cargo clang ");
        exit(2);
    }

}



#[cfg(feature = "unstable")]
fn package_error_unstable() {

    find_pkg_config_clang();

    if check_version("clang","--version","0",true)
    {
        println!("clang found");
    } else {
        println!("WRONG version of clang or not installed on system.");

        println!("\nInstallation instructions, install with packet manager");
        println!("STATIC installation cannot rely on Cargo clang ");
        exit(2);
    }
}

///help method to locate package in PATH or other env variable
fn is_in_path(path_dir: String, command: &str) -> bool {
    //                      PATH
    match env::var_os(path_dir) {
        Some(paths) => {
            for path in env::split_paths(&paths) {
                for entry in fs::read_dir(path).expect("Error reading directory") {
                    let entry = entry.expect("Could not read entry in directory");
                    let file_name = entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();

                    if file_name == command.to_string() {
                        println!("var {:?} was found in {:?}", command, entry);
                        return true;
                    }
                }
            }
        }
        None => println!("directory is not defined."),
    }

    return false;
}

#[cfg(not(feature = "static"))]
fn meson() {}

#[cfg(feature = "static")]
fn meson() {
    let build_path =
        PathBuf::from(env::var("OUT_DIR").expect("Could not get OUT_DIR env variable"));
    build_path.join("build");
    let build_path_str = build_path
        .to_str()
        .expect("Could not turn build path into a string");
    println!("cargo:rustc-link-search=native=wlroots");
    println!("cargo:rustc-link-search=native={}/lib", build_path_str);
    println!("cargo:rustc-link-search=native={}/lib64", build_path_str);
    println!("cargo:rustc-link-search=native={}/build/", build_path_str);
    if cfg!(feature = "static") {
        println!("cargo:rustc-link-search=native={}/util/", build_path_str);
        println!("cargo:rustc-link-search=native={}/types/", build_path_str);
        println!(
            "cargo:rustc-link-search=native={}/protocol/",
            build_path_str
        );
        println!("cargo:rustc-link-search=native={}/xcursor/", build_path_str);
        println!(
            "cargo:rustc-link-search=native={}/xwayland/",
            build_path_str
        );
        println!("cargo:rustc-link-search=native={}/backend/", build_path_str);
        println!(
            "cargo:rustc-link-search=native={}/backend/x11",
            build_path_str
        );
        println!("cargo:rustc-link-search=native={}/render/", build_path_str);

        //below not used in wlroots 0.10.0 only in older versions
        //println!("cargo:rustc-link-lib=static=wlr_util");
        //println!("cargo:rustc-link-lib=static=wlr_types");
        //println!("cargo:rustc-link-lib=static=wlr_xcursor");
        //println!("cargo:rustc-link-lib=static=wlr_xwayland");
        //println!("cargo:rustc-link-lib=static=wlr_backend");
        //println!("cargo:rustc-link-lib=static=wlr_backend_x11");
        //println!("cargo:rustc-link-lib=static=wlr_render");
        //println!("cargo:rustc-link-lib=static=wl_protos");
    }

    package_error_static();

    if Path::new("wayland").exists() {
        meson::build("wayland", build_path_str);
    } else {
        panic!("The `wayland` submodule does not exist");
    }

    if Path::new("wlroots").exists() {
        meson::build("wlroots", build_path_str);
    } else {
        panic!("The `wlroots` submodule does not exist");
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
    let protocols_prefix = pkg_config::get_variable("wayland-protocols", "prefix")
        .expect(package_error("wayland-protocols".to_string()).as_ref());
    let protocols = fs::read_dir(format!(
        "{}/share/wayland-protocols/stable",
        protocols_prefix
    ))?
        .chain(fs::read_dir(format!(
            "{}/share/wayland-protocols/unstable",
            protocols_prefix
        ))?);
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
                .expect("\nerror with package libwayland-bin\n");
        }
    }
    Ok(out_path)
}

fn generate_protocols() {
    let output_dir_str = env::var("OUT_DIR").unwrap();

    let output_dir = Path::new(&output_dir_str);

    let protocols = &[
        ("./wlroots/protocol/server-decoration.xml","server_decoration",),
        ("./wlroots/protocol/wlr-gamma-control-unstable-v1.xml","gamma_control",),
        ("./wlroots/protocol/wlr-screencopy-unstable-v1.xml","screencopy",),
        ("./wlroots/protocol/gtk-primary-selection.xml", "gtk_primary_selection",),
        ("./wlroots/protocol/input-method-unstable-v2.xml", "input_method",),
        ("./wlroots/protocol/virtual-keyboard-unstable-v1.xml", "virtual_keyboard",),
        ("./wlroots/protocol/wlr-data-control-unstable-v1.xml", "data_control",),
        ("./wlroots/protocol/wlr-export-dmabuf-unstable-v1.xml", "export_dmabuf",),
        ("./wlroots/protocol/wlr-foreign-toplevel-management-unstable-v1.xml", "foreign_toplevel_management",),
        ("./wlroots/protocol/wlr-input-inhibitor-unstable-v1.xml", "input_inhibitor",),
        ("./wlroots/protocol/wlr-layer-shell-unstable-v1.xml", "layer_shell",),
        ("./wlroots/protocol/wlr-output-management-unstable-v1.xml", "output_management",),
        ("./wlroots/protocol/wlr-output-power-management-unstable-v1.xml", "output_power_management",),
        ("./wlroots/protocol/wlr-virtual-pointer-unstable-v1.xml", "virtual_pointer",),
        ("./wlroots/protocol/idle.xml", "idle"),
    ];

    for protocol in protocols {
        wayland_scanner::generate_code(
            protocol.0,
            output_dir.join(format!("{}_server_api.rs", protocol.1)),
            wayland_scanner::Side::Server,
        );
        wayland_scanner::generate_code(
            protocol.0,
            output_dir.join(format!("{}_client_api.rs", protocol.1)),
            wayland_scanner::Side::Client,
        );
        wayland_scanner::generate_code(
            protocol.0,
            output_dir.join(format!("{}_interfaces.rs", protocol.1)),
            wayland_scanner::Side::Server,
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
