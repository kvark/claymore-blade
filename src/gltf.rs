//! Minimal glTF 2.0 / GLB loader for hunt meshes (POSITION + NORMAL + indices).

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Expand indexed triangles into a flat vertex list for the hunt pipeline.
    /// Static skin: joint 0, weight 0xFF.
    pub fn to_skinned_vertices(&self) -> Vec<([f32; 3], [f32; 3], u32, u32)> {
        let mut out = Vec::with_capacity(self.indices.len());
        let w = 0x0000_00FFu32;
        for &i in &self.indices {
            let i = i as usize;
            let pos = self.positions.get(i).copied().unwrap_or([0.0; 3]);
            let nrm = self.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
            out.push((pos, nrm, 0, w));
        }
        out
    }
}

#[derive(Debug)]
pub enum Error {
    Msg(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Msg(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Error {}

fn err(s: impl Into<String>) -> Error {
    Error::Msg(s.into())
}

/// Parse a `.glb` (binary glTF 2.0) and return the first mesh's POSITION/NORMAL/indices.
pub fn load_glb(bytes: &[u8]) -> Result<MeshData, Error> {
    if bytes.len() < 12 {
        return Err(err("glb too short"));
    }
    if &bytes[0..4] != b"glTF" {
        return Err(err("missing glTF magic"));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
    if version != 2 {
        return Err(err(format!("unsupported glTF version {version}")));
    }
    let total = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    if total > bytes.len() {
        return Err(err("glb length exceeds buffer"));
    }

    let mut json: Option<&str> = None;
    let mut bin: Option<&[u8]> = None;
    let mut off = 12usize;
    while off + 8 <= bytes.len() {
        let chunk_len = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap()) as usize;
        let chunk_ty = &bytes[off + 4..off + 8];
        off += 8;
        if off + chunk_len > bytes.len() {
            return Err(err("chunk overrun"));
        }
        let payload = &bytes[off..off + chunk_len];
        off += chunk_len;
        match chunk_ty {
            b"JSON" => {
                let text = std::str::from_utf8(payload).map_err(|e| err(e.to_string()))?;
                json = Some(text.trim_end_matches('\0').trim_end());
            }
            b"BIN\0" => bin = Some(payload),
            _ => {}
        }
    }
    let json = json.ok_or_else(|| err("no JSON chunk"))?;
    let bin = bin.unwrap_or(&[]);
    parse_gltf_json(json, bin)
}

fn parse_gltf_json(json: &str, bin: &[u8]) -> Result<MeshData, Error> {
    let root: serde_json::Value = serde_json::from_str(json).map_err(|e| err(e.to_string()))?;
    let meshes = root
        .get("meshes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| err("no meshes"))?;
    let prim = meshes
        .first()
        .and_then(|m| m.get("primitives"))
        .and_then(|p| p.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| err("no primitives"))?;
    let attrs = prim
        .get("attributes")
        .and_then(|v| v.as_object())
        .ok_or_else(|| err("no attributes"))?;
    let pos_acc = attrs
        .get("POSITION")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| err("no POSITION"))? as usize;
    let nrm_acc = attrs.get("NORMAL").and_then(|v| v.as_u64()).map(|v| v as usize);
    let idx_acc = prim
        .get("indices")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| err("no indices"))? as usize;

    let accessors = root
        .get("accessors")
        .and_then(|v| v.as_array())
        .ok_or_else(|| err("no accessors"))?;
    let views = root
        .get("bufferViews")
        .and_then(|v| v.as_array())
        .ok_or_else(|| err("no bufferViews"))?;

    let positions = read_f32x3(accessors, views, bin, pos_acc)?;
    let normals = if let Some(ni) = nrm_acc {
        read_f32x3(accessors, views, bin, ni)?
    } else {
        vec![[0.0, 1.0, 0.0]; positions.len()]
    };
    let indices = read_indices(accessors, views, bin, idx_acc)?;
    if positions.is_empty() {
        return Err(err("empty positions"));
    }
    Ok(MeshData {
        positions,
        normals,
        indices,
    })
}

fn acc_view<'a>(
    accessors: &'a [serde_json::Value],
    views: &'a [serde_json::Value],
    bin: &'a [u8],
    acc_i: usize,
) -> Result<(&'a [u8], usize, u32, Option<usize>), Error> {
    let acc = accessors.get(acc_i).ok_or_else(|| err("accessor OOB"))?;
    let count = acc.get("count").and_then(|v| v.as_u64()).ok_or_else(|| err("no count"))? as usize;
    let comp = acc
        .get("componentType")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| err("no componentType"))? as u32;
    let view_i = acc
        .get("bufferView")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| err("no bufferView"))? as usize;
    let acc_off = acc.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let view = views.get(view_i).ok_or_else(|| err("view OOB"))?;
    let view_off = view.get("byteOffset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let view_len = view
        .get("byteLength")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| err("no byteLength"))? as usize;
    let stride = view.get("byteStride").and_then(|v| v.as_u64()).map(|v| v as usize);
    let start = view_off + acc_off;
    if start > bin.len() || view_off + view_len > bin.len() {
        return Err(err("buffer overrun"));
    }
    Ok((&bin[start..], count, comp, stride))
}

