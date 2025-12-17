# tauri-plugin-sparkle-updater

[![Crates.io Version](https://img.shields.io/crates/v/tauri-plugin-sparkle-updater)](https://crates.io/crates/tauri-plugin-sparkle-updater)
[![npm Version](https://img.shields.io/npm/v/tauri-plugin-sparkle-updater-api)](https://www.npmjs.com/package/tauri-plugin-sparkle-updater-api)
[![License](https://img.shields.io/crates/l/tauri-plugin-sparkle-updater)](LICENSE)

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
tauri-plugin-sparkle-updater = "0.1"
```

### 2. Add the JavaScript dependency

```bash
npm install tauri-plugin-sparkle-updater-api
# or
pnpm add tauri-plugin-sparkle-updater-api
# or
yarn add tauri-plugin-sparkle-updater-api
```

### 3. Download Sparkle framework

```bash
# From the plugin directory
./scripts/download-sparkle.sh
```

Or download manually from [Sparkle releases](https://github.com/sparkle-project/Sparkle/releases).

### 4. Generate EdDSA keys

```bash
# Using Sparkle's generate_keys tool (included after running download-sparkle.sh)
./sparkle-bin/generate_keys
```

This saves the private key to your macOS Keychain and prints the public key (base64-encoded).

> **Note**: Sparkle uses its own Ed25519 key format, which is different from Tauri's built-in updater (minisign). You cannot reuse keys between them - you must generate new keys with Sparkle's tool.

## Configuration

### Info.plist Configuration

Sparkle reads its configuration directly from your app's `Info.plist`. Create `src-tauri/Info.plist` with the following Sparkle keys:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>SUFeedURL</key>
    <string>https://example.com/appcast.xml</string>
    <key>SUPublicEDKey</key>
    <string>YOUR_BASE64_ED25519_PUBLIC_KEY</string>
    <key>SUEnableAutomaticChecks</key>
    <true/>
    <key>SUAutomaticallyUpdate</key>
    <false/>
    <key>SUScheduledCheckInterval</key>
    <integer>86400</integer>
</dict>
</plist>
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `SUFeedURL` | string | *required* | URL to your appcast XML feed |
| `SUPublicEDKey` | string | *required in release* | Base64-encoded Ed25519 public key |
| `SUEnableAutomaticChecks` | boolean | `true` | Enable automatic update checks |
| `SUAutomaticallyUpdate` | boolean | `false` | Auto-download and install updates |
| `SUScheduledCheckInterval` | integer | `86400` | Check interval in seconds (default: 1 day) |

Tauri automatically merges your `src-tauri/Info.plist` into the final app bundle.

### Bundle Configuration

Make sure to include the Sparkle framework in your bundle:

```json
{
  "bundle": {
    "macOS": {
      "frameworks": ["path/to/Sparkle.framework"]
    }
  }
}
```

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
    // sparkle_updater() returns Option - None during `tauri dev`
    if let Some(updater) = app.sparkle_updater() {
        // Check with UI
        updater.check_for_updates().unwrap();

        // Background check
        updater.check_for_updates_in_background().unwrap();

        // Get current version
        let version = updater.current_version().unwrap();
    }
}
```

> **Note**: `sparkle_updater()` returns `None` when running with `tauri dev` because Sparkle requires a valid macOS `.app` bundle. It works normally in release builds (`tauri build`).

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
./sparkle-bin/sign_update App-1.1.0.dmg
# Output: sparkle:edSignature="xxxx" length="12345678"
```

Copy the output values into your appcast XML.

## Signing

### Understanding the Two Types of Signing

There are two separate signing mechanisms involved:

| Type | Purpose | Tool | Key Storage |
|------|---------|------|-------------|
| **Apple Code Signing** | macOS trusts the app binary | `codesign` (via Tauri) | Apple Developer Certificate |
| **Sparkle Update Signing** | Verify update packages | `sign_update` | macOS Keychain (Ed25519) |

**Important**: These are completely independent:
- Apple code signing is handled by Tauri's build process - no changes needed
- Sparkle update signing requires using `./sparkle-bin/sign_update` for your DMG/ZIP files

### Migrating from Tauri's Built-in Updater

If you were using Tauri's built-in updater with minisign:

```bash
# Before (Tauri updater - minisign format)
tauri signer sign app.tar.gz

# After (Sparkle - Ed25519 format)
./sparkle-bin/sign_update app.dmg
```

You **cannot** reuse minisign keys with Sparkle. Generate new keys with `./sparkle-bin/generate_keys`.

### Sandbox Requirements

For sandboxed applications:

1. Add `com.apple.security.network.client` entitlement
2. Configure Sparkle XPC service (see [Sparkle docs](https://sparkle-project.org/documentation/sandboxing/))
3. Ensure proper code signing and notarization

## CI/CD Integration

### GitHub Actions Setup

To sign DMG files with Sparkle in your CI/CD pipeline:

#### 1. Export Private Key

```bash
# Export from your local Keychain
./sparkle-bin/generate_keys -x sparkle_private_key.txt
cat sparkle_private_key.txt
# Output: HfWkFlscLFHsthUMxzU+ERX5HvjUwq/VQUsOgNKND4U= (example)
```

#### 2. Add GitHub Secret

Add `SPARKLE_PRIVATE_KEY` to your repository secrets (Settings → Secrets → Actions).

#### 3. Add Signing Step to Workflow

```yaml
- name: Download Sparkle tools
  run: |
    curl -L -o sparkle.tar.xz "https://github.com/sparkle-project/Sparkle/releases/download/2.8.1/Sparkle-2.8.1.tar.xz"
    mkdir -p sparkle-tools
    tar -xf sparkle.tar.xz -C sparkle-tools
    chmod +x sparkle-tools/bin/*

- name: Sign DMG with Sparkle
  env:
    SPARKLE_PRIVATE_KEY: ${{ secrets.SPARKLE_PRIVATE_KEY }}
  run: |
    DMG_PATH=$(find ./target/release/bundle/dmg -name "*.dmg" -print -quit)

    # Write key to temp file
    echo "$SPARKLE_PRIVATE_KEY" > /tmp/sparkle_key

    # Sign and get signature
    SIGNATURE=$(./sparkle-tools/bin/sign_update "$DMG_PATH" -f /tmp/sparkle_key)
    rm -f /tmp/sparkle_key

    echo "Signature: $SIGNATURE"
    # Output: sparkle:edSignature="xxx" length="123456"
```

### Appcast Requirements

Your appcast feed must serve **DMG files** (not `.tar.gz`) with Sparkle signatures:

```xml
<enclosure
  url="https://cdn.example.com/MyApp-1.0.0.dmg"
  sparkle:edSignature="SPARKLE_SIGNATURE"
  length="12345678"
  type="application/octet-stream"
/>
```

> **Note**: Sparkle signatures are different from Tauri's minisign signatures. You need to update your release pipeline to generate Sparkle signatures for DMG files.

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
