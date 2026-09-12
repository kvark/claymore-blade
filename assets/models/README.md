# Archive 3D models

Archive meshes serve the Claymore anime tactical-hunt look (series warriors / yoma), not a generic dark-fantasy skin.

Converted from [kvark/claymore v0.1 `data.tgz`](https://github.com/kvark/claymore/releases/tag/v0.1) KRI `k3mesh` scenes (Blender exporter + rust `claymore_load`).

| GLB | Source scene | Role in hunt |
|-----|--------------|--------------|
| `vika.glb` | `vika/all.k3mesh` | Clare / player fighters |
| `valefor.glb` | `valefor/all.k3mesh` | Yoma / enemies |
| `ifrit.glb` | `ifrit/all.k3mesh` | optional monster alt |
| `kachujin.glb` | `kachujin/all.k3mesh` | optional humanoid |

Stored with Git LFS (`*.glb`). Textures from the archive are **not** shipped — the hunt shader is untextured lit color (positions + normals).

## Convert

```bash
python3 scripts/k3mesh_to_glb.py /path/to/all.k3mesh assets/models/NAME.glb
# optional: --height 1.0  (normalize so feet sit on y=0 and height is 1)
```

Re-run after extracting `data.tgz`; commit new GLBs via LFS.
