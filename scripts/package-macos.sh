#!/bin/bash
set -euo pipefail

# Package Light browser as a macOS .app bundle and DMG
# Usage: ./scripts/package-macos.sh [--sign] [--notarize]

TARGET="${1:-}"
SIGN=false
NOTARIZE=false

for arg in "$@"; do
  case $arg in
    --sign) SIGN=true ;;
    --notarize) NOTARIZE=true ;;
  esac
done

# Determine architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
  RUST_TARGET="aarch64-apple-darwin"
else
  RUST_TARGET="x86_64-apple-darwin"
fi

echo "==> Building for $RUST_TARGET"
cargo build --release --target "$RUST_TARGET"

# Create .app bundle
APP_DIR="target/release/Light.app"
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cp macos/Light.app/Contents/Info.plist "$APP_DIR/Contents/"
cp macos/Light.app/Contents/Resources/AppIcon.icns "$APP_DIR/Contents/Resources/"
cp "target/$RUST_TARGET/release/Light" "$APP_DIR/Contents/MacOS/Light"

echo "==> Created $APP_DIR"

# Code sign
if [ "$SIGN" = true ]; then
  echo "==> Signing app bundle"
  codesign --deep --force --options runtime \
    --sign "${APPLE_SIGNING_IDENTITY:-Developer ID Application}" \
    "$APP_DIR"
  echo "==> Signed"
fi

# Create DMG
DMG_NAME="Light-${ARCH}.dmg"
echo "==> Creating DMG: $DMG_NAME"

if command -v create-dmg &> /dev/null; then
  create-dmg \
    --volname "Light" \
    --window-size 600 400 \
    --icon "Light.app" 150 200 \
    --app-drop-link 450 200 \
    --no-internet-enable \
    "target/release/$DMG_NAME" \
    "$APP_DIR"
else
  # Fallback to hdiutil
  hdiutil create -volname "Light" -srcfolder "$APP_DIR" \
    -ov -format UDZO "target/release/$DMG_NAME"
fi

echo "==> Created target/release/$DMG_NAME"

# Notarize
if [ "$NOTARIZE" = true ]; then
  echo "==> Submitting for notarization"
  xcrun notarytool submit "target/release/$DMG_NAME" \
    --apple-id "$APPLE_ID" \
    --password "$APPLE_ID_PASSWORD" \
    --team-id "$APPLE_TEAM_ID" \
    --wait

  echo "==> Stapling"
  xcrun stapler staple "target/release/$DMG_NAME"
  echo "==> Notarization complete"
fi
