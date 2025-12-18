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
    "update_check_interval",
    "set_update_check_interval",
    "check_for_update_information",
    "session_in_progress",
    "http_headers",
    "set_http_headers",
    "user_agent_string",
    "set_user_agent_string",
    "sends_system_profile",
    "set_sends_system_profile",
    "clear_feed_url_from_user_defaults",
    "reset_update_cycle_after_short_delay",
    "allowed_channels",
    "set_allowed_channels",
    "feed_url_override",
    "set_feed_url_override",
    "feed_parameters",
    "set_feed_parameters",
    "should_download_release_notes",
    "set_should_download_release_notes",
    "should_relaunch_application",
    "set_should_relaunch_application",
    "may_check_for_updates_config",
    "set_may_check_for_updates_config",
    "should_proceed_with_update",
    "set_should_proceed_with_update",
    "decryption_password",
    "set_decryption_password",
    "last_found_update",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" && !is_publish_verify() {
        setup_sparkle_framework();
    }
}

fn is_publish_verify() -> bool {
    if std::env::var("DOCS_RS").is_ok() {
        return true;
    }

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        if manifest_dir.contains("target/package/") {
            return true;
        }
    }

    false
}

fn find_src_tauri_from_out_dir() -> Option<String> {
    let out_dir = std::env::var("OUT_DIR").ok()?;
    let out_path = Path::new(&out_dir);

    for ancestor in out_path.ancestors() {
        if ancestor.join("tauri.conf.json").exists() {
            return Some(ancestor.to_string_lossy().to_string());
        }
    }
    None
}

fn setup_sparkle_framework() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut search_paths: Vec<String> = Vec::new();

    if let Ok(path) = std::env::var("SPARKLE_FRAMEWORK_PATH") {
        search_paths.push(path);
    }

    if let Some(src_tauri) = find_src_tauri_from_out_dir() {
        search_paths.push(src_tauri);
    }

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
