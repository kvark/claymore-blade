# Claymore

Roleplaying tactical hunt. Island exploration, turn-based hex combat, original Claymore series roster. Built on [Blade](https://github.com/kvark/blade).

Repo: [github.com/kvark/claymore-blade](https://github.com/kvark/claymore-blade)

## Two surfaces, one ruleset

| | Web slice (this preview) | Blade |
| --- | --- | --- |
| Rules | `src/game/sim` | `crates/claymore-sim` |
| Hunt camera | isometric hex prisms (`src/game/render/hex-canvas.tsx`) | `crates/claymore-scene` + `blade-render::Rasterizer` |
| GPU | Canvas 2D projecting the same mesh | `blade-graphics` — **WebGL2** on wasm, Vulkan/Metal/GLES native |
| RT | — | optional native `RenderBackend::RayTracer` only |

The hunt is **isometric**, not top-down. Multi-cell bosses occupy several prisms. Design: [`artifacts/DESIGN.md`](artifacts/DESIGN.md).

## Rust

```bash
cargo test -p claymore-sim -p claymore-scene -p claymore-view
```

`claymore-view` `--features gpu` pulls `blade-graphics` + `blade-render` and is the Rasterizer / WebGL entry. The CPU board compiles without a GPU.

```
crates/claymore-sim     hex, hit table, LCG
crates/claymore-scene   30° camera, hex-prism mesh
crates/claymore-view    HuntBoard → Rasterizer plan
```

## Web slice

```bash
npm install
npm run dev          # 0.0.0.0:8080
npm run typecheck
```

Walk the island, take the Doga hunt, raise Trans, fight Ophelia as a 4-cell prism.

## Kenney

Do not dump the all-in-1 pack. Pull list is in the design doc §15. First: **Hexagon Pack**, **Isometric Nature**, **Isometric Miniature Dungeon**.
