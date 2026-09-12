#!/usr/bin/env python3
"""Convert KRI k3mesh (claymore v0.1 / claymore_load) to a minimal glTF 2.0 GLB.

Chunk layout (8-byte name + u32 size), matching claymore_load chunk.rs / mesh.rs:
  root walks `mesh` chunks
  mesh: name (u8-len str), n_vert u32, topology str ("3"=tris), then `buffer` / `index`
  buffer: stride u8, format like `3f4h2h` (python-struct counts), interleaved verts,
          then per-attr name (str) + flags u8
  index: n_ind u32, type char B/H/L, then index bytes

Usage:
  python3 scripts/k3mesh_to_glb.py INPUT.k3mesh OUTPUT.glb [--height 1.0]
  python3 scripts/k3mesh_to_glb.py /path/to/scene/all.k3mesh assets/models/vika.glb

Merges all meshes in the file. Embeds POSITION + NORMAL only (hunt shader is untextured).
Normalizes so the model stands on y=0 with the given height (default 1.0).
"""

from __future__ import annotations

import argparse
import json
import math
import struct
import sys
from pathlib import Path
from typing import List, Tuple


def _read_cstr8(data: bytes, pos: int) -> Tuple[str, int]:
    raw = data[pos : pos + 8]
    pos += 8
    name = raw.split(b"\x00", 1)[0].decode("utf-8", "replace")
    size = struct.unpack_from("<I", data, pos)[0]
    pos += 4
    return name, size, pos


def _read_str(data: bytes, pos: int) -> Tuple[str, int]:
    n = data[pos]
    pos += 1
    s = data[pos : pos + n].decode("utf-8", "replace")
    pos += n
    return s, pos


def _type_size(typ: str) -> int:
    return {"b": 1, "B": 1, "h": 2, "H": 2, "l": 4, "L": 4, "f": 4, "d": 8}[typ]


def _decode_attr(
    data: bytes, base: int, fmt_pair: str, flags: int
) -> Tuple[Tuple[float, ...], int]:
    cnt = int(fmt_pair[0])
    typ = fmt_pair[1]
    if typ == "f":
        vals = struct.unpack_from("<" + "f" * cnt, data, base)
        return vals, 4 * cnt
    if typ == "d":
        vals = struct.unpack_from("<" + "d" * cnt, data, base)
        return tuple(float(v) for v in vals), 8 * cnt
    if typ in "hH":
        fmtc = "h" if typ == "h" else "H"
        vals = struct.unpack_from("<" + fmtc * cnt, data, base)
        if flags & 1:
            den = 32767.0 if typ == "h" else 65535.0
            vals = tuple(v / den for v in vals)
        else:
            vals = tuple(float(v) for v in vals)
        return vals, 2 * cnt
    if typ in "bB":
        fmtc = "b" if typ == "b" else "B"
        vals = struct.unpack_from("<" + fmtc * cnt, data, base)
        if flags & 1:
            den = 127.0 if typ == "b" else 255.0
            vals = tuple(v / den for v in vals)
        else:
            vals = tuple(float(v) for v in vals)
        return vals, 1 * cnt
    if typ in "lL":
        fmtc = "i" if typ == "l" else "I"
        vals = struct.unpack_from("<" + fmtc * cnt, data, base)
        return tuple(float(v) for v in vals), 4 * cnt
    raise ValueError(f"unsupported type {typ!r}")


