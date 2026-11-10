//! Build script: on macOS, compiles the small Objective-C gamepad shim
//! (`macos/shim/oi_platform.m`) with the system toolchain — no extra crates — and links
//! it with the GameController framework. On other platforms it does nothing.
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
        macos_shim();
    }
}

fn macos_shim() {
    let source = "macos/shim/oi_platform.m";
    println!("cargo:rerun-if-changed={source}");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let arch = match std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
        Ok("aarch64") => "arm64",
        Ok("x86_64") => "x86_64",
        Ok(other) => panic!("unsupported macOS architecture: {other}"),
        Err(e) => panic!("CARGO_CFG_TARGET_ARCH: {e}"),
    };
    // Match rustc's deployment target so the linker sees one consistent minimum.
    let min = std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "11.0".into());
    let object = out.join("oi_platform.o");
    let run = |command: &mut Command| {
        let status = command.status().unwrap_or_else(|e| panic!("run {command:?}: {e}"));
        assert!(status.success(), "{command:?} failed with {status}");
    };
    run(Command::new("xcrun")
        .args(["clang", "-c", "-fobjc-arc", "-O2", "-Wall"])
        .args(["-target", &format!("{arch}-apple-macos{min}")])
        .arg(source)
        .arg("-o")
        .arg(&object));
    run(Command::new("xcrun").args(["ar", "crs"]).arg(out.join("liboi_platform.a")).arg(&object));
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=oi_platform");
    for framework in ["GameController", "Foundation"] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}
