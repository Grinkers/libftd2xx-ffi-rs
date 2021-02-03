use std::env;

fn search_path<'a>() -> &'a str {
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86_64" => {
                #[cfg(feature = "dynamic")]
                {
                    "vendor\\windows\\amd64"
                }
                #[cfg(not(feature = "dynamic"))]
                {
                    "vendor\\windows\\Static\\amd64"
                }
            }
            "x86" => {
                #[cfg(feature = "dynamic")]
                {
                    "vendor\\windows\\i386"
                }
                #[cfg(not(feature = "dynamic"))]
                {
                    "vendor\\windows\\Static\\i386"
                }
            }
            target_arch => panic!("Target architecture not supported: {}", target_arch),
        },
        "linux" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86_64" => "vendor/linux/x64/build",
            "x86" => "vendor/linux/x86/build",
            "arm" | "aarch64" => match env::var("TARGET").unwrap().as_str() {
                "arm-unknown-linux-musleabihf" | "arm-unknown-linux-gnueabihf" => {
                    "vendor/linux/armv6-hf/build"
                }
                "armv7-unknown-linux-musleabihf" | "armv7-unknown-linux-gnueabihf" => {
                    "vendor/linux/armv7-hf/build"
                }
                "aarch64-unknown-linux-musl" | "aarch64-unknown-linux-gnu" => {
                    "vendor/linux/armv8-hf/build"
                }
                target => panic!("Target not supported: {}", target),
            },
            target_arch => panic!("Target architecture not supported: {}", target_arch),
        },
        target_os => panic!("Target OS not supported: {}", target_os),
    }
}

fn header_path<'a>() -> &'a str {
    match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => "vendor/windows/ftd2xx.h",
        "linux" => match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
            "x86_64" => "vendor/linux/x64/ftd2xx.h",
            "x86" => "vendor/linux/x86/ftd2xx.h",
            "arm" | "aarch64" => match env::var("TARGET").unwrap().as_str() {
                "arm-unknown-linux-musleabihf" | "arm-unknown-linux-gnueabihf" => {
                    "vendor/linux/armv6-hf/ftd2xx.h"
                }
                "armv7-unknown-linux-musleabihf" | "armv7-unknown-linux-gnueabihf" => {
                    "vendor/linux/armv7-hf/ftd2xx.h"
                }
                "aarch64-unknown-linux-musl" | "aarch64-unknown-linux-gnu" => {
                    "vendor/linux/armv8-hf/ftd2xx.h"
                }
                target => panic!("Target not supported: {}", target),
            },
            target_arch => panic!("Target architecture not supported: {}", target_arch),
        },
        target_os => panic!("Target OS not supported: {}", target_os),
    }
}

// This adds sysroot to bindgen if cross-compiling.
// This is not great, please open an issue or pull-request if you know of a
// better way to handle this problem.
#[cfg(feature = "bindgen")]
fn clang_args() -> &'static [&'static str] {
    match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
        #[cfg(all(target_os = "linux", not(target_arch = "arm")))]
        "arm" => &["--sysroot=/usr/arm-linux-gnueabihf"],
        #[cfg(all(target_os = "linux", not(target_arch = "aarch64")))]
        "aarch64" => &["--sysroot=/usr/aarch64-linux-gnu"],
        _ => &[],
    }
}

fn linker_options(search: &str) {
    println!("cargo:rustc-link-search=native={}", search);

    #[cfg(feature = "dynamic")]
    {
        println!("cargo:rustc-link-lib=dylib=ftd2xx");
    }

    #[cfg(not(feature = "dynamic"))]
    {
        println!("cargo:rustc-link-lib=static=ftd2xx");
        match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
            "windows" => {
                // TODO: find a real way to find the VS install path
                match env::var("CARGO_CFG_TARGET_ARCH").unwrap().as_str() {
                    "x86_64" => {
                        println!("cargo:rustc-link-search=native={}",
                                 "C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\MSVC\\14.28.29333\\lib\\x64");
                    }
                    "x86" => {
                        println!("cargo:rustc-link-search=native={}",
                                 "C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\MSVC\\14.28.29333\\lib\\x86");
                    }
                    target_arch => panic!("Target architecture not supported: {}", target_arch),
                }

                println!("cargo:rustc-link-lib=static=legacy_stdio_definitions");
                println!("cargo:rustc-link-lib=user32");
            }
            "linux" => {}
            target_os => panic!("Target OS not supported: {}", target_os),
        }
    }
}

fn main() {
    let cwd = env::current_dir().unwrap();
    let mut header = cwd.clone();
    header.push(header_path());
    let mut search = cwd;
    search.push(search_path());

    linker_options(search.to_str().unwrap());

    println!("cargo:rerun-if-changed={}", header.to_str().unwrap());

    #[cfg(feature = "bindgen")]
    {
        let bindings = bindgen::Builder::default()
            .header(header.to_str().unwrap())
            .whitelist_function("FT_.*")
            .whitelist_type("FT_.*")
            .whitelist_var("FT_.*")
            .rustfmt_bindings(true)
            .derive_default(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .clang_args(clang_args())
            .generate()
            .expect("Unable to generate bindings");

        let out_path = std::path::PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
