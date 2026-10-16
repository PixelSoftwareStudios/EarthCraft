//! Platform runtimes required by the statically compiled SDL input backend.
use std::{path::Path, process::Command};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=DEVELOPER_DIR");
    println!("cargo::rerun-if-env-changed=SDKROOT");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS")?;
    if target_os == "freebsd" {
        // SDL's BSD joystick parser uses the base-system usbhid library.
        println!("cargo::rustc-link-lib=usbhid");
    }
    if target_os != "macos" {
        return Ok(());
    }
    // Rust's final link does not automatically include Clang's availability
    // runtime for Objective-C @available checks inside the SDL static archive.
    let output = Command::new("xcrun")
        .args([
            "--sdk",
            "macosx",
            "clang",
            "-print-file-name=libclang_rt.osx.a",
        ])
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "Cannot locate Apple compiler runtime: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    let runtime = String::from_utf8(output.stdout)?;
    let runtime = runtime.trim();
    if !Path::new(runtime).is_file() {
        return Err(format!("Apple compiler runtime archive is unavailable: {runtime}").into());
    }
    println!("cargo::rustc-link-arg={runtime}");
    Ok(())
}
