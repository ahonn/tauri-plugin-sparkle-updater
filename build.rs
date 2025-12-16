use plist::Value;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
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

/// Sparkle updater plugin configuration from tauri.conf.json
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SparkleConfig {
    feed_url: Option<String>,
    public_ed_key: Option<String>,
}

/// Partial tauri.conf.json structure
#[derive(Debug, Deserialize)]
struct TauriConfig {
    plugins: Option<Plugins>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Plugins {
    sparkle_updater: Option<SparkleConfig>,
}

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "macos" {
        setup_sparkle_framework();
        generate_info_plist();
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

/// Generate Info.plist from tauri.conf.json plugin configuration
fn generate_info_plist() {
    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();

    let tauri_conf_path = find_tauri_conf(&out_dir);

    let Some(conf_path) = tauri_conf_path else {
        println!("cargo:warning=Could not find tauri.conf.json, skipping Info.plist generation");
        return;
    };

    println!("cargo:rerun-if-changed={}", conf_path.display());

    // Read and parse tauri.conf.json
    let conf_content = match fs::read_to_string(&conf_path) {
        Ok(content) => content,
        Err(e) => {
            println!("cargo:warning=Failed to read tauri.conf.json: {}", e);
            return;
        }
    };

    let config: TauriConfig = match serde_json::from_str(&conf_content) {
        Ok(config) => config,
        Err(e) => {
            println!("cargo:warning=Failed to parse tauri.conf.json: {}", e);
            return;
        }
    };

    // Extract sparkle-updater configuration
    let Some(plugins) = config.plugins else {
        println!("cargo:warning=No plugins section in tauri.conf.json");
        return;
    };

    let Some(sparkle_config) = plugins.sparkle_updater else {
        println!("cargo:warning=No sparkle-updater plugin configuration found");
        return;
    };

    // Collect Sparkle keys to add/update
    let mut sparkle_keys: HashMap<String, String> = HashMap::new();

    if let Some(ref feed_url) = sparkle_config.feed_url {
        sparkle_keys.insert("SUFeedURL".to_string(), feed_url.clone());
    }

    if let Some(ref public_key) = sparkle_config.public_ed_key {
        sparkle_keys.insert("SUPublicEDKey".to_string(), public_key.clone());
    }

    if sparkle_keys.is_empty() {
        println!("cargo:warning=No Sparkle configuration to write to Info.plist");
        return;
    }

    let info_plist_path = conf_path.parent().unwrap().join("Info.plist");
    println!("cargo:rerun-if-changed={}", info_plist_path.display());

    // Read existing Info.plist or create new dictionary
    let mut plist_dict: plist::Dictionary = if info_plist_path.exists() {
        match plist::from_file::<_, Value>(&info_plist_path) {
            Ok(Value::Dictionary(dict)) => dict,
            Ok(_) => {
                println!("cargo:warning=Info.plist is not a dictionary, creating new one");
                plist::Dictionary::new()
            }
            Err(e) => {
                println!("cargo:warning=Failed to parse existing Info.plist: {}", e);
                plist::Dictionary::new()
            }
        }
    } else {
        plist::Dictionary::new()
    };

    // Merge Sparkle keys into the dictionary
    for (key, value) in sparkle_keys {
        plist_dict.insert(key, Value::String(value));
    }

    // Write the updated plist
    match plist::to_file_xml(&info_plist_path, &plist_dict) {
        Ok(_) => {
            println!(
                "cargo:warning=Generated/Updated Info.plist at {}",
                info_plist_path.display()
            );
        }
        Err(e) => {
            println!("cargo:warning=Failed to write Info.plist: {}", e);
        }
    }
}

/// Find tauri.conf.json by searching from OUT_DIR upwards
fn find_tauri_conf(out_dir: &str) -> Option<std::path::PathBuf> {
    if out_dir.is_empty() {
        return None;
    }

    let out_path = Path::new(out_dir);

    // Strategy: Look for target directory and find src-tauri sibling
    // OUT_DIR is typically: /path/to/app/src-tauri/target/debug/build/xxx/out
    let mut current = out_path;
    while let Some(parent) = current.parent() {
        // Check if this directory contains tauri.conf.json
        let conf_path = parent.join("tauri.conf.json");
        if conf_path.exists() {
            return Some(conf_path);
        }

        // Check if parent is "target" and sibling has tauri.conf.json
        if parent.file_name().map(|n| n == "target").unwrap_or(false) {
            if let Some(workspace) = parent.parent() {
                let conf_path = workspace.join("tauri.conf.json");
                if conf_path.exists() {
                    return Some(conf_path);
                }
            }
        }

        current = parent;
    }

    None
}
