#!/bin/bash
set -e

VERSION="2.8.1"
EXPECTED_SHA256="5cddb7695674ef7704268f38eccaee80e3accbf19e61c1689efff5b6116d85be"

cd "$(dirname "$0")/.."

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

echo "Done!"
