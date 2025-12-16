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

/// Find the src-tauri directory from OUT_DIR
/// OUT_DIR is typically: <project>/src-tauri/target/<arch>/release/build/<crate>-<hash>/out
fn find_src_tauri_from_out_dir() -> Option<String> {
    let out_dir = std::env::var("OUT_DIR").ok()?;
    let out_path = Path::new(&out_dir);

    // Walk up the directory tree looking for tauri.conf.json
    for ancestor in out_path.ancestors() {
        if ancestor.join("tauri.conf.json").exists() {
            return Some(ancestor.to_string_lossy().to_string());
        }
    }
    None
}

/// Set up Sparkle framework linking
fn setup_sparkle_framework() {
    // Search for Sparkle.framework in multiple locations:
    // 1. SPARKLE_FRAMEWORK_PATH environment variable (explicit override)
    // 2. Application's src-tauri directory (auto-detected from OUT_DIR)
    // 3. Plugin's CARGO_MANIFEST_DIR (for local development)

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let mut search_paths: Vec<String> = Vec::new();

    // 1. Environment variable override (highest priority)
    if let Ok(path) = std::env::var("SPARKLE_FRAMEWORK_PATH") {
        search_paths.push(path);
    }

    // 2. Auto-detect src-tauri from OUT_DIR (for git dependencies)
    if let Some(src_tauri) = find_src_tauri_from_out_dir() {
        search_paths.push(src_tauri);
    }

    // 3. Plugin directory (for local development with path dependency)
    search_paths.push(manifest_dir.clone());

    let mut framework_dir = None;

    for search_path in &search_paths {
        if search_path.is_empty() {
            continue;
        }
        let path = Path::new(search_path);
        let framework_path = path.join("Sparkle.framework");

        if framework_path.exists() {
            println!(
                "cargo:warning=Found Sparkle.framework at: {}",
                framework_path.display()
            );
            framework_dir = Some(search_path.clone());
            break;
        }
    }

    let framework_dir = framework_dir.unwrap_or_else(|| {
        eprintln!("Searched paths: {:?}", search_paths);
        panic!(
            "\n\
            ╔══════════════════════════════════════════════════════════════╗\n\
            ║  Sparkle.framework not found!                                ║\n\
            ╠══════════════════════════════════════════════════════════════╣\n\
            ║  Please download Sparkle and place it in src-tauri/:         ║\n\
            ║                                                              ║\n\
            ║  cd src-tauri                                                ║\n\
            ║  curl -L -o sparkle.tar.xz \\                                 ║\n\
            ║    https://github.com/sparkle-project/Sparkle/releases/\\     ║\n\
            ║    download/2.8.1/Sparkle-2.8.1.tar.xz                       ║\n\
            ║  tar -xf sparkle.tar.xz Sparkle.framework                    ║\n\
            ║  rm sparkle.tar.xz                                           ║\n\
            ╚══════════════════════════════════════════════════════════════╝\n"
        )
    });

    println!("cargo:rustc-link-search=framework={}", framework_dir);
    println!("cargo:rustc-link-lib=framework=Sparkle");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rerun-if-env-changed=SPARKLE_FRAMEWORK_PATH");
}
