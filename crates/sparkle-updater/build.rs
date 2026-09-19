use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=SPARKLE_FRAMEWORK_PATH");
    println!("cargo:rerun-if-env-changed=DOCS_RS");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos")
        || std::env::var_os("DOCS_RS").is_some()
    {
        return;
    }

    let framework_dir = if let Some(path) = std::env::var_os("SPARKLE_FRAMEWORK_PATH") {
        let path = PathBuf::from(path);
        assert!(
            path.join("Sparkle.framework").is_dir(),
            "SPARKLE_FRAMEWORK_PATH must point to a directory containing Sparkle.framework: {}",
            path.display()
        );
        path
    } else {
        find_framework().unwrap_or_else(|| {
            panic!(
                "Sparkle.framework was not found. Download the official Sparkle distribution \
                 explicitly, then set SPARKLE_FRAMEWORK_PATH to the directory containing \
                 Sparkle.framework. In this repository, run: bash scripts/download-sparkle.sh"
            )
        })
    };

    println!(
        "cargo:rerun-if-changed={}",
        framework_dir.join("Sparkle.framework").display()
    );
    println!(
        "cargo:rustc-link-search=framework={}",
        framework_dir.display()
    );
    println!("cargo:rustc-link-lib=framework=Sparkle");
    println!("cargo:rustc-link-lib=framework=AppKit");
    println!("cargo:rustc-link-lib=framework=Foundation");
}

fn find_framework() -> Option<PathBuf> {
    // OUT_DIR ancestors include the consuming application's project directory
    // when Cargo uses its default target directory. An explicit path also works
    // with shared targets and packaged crates.
    for variable in ["OUT_DIR", "CARGO_MANIFEST_DIR"] {
        if let Some(path) = std::env::var_os(variable) {
            for ancestor in Path::new(&path).ancestors() {
                if ancestor.join("Sparkle.framework").is_dir() {
                    return Some(ancestor.to_owned());
                }
            }
        }
    }
    None
}
