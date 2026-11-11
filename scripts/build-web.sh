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
rm -rf dist/web/art dist/web/sprites dist/web/kenney dist/web/models
cp -a assets/art dist/web/art
cp -a assets/sprites dist/web/sprites
cp -a assets/kenney dist/web/kenney
# Hunt models (src/gltf.rs, src/gpu/create.rs) load `models/*.glb` next to the
# page. They live in Git LFS: a checkout without LFS leaves text pointers.
is_lfs_pointer() {
  head -c 40 "$1" | grep -q '^version https://git-lfs'
}
need_lfs=0
for f in assets/models/*.glb; do
  is_lfs_pointer "$f" && need_lfs=1
done
if [ "$need_lfs" = 1 ] && command -v git-lfs >/dev/null 2>&1; then
  echo "fetching Git LFS models"
  git lfs pull --include="assets/models/*.glb" || true
fi
mkdir -p dist/web/models
for f in assets/models/*.glb; do
  if is_lfs_pointer "$f"; then
    echo "warning: $f is a Git LFS pointer, not shipping it (checkout with lfs: true)" >&2
  else
    cp -f "$f" dist/web/models/
  fi
done
touch dist/web/.nojekyll
echo "web build -> dist/web"
