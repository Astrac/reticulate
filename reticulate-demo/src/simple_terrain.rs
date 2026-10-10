use reticulate::{Coords3D, Fixed8Octree};

use crate::utils;

/// Smooth 2D value noise in 0..1.
fn value_noise(x: f32, z: f32, seed: u32) -> f32 {
    let (xi, zi) = (x.floor(), z.floor());
    let (fx, fz) = (x - xi, z - zi);
    let (sx, sz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let corner = |dx: i32, dz: i32| {
        (utils::hash3(xi as i32 + dx, 0, zi as i32 + dz, seed) & 0xFFFF) as f32 / 65535.0
    };
    let top = corner(0, 0) + (corner(1, 0) - corner(0, 0)) * sx;
    let bottom = corner(0, 1) + (corner(1, 1) - corner(0, 1)) * sx;
    top + (bottom - top) * sz
}

/// Surface height per (x, z) column, roughly 6..26 in a 32-high chunk.
fn heightmap() -> [[i32; 32]; 32] {
    let mut h = [[0i32; 32]; 32];
    for z in 0..32 {
        for x in 0..32 {
            let (fx, fz) = (x as f32, z as f32);
            let n = 0.65 * value_noise(fx / 16.0, fz / 16.0, 1)
                + 0.25 * value_noise(fx / 8.0, fz / 8.0, 2)
                + 0.10 * value_noise(fx / 4.0, fz / 4.0, 3);
            h[z][x] = 6 + (n * 20.0) as i32;
        }
    }
    h
}

pub fn chunk_terrain(
    tree: &mut Fixed8Octree<u16>,
    material_at: impl Fn(i32, i32, i32, i32) -> u16,
) {
    let heights = heightmap();
    for z in 0..32u8 {
        for y in 0..32u8 {
            for x in 0..32u8 {
                let surface = heights[z as usize][x as usize];
                if y < surface as u8 {
                    let m = material_at(x as i32, y as i32, z as i32, surface);
                    tree.update(Coords3D { x, y, z }, Some(m));
                }
            }
        }
    }
}
