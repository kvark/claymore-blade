#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
TARGET="${CARGO_TARGET_DIR:-target}"
echo "building claymore wasm"
cargo build --release --target wasm32-unknown-unknown --lib
wasm-bindgen --target web --no-typescript \
  --out-dir dist/web \
  --out-name claymore \
  "$TARGET/wasm32-unknown-unknown/release/claymore.wasm"
cp -f web/index.html dist/web/index.html
cp -f assets/favicon.svg assets/og.jpg assets/x-banner.jpg dist/web/ 2>/dev/null || true
rm -rf dist/web/art dist/web/sprites
cp -a assets/art dist/web/art
cp -a assets/sprites dist/web/sprites
touch dist/web/.nojekyll
echo "web build -> dist/web"