def load_k3mesh(path: Path):
    data = path.read_bytes()
    pos = 0
    meshes = []
    while pos < len(data):
        name, size, pos = _read_cstr8(data, pos)
        end = pos + size
        if name != "mesh":
            pos = end
            continue
        mesh_name, pos = _read_str(data, pos)
        n_vert = struct.unpack_from("<I", data, pos)[0]
        pos += 4
        topo, pos = _read_str(data, pos)
        if topo not in ("3", "3s", "3f"):
            print(f"warning: skipping mesh {mesh_name!r} topology {topo!r}", file=sys.stderr)
            pos = end
            continue
        positions: List[Tuple[float, float, float]] = []
        normals: List[Tuple[float, float, float]] = []
        indices: List[int] = []
        while pos < end:
            cname, csize, pos = _read_cstr8(data, pos)
            cend = pos + csize
            if cname == "buffer":
                stride = data[pos]
                pos += 1
                fmt, pos = _read_str(data, pos)
                vstart = pos
                pos += n_vert * stride
                attrs = []
                while pos < cend:
                    aname, pos = _read_str(data, pos)
                    flags = data[pos]
                    pos += 1
                    attrs.append((aname, flags))
                pairs = [fmt[i : i + 2] for i in range(0, len(fmt), 2)]
                if len(pairs) != len(attrs):
                    raise ValueError(
                        f"format/attr mismatch in {mesh_name}: {fmt} vs {attrs}"
                    )
                # layout offsets
                offsets = []
                off = 0
                for pair, (aname, flags) in zip(pairs, attrs):
                    offsets.append(off)
                    off += int(pair[0]) * _type_size(pair[1])
                if off != stride:
                    raise ValueError(
                        f"stride mismatch in {mesh_name}: layout {off} vs {stride}"
                    )
                pos_i = next((i for i, (n, _) in enumerate(attrs) if n == "Position"), None)
                nrm_i = next((i for i, (n, _) in enumerate(attrs) if n == "Normal"), None)
                if pos_i is None:
                    raise ValueError(f"no Position in {mesh_name}")
                for vi in range(n_vert):
                    base = vstart + vi * stride
                    pvals, _ = _decode_attr(
                        data, base + offsets[pos_i], pairs[pos_i], attrs[pos_i][1]
                    )
                    positions.append((pvals[0], pvals[1], pvals[2]))
                    if nrm_i is not None:
                        nvals, _ = _decode_attr(
                            data, base + offsets[nrm_i], pairs[nrm_i], attrs[nrm_i][1]
                        )
                        nx, ny, nz = nvals[0], nvals[1], nvals[2]
                        ln = math.sqrt(nx * nx + ny * ny + nz * nz) or 1.0
                        normals.append((nx / ln, ny / ln, nz / ln))
                    else:
                        normals.append((0.0, 1.0, 0.0))
                assert pos == cend or pos <= cend
                pos = cend
            elif cname == "index":
                n_ind = struct.unpack_from("<I", data, pos)[0]
                pos += 4
                itype = chr(data[pos])
                pos += 1
                es = {"B": 1, "H": 2, "L": 4}[itype]
                fmtc = {"B": "B", "H": "H", "L": "I"}[itype]
                for i in range(n_ind):
                    indices.append(struct.unpack_from("<" + fmtc, data, pos + i * es)[0])
                pos = cend
            else:
                print(f"warning: ignoring chunk {cname!r}", file=sys.stderr)
                pos = cend
        meshes.append(
            {
                "name": mesh_name,
                "positions": positions,
                "normals": normals,
                "indices": indices,
            }
        )
        pos = end
    return meshes


def merge_meshes(meshes):
    positions = []
    normals = []
    indices = []
    base = 0
    for m in meshes:
        positions.extend(m["positions"])
        normals.extend(m["normals"])
        indices.extend(i + base for i in m["indices"])
        base += len(m["positions"])
    return positions, normals, indices


def normalize(positions, normals, height: float):
    if not positions:
        return positions, normals
    xs = [p[0] for p in positions]
    ys = [p[1] for p in positions]
    zs = [p[2] for p in positions]
    min_y = min(ys)
    cx = 0.5 * (min(xs) + max(xs))
    cz = 0.5 * (min(zs) + max(zs))
    h = max(ys) - min_y
    if h < 1e-6:
        h = 1.0
    scale = height / h
    out_p = [((p[0] - cx) * scale, (p[1] - min_y) * scale, (p[2] - cz) * scale) for p in positions]
    # normals only rotate/scale uniformly — leave as-is (already unit)
    return out_p, normals


