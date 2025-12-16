use std::path::Path;

const COMMANDS: &[&str] = &[
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

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        setup_sparkle_framework();
    }
}

/// Set up Sparkle framework linking
fn setup_sparkle_framework() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let framework_path = Path::new(&manifest_dir).join("Sparkle.framework");

    if !framework_path.exists() {
        panic!(
            "\n\
            ╔══════════════════════════════════════════════════════════════╗\n\
            ║  Sparkle.framework not found!                                ║\n\
            ╠══════════════════════════════════════════════════════════════╣\n\
            ║  Please download Sparkle and place it in the project root:   ║\n\
            ║                                                              ║\n\
            ║  ./scripts/download-sparkle.sh                               ║\n\
            ║                                                              ║\n\
            ║  Or manually:                                                ║\n\
            ║  curl -L -o sparkle.tar.xz \\                                 ║\n\
            ║    https://github.com/sparkle-project/Sparkle/releases/\\     ║\n\
            ║    download/2.8.1/Sparkle-2.8.1.tar.xz                       ║\n\
            ║  tar -xf sparkle.tar.xz Sparkle.framework                    ║\n\
            ║  rm sparkle.tar.xz                                           ║\n\
            ╚══════════════════════════════════════════════════════════════╝\n"
        );
    }

    println!("cargo:rustc-link-search=framework={}", manifest_dir);
    println!("cargo:rustc-link-lib=framework=Sparkle");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rerun-if-changed=Sparkle.framework");
}
