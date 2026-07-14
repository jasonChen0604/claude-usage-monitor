#!/usr/bin/env bash
# Re-signs the Tauri-built .app and packages a fresh .dmg ourselves.
#
# Tauri's own macOS bundler can ad-hoc sign the .app *before* this project's
# custom `bundle.resources` mapping (scripts/claude-statusline-collector.cjs)
# finishes copying into Contents/Resources/, which leaves the code signature
# sealing a resource set that no longer matches the actual bundle contents.
# macOS then refuses to launch the app with a misleading "is damaged" error
# (confirmed via `spctl -a -vv`: "code has no resources but signature
# indicates they must be present"). Re-signing after all resources are in
# place fixes this — see docs/reinstall-testing-sop.md.
set -euo pipefail

cd "$(dirname "$0")/.."

APP_NAME="Claude Usage Monitor"
BUNDLE_MACOS_DIR="src-tauri/target/release/bundle/macos"
APP_PATH="$BUNDLE_MACOS_DIR/$APP_NAME.app"
VERSION=$(node -p "require('./package.json').version")
DMG_DIR="src-tauri/target/release/bundle/dmg"
DMG_PATH="$DMG_DIR/${APP_NAME// /_}_${VERSION}_aarch64.dmg"

if [ ! -d "$APP_PATH" ]; then
  echo "error: $APP_PATH not found — run 'pnpm tauri build' first" >&2
  exit 1
fi

echo "Re-signing $APP_PATH (ad-hoc, after resources are in place)..."
codesign --force --deep --sign - "$APP_PATH"

echo "Verifying signature..."
spctl -a -vv "$APP_PATH" 2>&1 | grep -q "code has no resources but signature" && {
  echo "error: signature still invalid after re-sign" >&2
  exit 1
}

mkdir -p "$DMG_DIR"
rm -f "$DMG_PATH"

STAGING_DIR=$(mktemp -d)
trap 'rm -rf "$STAGING_DIR"' EXIT

cp -R "$APP_PATH" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"

echo "Creating $DMG_PATH..."
hdiutil create -volname "$APP_NAME" -srcfolder "$STAGING_DIR" -ov -format UDZO "$DMG_PATH"

echo "Done: $DMG_PATH"