def write_glb(path: Path, positions, normals, indices):
    n_vert = len(positions)
    n_ind = len(indices)
    if n_vert == 0 or n_ind == 0:
        raise ValueError("empty mesh")

    # BIN: POSITION (f32*3) | NORMAL (f32*3) | INDICES (u16 or u32)
    pos_bytes = b"".join(struct.pack("<3f", *p) for p in positions)
    nrm_bytes = b"".join(struct.pack("<3f", *n) for n in normals)
    use_u32 = n_vert > 65535 or max(indices) > 65535
    if use_u32:
        idx_bytes = b"".join(struct.pack("<I", i) for i in indices)
        idx_comp = 5125  # UNSIGNED_INT
    else:
        idx_bytes = b"".join(struct.pack("<H", i) for i in indices)
        idx_comp = 5123  # UNSIGNED_SHORT
        if len(idx_bytes) % 4:
            idx_bytes += b"\x00\x00"  # align

    # pad each section to 4 bytes
    def pad4(b: bytes) -> bytes:
        return b + (b"\x00" * ((4 - (len(b) % 4)) % 4))

    pos_bytes = pad4(pos_bytes)
    nrm_bytes = pad4(nrm_bytes)
    idx_bytes = pad4(idx_bytes)

    pos_off = 0
    nrm_off = len(pos_bytes)
    idx_off = nrm_off + len(nrm_bytes)
    bin_blob = pos_bytes + nrm_bytes + idx_bytes

    min_p = [min(p[i] for p in positions) for i in range(3)]
    max_p = [max(p[i] for p in positions) for i in range(3)]

    gltf = {
        "asset": {"version": "2.0", "generator": "claymore-blade k3mesh_to_glb"},
        "buffers": [{"byteLength": len(bin_blob)}],
        "bufferViews": [
            {"buffer": 0, "byteOffset": pos_off, "byteLength": len(pos_bytes), "target": 34962},
            {"buffer": 0, "byteOffset": nrm_off, "byteLength": len(nrm_bytes), "target": 34962},
            {"buffer": 0, "byteOffset": idx_off, "byteLength": len(idx_bytes), "target": 34963},
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": n_vert,
                "type": "VEC3",
                "max": max_p,
                "min": min_p,
            },
            {
                "bufferView": 1,
                "componentType": 5126,
                "count": n_vert,
                "type": "VEC3",
            },
            {
                "bufferView": 2,
                "componentType": idx_comp,
                "count": n_ind,
                "type": "SCALAR",
            },
        ],
        "meshes": [
            {
                "name": path.stem,
                "primitives": [
                    {
                        "attributes": {"POSITION": 0, "NORMAL": 1},
                        "indices": 2,
                        "mode": 4,
                    }
                ],
            }
        ],
        "nodes": [{"mesh": 0, "name": path.stem}],
        "scenes": [{"nodes": [0]}],
        "scene": 0,
    }

    json_bytes = json.dumps(gltf, separators=(",", ":")).encode("utf-8")
    json_bytes = pad4(json_bytes)
    # GLB prefers space padding for JSON
    while len(json_bytes) % 4:
        json_bytes += b" "

    total = 12 + 8 + len(json_bytes) + 8 + len(bin_blob)
    header = struct.pack("<4sII", b"glTF", 2, total)
    json_chunk = struct.pack("<I4s", len(json_bytes), b"JSON") + json_bytes
    bin_chunk = struct.pack("<I4s", len(bin_blob), b"BIN\x00") + bin_blob
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(header + json_chunk + bin_chunk)
    print(
        f"wrote {path} verts={n_vert} indices={n_ind} bytes={path.stat().st_size}"
    )


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("input", type=Path, help="path to .k3mesh")
    ap.add_argument("output", type=Path, help="path to .glb")
    ap.add_argument(
        "--height",
        type=float,
        default=1.0,
        help="normalize model height (Y) to this value, feet on y=0 (default 1.0)",
    )
    args = ap.parse_args()
    meshes = load_k3mesh(args.input)
    if not meshes:
        sys.exit(f"no meshes in {args.input}")
    print(f"loaded {len(meshes)} mesh(es) from {args.input}:")
    for m in meshes:
        print(f"  {m['name']!r}: {len(m['positions'])} verts, {len(m['indices'])} indices")
    positions, normals, indices = merge_meshes(meshes)
    positions, normals = normalize(positions, normals, args.height)
    write_glb(args.output, positions, normals, indices)


if __name__ == "__main__":
    main()
