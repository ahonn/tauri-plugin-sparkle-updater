# tauri-plugin-sparkle-updater

[![Crates.io Version](https://img.shields.io/crates/v/tauri-plugin-sparkle-updater)](https://crates.io/crates/tauri-plugin-sparkle-updater)
[![npm Version](https://img.shields.io/npm/v/tauri-plugin-sparkle-updater-api)](https://www.npmjs.com/package/tauri-plugin-sparkle-updater-api)
[![License](https://img.shields.io/crates/l/tauri-plugin-sparkle-updater)](LICENSE)

A Tauri plugin that integrates the [Sparkle](https://sparkle-project.org/) update framework for macOS applications.

## Features

- Native macOS update UI via Sparkle framework
- EdDSA (Ed25519) signature verification
- Automatic and background update checks
- Full event system for custom UI integration
- Channel-based updates, custom HTTP headers, phased rollout
- TypeScript/JavaScript API with full type definitions

## Requirements

- macOS 11.0+
- Tauri 2.x
- Sparkle framework 2.8.1

## Quick Start

### 1. Install dependencies

```toml
# src-tauri/Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
tauri-plugin-sparkle-updater = "0.2"
```

```bash
npm install tauri-plugin-sparkle-updater-api
```

### 2. Download Sparkle & generate keys

```bash
# Download Sparkle framework
curl -fsSL https://raw.githubusercontent.com/ahonn/tauri-plugin-sparkle-updater/refs/heads/master/scripts/download-sparkle.sh | bash

# Generate signing keys (saved to Keychain)
./sparkle-bin/generate_keys
```

### 3. Configure Info.plist

Create `src-tauri/Info.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>SUFeedURL</key>
    <string>https://example.com/appcast.xml</string>
    <key>SUPublicEDKey</key>
    <string>YOUR_BASE64_PUBLIC_KEY</string>
</dict>
</plist>
```

### 4. Bundle configuration

```json
{
  "bundle": {
    "macOS": {
      "frameworks": ["path/to/Sparkle.framework"]
    }
  }
}
```

### 5. Register plugin

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sparkle_updater::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Basic Usage

### Rust

```rust
use tauri_plugin_sparkle_updater::SparkleUpdaterExt;

fn check_updates(app: &tauri::AppHandle) {
    if let Some(updater) = app.sparkle_updater() {
        updater.check_for_updates().unwrap();
    }
}
```

> **Note**: `sparkle_updater()` returns `None` during `tauri dev` (requires `.app` bundle).

### TypeScript

```ts
import {
  checkForUpdates,
  checkForUpdatesInBackground,
  onDidFindValidUpdate,
  onDidAbortWithError,
} from 'tauri-plugin-sparkle-updater-api';

// Check with native UI
await checkForUpdates();

// Background check
await checkForUpdatesInBackground();

// Listen for events
await onDidFindValidUpdate((info) => {
  console.log(`Update ${info.version} available!`);
});
```

## Documentation

- [API Reference (docs.rs)](https://docs.rs/tauri-plugin-sparkle-updater) - Rust API documentation
- [Publishing](./docs/PUBLISHING.md) - Signing and CI/CD workflow
- [Sparkle Documentation](https://sparkle-project.org/documentation/) - Appcast format, configuration keys, sandboxing

## Cross-Platform

For Windows/Linux, use the official [tauri-plugin-updater](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/updater):

```rust
#[cfg(target_os = "macos")]
builder = builder.plugin(tauri_plugin_sparkle_updater::init());

#[cfg(not(target_os = "macos"))]
builder = builder.plugin(tauri_plugin_updater::Builder::new().build());
```

## License

MIT
