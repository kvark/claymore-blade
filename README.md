# Claymore

Roleplaying tactical hunt. Island map, isometric hex combat, Claymore series roster.

One Rust crate. [Blade](https://github.com/kvark/blade) draws the same hunt on desktop (Vulkan) and in the browser (WebGL2).

Play: [kvark.github.io/claymore-blade](https://kvark.github.io/claymore-blade/)  
Repo: [github.com/kvark/claymore-blade](https://github.com/kvark/claymore-blade)

## Layout

```
src/           game, combat, hex, Blade renderer, juice, HUD
assets/        art, sprites, Kenney CC0, WGSL
assets/kenney  UI, props, FX, prompts, SFX (CC0, Kenney)
web/           html shell for wasm
scripts/       wasm build + static serve
```

## Native

```bash
cargo run
cargo test
```

Needs a GPU. Saves to `claymore.save.json`.

## Web

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.127 --locked
bash scripts/build-web.sh
```

Output is `dist/web`. GitHub Pages deploys that folder from `main`.

## Controls

| | |
| --- | --- |
| Title | New Hunt / Continue |
| Island | WASD walk (A left, D right, W up the map), click a town |
| Town | Hunt / Rest / Leave |
| Hunt | click a hex to step, Cut / Guard / Wait / Raise on the bar, drag to pan, wheel to zoom |
| Keys | 1 Cut, 2 Guard, 3 Aimed, T raise trans, Space wait, Esc back |

Terrain/UI/SFX: [Kenney](https://kenney.nl) CC0. Faces and awakened bodies are original.

Design: [`docs/DESIGN.md`](docs/DESIGN.md).
