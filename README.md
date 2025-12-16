# tauri-plugin-sparkle-updater

A Tauri plugin that integrates the [Sparkle](https://sparkle-project.org/) update framework for macOS applications.

## Features

- Native macOS update UI via Sparkle framework
- EdDSA (Ed25519) signature verification
- Automatic update checks
- Background silent checks
- Full event system for custom UI integration
- TypeScript/JavaScript API

## Requirements

- macOS 11.0+
- Tauri 2.x
- Sparkle framework 2.x

## Installation

### 1. Add the Rust dependency

```toml
# src-tauri/Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
tauri-plugin-sparkle-updater = { git = "https://github.com/ahonn/tauri-plugin-sparkle_updater" }
```

### 2. Download Sparkle framework

```bash
# From the plugin directory
./scripts/download-sparkle.sh
```

Or download manually from [Sparkle releases](https://github.com/sparkle-project/Sparkle/releases).

### 3. Generate EdDSA keys

```bash
# Using Sparkle's generate_keys tool
./Sparkle.framework/bin/generate_keys
```

This saves the private key to your Keychain and prints the public key (base64-encoded).

## Configuration

### Plugin Configuration

Add to `src-tauri/tauri.conf.json`:

```json
{
  "plugins": {
    "sparkle-updater": {
      "feedUrl": "https://example.com/appcast.xml",
      "publicEdKey": "YOUR_BASE64_ED25519_PUBLIC_KEY",
      "automaticallyChecksForUpdates": true,
      "automaticallyDownloadsUpdates": false,
      "updateCheckInterval": 86400
    }
  }
}
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `feedUrl` | string | *required* | URL to your appcast XML feed |
| `publicEdKey` | string | *required in release* | Base64-encoded Ed25519 public key |
| `automaticallyChecksForUpdates` | boolean | `true` | Enable automatic update checks |
| `automaticallyDownloadsUpdates` | boolean | `false` | Auto-download updates when found |
| `updateCheckInterval` | number | `86400` | Check interval in seconds (default: 1 day) |

### Info.plist Configuration (Required)

Sparkle reads the EdDSA public key directly from Info.plist. Add to your bundle configuration:

```json
{
  "bundle": {
    "macOS": {
      "frameworks": ["path/to/Sparkle.framework"],
      "infoPlist": {
        "SUPublicEDKey": "YOUR_BASE64_ED25519_PUBLIC_KEY",
        "SUFeedURL": "https://example.com/appcast.xml"
      }
    }
  }
}
```

> **Important**: The `SUPublicEDKey` in Info.plist is required for Sparkle to verify update signatures.

## Usage

### Rust Setup

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sparkle_updater::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Rust API

```rust
use tauri_plugin_sparkle_updater::SparkleUpdaterExt;

fn check_updates(app: &tauri::AppHandle) {
    let updater = app.sparkle_updater();

    // Check with UI
    updater.check_for_updates().unwrap();

    // Background check
    updater.check_for_updates_in_background().unwrap();

    // Get current version
    let version = updater.current_version().unwrap();
}
```

### JavaScript/TypeScript API

```ts
import {
  checkForUpdates,
  checkForUpdatesInBackground,
  onUpdateAvailable,
  onError,
} from 'tauri-plugin-sparkle-updater-api';

// Check for updates with native UI
await checkForUpdates();

// Silent background check
await checkForUpdatesInBackground();

// Listen for events
const unlisten = await onUpdateAvailable(({ version, releaseNotes }) => {
  console.log(`Update ${version} available!`);
});

await onError(({ message, code, domain }) => {
  console.error(`Update error: ${message}`);
});
```

### Available Functions

| Function | Description |
|----------|-------------|
| `checkForUpdates()` | Check with native UI dialog |
| `checkForUpdatesInBackground()` | Silent background check |
| `canCheckForUpdates()` | Returns whether checking is possible |
| `currentVersion()` | Get current app version |
| `feedUrl()` / `setFeedUrl(url)` | Get/set feed URL at runtime |
| `automaticallyChecksForUpdates()` / `setAutomaticallyChecksForUpdates(enabled)` | Get/set auto-check |
| `automaticallyDownloadsUpdates()` / `setAutomaticallyDownloadsUpdates(enabled)` | Get/set auto-download |
| `lastUpdateCheckDate()` | Get last check timestamp (ms) |
| `resetUpdateCycle()` | Reset update schedule |

### Events

| Event | Payload | Description |
|-------|---------|-------------|
| `sparkle://checking` | `{}` | Appcast loaded |
| `sparkle://update-available` | `{ version, releaseNotes? }` | Update found |
| `sparkle://update-not-available` | `{}` | No update available |
| `sparkle://downloading` | `{ version }` | Download started |
| `sparkle://downloaded` | `{ version }` | Download completed |
| `sparkle://installing` | `{ version }` | Installation starting |
| `sparkle://error` | `{ message, code?, domain? }` | Error occurred |

## Appcast Feed

Create an appcast XML file for your updates:

```xml
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle">
  <channel>
    <item>
      <title>1.1.0</title>
      <sparkle:version>1.1.0</sparkle:version>
      <sparkle:shortVersionString>1.1.0</sparkle:shortVersionString>
      <sparkle:minimumSystemVersion>11.0</sparkle:minimumSystemVersion>
      <enclosure
        url="https://example.com/releases/App-1.1.0.dmg"
        sparkle:edSignature="BASE64_ED_SIGNATURE"
        length="12345678"
        type="application/octet-stream"
      />
    </item>
  </channel>
</rss>
```

Sign your updates using Sparkle's `sign_update` tool:

```bash
./Sparkle.framework/bin/sign_update App-1.1.0.dmg
```

## Sandbox & Signing

For sandboxed applications:

1. Add `com.apple.security.network.client` entitlement
2. Configure Sparkle XPC service (see [Sparkle docs](https://sparkle-project.org/documentation/sandboxing/))
3. Ensure proper code signing and notarization

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
