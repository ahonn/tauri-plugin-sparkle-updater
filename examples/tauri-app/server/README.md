# Appcast Server

A simple local server for testing Sparkle updates with the tauri-app example.

## Setup

```bash
# Install dependencies
pnpm install

# Start the server
pnpm dev
```

The server runs on `http://localhost:8787` by default.

## Endpoints

| Endpoint | Description |
|----------|-------------|
| `GET /` | Server status page |
| `GET /appcast.xml` | Sparkle appcast XML feed |
| `GET /releases/:filename` | Download DMG files |

## Complete Testing Guide

### Prerequisites

1. macOS 11.0 or later
2. Node.js and pnpm installed
3. Rust and Tauri CLI installed

### Step 1: Generate EdDSA Keys (First Time Only)

```bash
# Generate keys with a custom account name (recommended)
../../../sparkle-bin/generate_keys --account "tauri-app-demo"

# Output example:
# A key has been generated and saved in your keychain.
#     <key>SUPublicEDKey</key>
#     <string>WV/UDuFy1UI+UPuzoo5bDxkYLFM8QDMWmFTmYnuOckY=</string>

# Export private key for signing (save to server directory)
../../../sparkle-bin/generate_keys --account "tauri-app-demo" -x sparkle_private_key.txt
```

Update `src-tauri/Info.plist` with the public key:

```xml
<key>SUPublicEDKey</key>
<string>YOUR_PUBLIC_KEY_HERE</string>

<key>SUFeedURL</key>
<string>http://localhost:8787/appcast.xml</string>
```

### Step 2: Build the Update Version (0.2.0)

```bash
cd ../src-tauri

# Update version in tauri.conf.json to "0.2.0"
# Then build
pnpm tauri build
```

### Step 3: Fix Code Signing (Important!)

Tauri's default build uses linker-signed ad-hoc signatures which lack sealed resources. Sparkle requires proper code signing for update validation.

```bash
# Re-sign the .app with proper ad-hoc signature
cd target/release/bundle/macos
codesign --force --deep --sign - tauri-app.app

# Verify the signature
codesign -vvv --deep tauri-app.app
# Should output: "valid on disk" and "satisfies its Designated Requirement"
```

### Step 4: Create Properly Signed DMG

```bash
# Create a new DMG with the re-signed app
cd ../  # Now in target/release/bundle/

# Remove old DMG
rm -f dmg/tauri-app_0.2.0_aarch64.dmg

# Create new DMG
hdiutil create -volname "tauri-app" \
  -srcfolder macos/tauri-app.app \
  -ov -format UDZO \
  dmg/tauri-app_0.2.0_aarch64.dmg
```

### Step 5: Sign and Configure the DMG

```bash
# Copy to releases directory
cp dmg/tauri-app_0.2.0_aarch64.dmg /path/to/server/releases/

# Sign with EdDSA
cd /path/to/server
../../../sparkle-bin/sign_update releases/tauri-app_0.2.0_aarch64.dmg \
  -f sparkle_private_key.txt

# Output example:
# sparkle:edSignature="xxx..." length="4394522"
```

Update `releases/releases.json` with the signature:

```json
{
  "tauri-app_0.2.0_aarch64.dmg": {
    "version": "0.2.0",
    "signature": "YOUR_SIGNATURE_HERE",
    "length": 4394522
  }
}
```

### Step 6: Build the Base Version (0.1.0)

```bash
cd ../src-tauri

# Update version in tauri.conf.json to "0.1.0"
pnpm tauri build

# Re-sign the app (same as Step 3)
cd target/release/bundle/macos
codesign --force --deep --sign - tauri-app.app
```

### Step 7: Run the Test

```bash
# Terminal 1: Start the appcast server
cd examples/tauri-app/server
pnpm dev

# Terminal 2: Run the 0.1.0 app
open ../src-tauri/target/release/bundle/macos/tauri-app.app
```

In the app:
1. Click "Check for Updates"
2. Sparkle should detect version 0.2.0
3. Click "Install Update"
4. The app should update and restart

## Configuration

### releases.json

The server reads signature information from `releases/releases.json`:

```json
{
  "tauri-app_0.2.0_aarch64.dmg": {
    "version": "0.2.0",
    "signature": "EdDSA_SIGNATURE_BASE64",
    "length": 4394522
  }
}
```

Alternatively, you can set the `ED_SIGNATURE` environment variable.

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Server port | `8787` |
| `ED_SIGNATURE` | EdDSA signature (fallback if not in releases.json) | (empty) |
| `UPDATE_VERSION` | Override version detection | (auto-detected) |

## Directory Structure

```
server/
├── src/
│   └── index.ts           # Server implementation
├── releases/
│   ├── releases.json      # Signature configuration
│   ├── *.dmg              # DMG files for updates
│   └── .gitkeep
├── sparkle_private_key.txt # EdDSA private key (do not commit!)
├── package.json
├── tsconfig.json
└── README.md
```

## Troubleshooting

### "The update is improperly signed and could not be validated"

This error occurs when Sparkle cannot verify the update. Common causes:

1. **Code signing issue**: The app inside DMG lacks proper code signing
   ```bash
   # Check DMG app signature
   hdiutil attach releases/tauri-app_0.2.0_aarch64.dmg -nobrowse
   codesign -vvv --deep /Volumes/tauri-app/tauri-app.app
   hdiutil detach /Volumes/tauri-app
   ```

   If you see `code has no resources but signature indicates they must be present`, you need to re-sign the app and recreate the DMG (see Steps 3-4).

2. **EdDSA signature mismatch**: The signature in appcast doesn't match the file
   ```bash
   # Re-sign and verify
   ../../../sparkle-bin/sign_update releases/tauri-app_0.2.0_aarch64.dmg -f sparkle_private_key.txt
   ```

3. **Public key mismatch**: `SUPublicEDKey` in Info.plist doesn't match the private key
   ```bash
   # Check current public key
   ../../../sparkle-bin/generate_keys --account "tauri-app-demo" -p
   ```

4. **File size mismatch**: The `length` in appcast doesn't match actual file size
   ```bash
   ls -l releases/tauri-app_0.2.0_aarch64.dmg
   ```

### Verify appcast content

```bash
curl http://localhost:8787/appcast.xml
```

Check that:
- `sparkle:edSignature` matches your signature
- `length` matches the actual file size
- `sparkle:version` is higher than the running app version

## Security Notes

- **Never commit** `sparkle_private_key.txt` to version control
- Add it to `.gitignore`
- In CI/CD, use secrets management for the private key
- The public key (`SUPublicEDKey`) is safe to commit in Info.plist
