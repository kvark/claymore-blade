//! CPU hunt board. The web canvas and the Blade rasterizer both consume this.

use crate::hex::Axial;
use crate::iso::terrain_height;
use crate::prism::{tile_instance, TileInstance};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub hex: Axial,
    pub terrain: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HuntBoard {
    pub cols: i32,
    pub rows: i32,
    pub size: f32,
    pub tiles: Vec<Tile>,
}

impl HuntBoard {
    pub fn new(cols: i32, rows: i32, size: f32) -> Self {
        let mut tiles = Vec::with_capacity((cols * rows) as usize);
        for q in 0..cols {
            for r in 0..rows {
                tiles.push(Tile {
                    hex: Axial::new(q, r),
                    terrain: "grass".into(),
                });
            }
        }
        Self {
            cols,
            rows,
            size,
            tiles,
        }
    }

    pub fn instances(&self) -> Vec<TileInstance> {
        self.tiles
            .iter()
            .map(|t| {
                let h = terrain_height(&t.terrain, self.size);
                let color = match t.terrain.as_str() {
                    "water" => [0.18, 0.24, 0.27, 1.0],
                    "mud" => [0.30, 0.22, 0.16, 1.0],
                    "ruin" => [0.30, 0.29, 0.28, 1.0],
                    _ => [0.19, 0.20, 0.16, 1.0],
                };
                tile_instance(t.hex, self.size, h, color)
            })
            .collect()
    }

    pub fn center(&self) -> Axial {
        Axial::new(self.cols / 2, self.rows / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instances_match_tiles() {
        let board = HuntBoard::new(11, 9, 1.0);
        assert_eq!(board.instances().len(), 11 * 9);
        assert_eq!(board.center(), crate::Axial::new(5, 4));
    }
}
