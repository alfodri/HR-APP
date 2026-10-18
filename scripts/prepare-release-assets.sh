#!/usr/bin/env bash
# Build signed bundles and write release-assets/ + latest.json for GitHub Releases.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="$(node -p "require('./package.json').version")"
TAG="v${VERSION}"
OUT="$ROOT/release-assets"
KEY="$ROOT/src-tauri/.tauri/hr-app.key"

if [[ ! -f "$KEY" ]]; then
  echo "Missing signing key: $KEY"
  echo "Run: npx tauri signer generate -w src-tauri/.tauri/hr-app.key -f --password \"\""
  exit 1
fi

export TAURI_SIGNING_PRIVATE_KEY="$(cat "$KEY")"
npm run tauri build

mkdir -p "$OUT"
rm -f "$OUT"/*

cp "$ROOT/src-tauri/target/release/bundle/appimage/"*.AppImage "$OUT/" 2>/dev/null || true
cp "$ROOT/src-tauri/target/release/bundle/appimage/"*.AppImage.sig "$OUT/" 2>/dev/null || true
cp "$ROOT/src-tauri/target/release/bundle/deb/"*.deb "$OUT/" 2>/dev/null || true

node "$ROOT/scripts/generate-latest-json.mjs" "$VERSION" "$TAG" "$OUT"

echo ""
echo "Upload everything in release-assets/ to GitHub Release $TAG:"
echo "  https://github.com/alfodri/HR-APP/releases/new?tag=$TAG"
echo "Required for updater: latest.json + .AppImage + .AppImage.sig"
ls -la "$OUT"
