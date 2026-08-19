# Claymore

Roleplaying tactical hunt. Island map, isometric hex combat, Claymore series roster. Built on [Blade](https://github.com/kvark/blade).

Play: [kvark.github.io/claymore-blade](https://kvark.github.io/claymore-blade/)  
Repo: [github.com/kvark/claymore-blade](https://github.com/kvark/claymore-blade)

## One crate

```
rust/          hex, combat, iso camera, hex-prism mesh, hunt board
src/game/      web slice (same rules, Canvas 2D until wasm Rasterizer)
```

```bash
cargo test
npm install
npm run dev          # live preview
npm run build:pages  # static SPA for GitHub Pages
```

Hunt is isometric (drag / wheel). GitHub Pages deploys from `main` via Actions. Design: [`artifacts/DESIGN.md`](artifacts/DESIGN.md).

Kenney pull list: design doc §15. First: Hexagon Pack, Isometric Nature, Isometric Miniature Dungeon.
