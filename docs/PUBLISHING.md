# CI/CD Integration

## Signing Overview

There are two independent signing mechanisms:

| Type | Purpose | Tool |
|------|---------|------|
| **Apple Code Signing** | macOS app trust | `codesign` (via Tauri) |
| **Sparkle Update Signing** | Verify update packages | `sign_update` |

> **Note**: Sparkle uses Ed25519 keys, different from Tauri's minisign. You must generate new keys with `./sparkle-bin/generate_keys`.

## GitHub Actions Setup

### 1. Export Private Key

```bash
./sparkle-bin/generate_keys -x sparkle_private_key.txt
cat sparkle_private_key.txt
```

### 2. Add GitHub Secret

Add `SPARKLE_PRIVATE_KEY` to repository secrets (Settings → Secrets → Actions).

### 3. Workflow Configuration

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

    echo "$SPARKLE_PRIVATE_KEY" > /tmp/sparkle_key
    SIGNATURE=$(./sparkle-tools/bin/sign_update "$DMG_PATH" -f /tmp/sparkle_key)
    rm -f /tmp/sparkle_key

    echo "Signature: $SIGNATURE"
    # Output: sparkle:edSignature="xxx" length="123456"
```

## Appcast Generation

Use the signature output in your appcast XML:

```xml
<enclosure
  url="https://cdn.example.com/MyApp-1.0.0.dmg"
  sparkle:edSignature="SPARKLE_SIGNATURE"
  length="12345678"
  type="application/octet-stream"
/>
```

For appcast format details, see [Sparkle Publishing Documentation](https://sparkle-project.org/documentation/publishing/).
