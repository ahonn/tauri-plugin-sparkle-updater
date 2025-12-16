use std::path::Path;
use std::process::Command;

const COMMANDS: &[&str] = &[
    "ping",
    "check_for_updates",
    "check_for_updates_in_background",
    "can_check_for_updates",
    "current_version",
    "feed_url",
    "set_feed_url",
    "automatically_checks_for_updates",
    "set_automatically_checks_for_updates",
    "automatically_downloads_updates",
    "set_automatically_downloads_updates",
    "last_update_check_date",
    "reset_update_cycle",
];

const SPARKLE_VERSION: &str = "2.8.1";

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let framework_path = Path::new(&manifest_dir).join("Sparkle.framework");

        if !framework_path.exists() {
            download_sparkle(&manifest_dir);
        }

        println!("cargo:rustc-link-search=framework={}", manifest_dir);
        println!("cargo:rustc-link-lib=framework=Sparkle");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
        println!("cargo:rerun-if-changed=Sparkle.framework");
    }
}

fn download_sparkle(manifest_dir: &str) {
    println!("cargo:warning=Sparkle.framework not found, downloading...");

    let url = format!(
        "https://github.com/sparkle-project/Sparkle/releases/download/{}/Sparkle-{}.tar.xz",
        SPARKLE_VERSION, SPARKLE_VERSION
    );

    let temp_dir = std::env::temp_dir().join("sparkle_download");
    let archive_path = temp_dir.join("sparkle.tar.xz");

    std::fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");

    let status = Command::new("curl")
        .args(["-L", "-o", archive_path.to_str().unwrap(), &url])
        .status()
        .expect("Failed to execute curl");

    if !status.success() {
        panic!("Failed to download Sparkle framework");
    }

    let status = Command::new("tar")
        .args([
            "-xf",
            archive_path.to_str().unwrap(),
            "-C",
            temp_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute tar");

    if !status.success() {
        panic!("Failed to extract Sparkle framework");
    }

    let src = temp_dir.join("Sparkle.framework");
    let dst = Path::new(manifest_dir).join("Sparkle.framework");

    if src.exists() {
        let status = Command::new("cp")
            .args(["-R", src.to_str().unwrap(), dst.to_str().unwrap()])
            .status()
            .expect("Failed to copy Sparkle.framework");

        if !status.success() {
            panic!("Failed to copy Sparkle.framework");
        }
    } else {
        panic!("Sparkle.framework not found in extracted archive");
    }

    let _ = std::fs::remove_dir_all(&temp_dir);

    println!("cargo:warning=Sparkle.framework downloaded successfully");
}