fn read_f32x3(
    accessors: &[serde_json::Value],
    views: &[serde_json::Value],
    bin: &[u8],
    acc_i: usize,
) -> Result<Vec<[f32; 3]>, Error> {
    let (data, count, comp, stride) = acc_view(accessors, views, bin, acc_i)?;
    if comp != 5126 {
        return Err(err("POSITION/NORMAL must be FLOAT"));
    }
    let step = stride.unwrap_or(12);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = i * step;
        if o + 12 > data.len() {
            return Err(err("vec3 overrun"));
        }
        let x = f32::from_le_bytes(data[o..o + 4].try_into().unwrap());
        let y = f32::from_le_bytes(data[o + 4..o + 8].try_into().unwrap());
        let z = f32::from_le_bytes(data[o + 8..o + 12].try_into().unwrap());
        out.push([x, y, z]);
    }
    Ok(out)
}

fn read_indices(
    accessors: &[serde_json::Value],
    views: &[serde_json::Value],
    bin: &[u8],
    acc_i: usize,
) -> Result<Vec<u32>, Error> {
    let (data, count, comp, stride) = acc_view(accessors, views, bin, acc_i)?;
    let (esize, read): (usize, fn(&[u8]) -> u32) = match comp {
        5121 => (1, |b| b[0] as u32),
        5123 => (2, |b| u16::from_le_bytes(b[0..2].try_into().unwrap()) as u32),
        5125 => (4, |b| u32::from_le_bytes(b[0..4].try_into().unwrap())),
        _ => return Err(err(format!("bad index componentType {comp}"))),
    };
    let step = stride.unwrap_or(esize);
    let mut out = Vec::with_capacity(count);
    for i in 0..count {
        let o = i * step;
        if o + esize > data.len() {
            return Err(err("index overrun"));
        }
        out.push(read(&data[o..o + esize]));
    }
    Ok(out)
}

/// Load a hunt model from `assets/{rel}` (e.g. `models/vika.glb`).
pub fn load_asset(rel: &str) -> Result<MeshData, Error> {
    let bytes = crate::io::read_bytes(rel).map_err(err)?;
    load_glb(&bytes)
}

/// Convenience map of known archive models.
pub fn load_archive_models() -> HashMap<&'static str, MeshData> {
    let mut map = HashMap::new();
    for (key, path) in [
        ("vika", "models/vika.glb"),
        ("valefor", "models/valefor.glb"),
        ("ifrit", "models/ifrit.glb"),
        ("kachujin", "models/kachujin.glb"),
    ] {
        match load_asset(path) {
            Ok(m) => {
                log::info!("gltf {path}: {} verts, {} indices", m.vertex_count(), m.indices.len());
                map.insert(key, m);
            }
            Err(e) => log::warn!("gltf {path}: {e}"),
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vika_glb_has_vertices() {
        let bytes = std::fs::read("assets/models/vika.glb")
            .expect("assets/models/vika.glb must exist for tests");
        let mesh = load_glb(&bytes).expect("parse vika.glb");
        assert!(mesh.vertex_count() > 0, "vika should have vertices");
        assert!(mesh.indices.len() > 0, "vika should have indices");
        assert_eq!(mesh.indices.len() % 3, 0);
        let skinned = mesh.to_skinned_vertices();
        assert_eq!(skinned.len(), mesh.indices.len());
        assert!(skinned[0].2 == 0 && skinned[0].3 == 0xFF);
    }

    #[test]
    fn valefor_glb_parses() {
        let bytes = std::fs::read("assets/models/valefor.glb").expect("valefor.glb");
        let mesh = load_glb(&bytes).expect("parse valefor");
        assert!(mesh.vertex_count() > 100);
    }
}
