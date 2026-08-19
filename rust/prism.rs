//! Procedural hex prism. Same mesh Blade instances and the canvas projects.

use bytemuck::{Pod, Zeroable};
use crate::hex::axial_to_world;
use crate::Axial;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

fn push_tri(mesh: &mut Mesh, a: Vertex, b: Vertex, c: Vertex) {
    let i = mesh.vertices.len() as u32;
    mesh.vertices.extend_from_slice(&[a, b, c]);
    mesh.indices.extend_from_slice(&[i, i + 1, i + 2]);
}

fn hex_corners_xz(cx: f32, cz: f32, radius: f32) -> [(f32, f32); 6] {
    let mut out = [(0.0, 0.0); 6];
    for i in 0..6 {
        let angle = ((60.0 * i as f32) - 30.0).to_radians();
        out[i] = (cx + radius * angle.cos(), cz + radius * angle.sin());
    }
    out
}

/// Unit prism at the origin, top at y=1, radius 1. Instance with (xz, height, tint).
pub fn unit_hex_prism(color: [f32; 4]) -> Mesh {
    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    let top = hex_corners_xz(0.0, 0.0, 1.0);
    let up = [0.0, 1.0, 0.0];
    for i in 0..6 {
        let j = (i + 1) % 6;
        let a = Vertex {
            position: [0.0, 1.0, 0.0],
            normal: up,
            color,
        };
        let b = Vertex {
            position: [top[i].0, 1.0, top[i].1],
            normal: up,
            color,
        };
        let c = Vertex {
            position: [top[j].0, 1.0, top[j].1],
            normal: up,
            color,
        };
        push_tri(&mut mesh, a, b, c);
    }
    for i in 0..6 {
        let j = (i + 1) % 6;
        let (x0, z0) = top[i];
        let (x1, z1) = top[j];
        let nx = z0 - z1;
        let nz = x1 - x0;
        let len = (nx * nx + nz * nz).sqrt().max(1e-5);
        let n = [nx / len, 0.0, nz / len];
        let v00 = Vertex {
            position: [x0, 0.0, z0],
            normal: n,
            color,
        };
        let v10 = Vertex {
            position: [x1, 0.0, z1],
            normal: n,
            color,
        };
        let v11 = Vertex {
            position: [x1, 1.0, z1],
            normal: n,
            color,
        };
        let v01 = Vertex {
            position: [x0, 1.0, z0],
            normal: n,
            color,
        };
        push_tri(&mut mesh, v00, v10, v11);
        push_tri(&mut mesh, v00, v11, v01);
    }
    mesh
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TileInstance {
    pub world: [f32; 3],
    pub height: f32,
    pub color: [f32; 4],
}

pub fn tile_instance(hex: Axial, size: f32, height: f32, color: [f32; 4]) -> TileInstance {
    let (x, z) = axial_to_world(hex, size);
    TileInstance {
        world: [x, 0.0, z],
        height,
        color,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prism_has_sides() {
        let m = unit_hex_prism([1.0; 4]);
        assert!(m.indices.len() >= 8 * 3);
        assert_eq!(m.indices.len() % 3, 0);
    }
}
