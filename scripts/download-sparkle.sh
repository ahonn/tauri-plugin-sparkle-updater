#!/bin/bash
set -e

VERSION="2.9.6"
EXPECTED_SHA256="52bf9e88cdd972fc0c81501377a880e90d47031bd8ca5462488f843e2609e192"

cd "$(pwd "$0")/src-tauri"

if [ -d "Sparkle.framework" ]; then
    echo "Sparkle.framework already exists"
    exit 0
fi

TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Downloading Sparkle ${VERSION}..."
curl -L -o "$TEMP_DIR/sparkle.tar.xz" \
    "https://github.com/sparkle-project/Sparkle/releases/download/${VERSION}/Sparkle-${VERSION}.tar.xz"

echo "Verifying checksum..."
echo "${EXPECTED_SHA256}  $TEMP_DIR/sparkle.tar.xz" | shasum -a 256 -c -

echo "Extracting Sparkle.framework..."
tar -xf "$TEMP_DIR/sparkle.tar.xz" -C "$TEMP_DIR"

cp -R "$TEMP_DIR/Sparkle.framework" .

# Also copy the bin tools (generate_keys, sign_update)
if [ -d "$TEMP_DIR/bin" ]; then
    echo "Copying Sparkle bin tools..."
    cp -R "$TEMP_DIR/bin" ./sparkle-bin
    chmod +x ./sparkle-bin/*
fi

echo "Done!"
echo ""
echo "To generate EdDSA keys for signing updates:"
echo "  ./sparkle-bin/generate_keys"
