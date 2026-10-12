use crate::{ChunkDemo, chunks::helpers};

/// Smooth 2D value noise in 0..1.
fn value_noise(x: f32, z: f32, seed: u32) -> f32 {
    let (xi, zi) = (x.floor(), z.floor());
    let (fx, fz) = (x - xi, z - zi);
    let (sx, sz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let corner = |dx: i32, dz: i32| {
        (helpers::hash3(xi as i32 + dx, 0, zi as i32 + dz, seed) & 0xFFFF) as f32 / 65535.0
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

pub fn terrain(material_at: impl Fn(i32, i32, i32, i32) -> u16 + 'static) -> ChunkDemo {
    let heights = heightmap();
    ChunkDemo::new("heightmap_terrain", move |coords| {
        let surface = heights[coords.z as usize][coords.x as usize];
        if coords.y < surface as u8 {
            material_at(coords.x as i32, coords.y as i32, coords.z as i32, surface)
        } else {
            0
        }
    })
}
