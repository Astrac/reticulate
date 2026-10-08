use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use reticulate::{ChunkCoord, CompactOctree, Coords3D, Fixed8Octree, Mesher};

pub fn make_sphere<Data: Clone + PartialEq>(
    tree: &mut Fixed8Octree<Data>,
    center: Coords3D,
    radius: u8,
    mut data: impl FnMut(Coords3D) -> Data,
) {
    let (center_x, center_y, center_z) = (center.x, center.y, center.z);
    let radius_squared = radius * radius;

    let lower_bound = |center: u8| center.saturating_sub(radius);
    let higher_bound = |center: u8| (center + radius).min(31);

    for x in lower_bound(center_x)..=higher_bound(center_x) {
        let dx = x as i32 - center_x as i32;
        for y in lower_bound(center_y)..=higher_bound(center_y) {
            let dy = y as i32 - center_y as i32;
            for z in lower_bound(center_z)..=higher_bound(center_z) {
                let dz = z as i32 - center_z as i32;
                if dx * dx + dy * dy + dz * dz <= radius_squared as i32 {
                    let coords = Coords3D {
                        x: x as ChunkCoord,
                        y: y as ChunkCoord,
                        z: z as ChunkCoord,
                    };

                    tree.update(coords, Some(data(coords)));
                }
            }
        }
    }
}

fn bench_fixed_8_sphere_single_mat(c: &mut Criterion) {
    let center = Coords3D {
        x: 16,
        y: 16,
        z: 16,
    };

    let mut tree = Fixed8Octree::new();
    make_sphere(&mut tree, center, 14, |_| 1u16);

    let mut mesher = Mesher::new();
    c.bench_function("fixed_8_sphere_single_mat", |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        })
    });
}

fn bench_compact_sphere_single_mat(c: &mut Criterion) {
    let center = Coords3D {
        x: 16,
        y: 16,
        z: 16,
    };

    let mut tree = Fixed8Octree::new();
    make_sphere(&mut tree, center, 14, |_| 1u16);

    let tree = CompactOctree::from_fixed8(&tree);

    let mut mesher = Mesher::new();
    c.bench_function("compact_sphere_single_mat", |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        })
    });
}

pub fn bench_full_checkerboard(c: &mut Criterion) {
    let mut tree = Fixed8Octree::new();

    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                if (x + y + z) % 2 == 0 {
                    tree.update(Coords3D { x, y, z }, Some(1));
                }
            }
        }
    }

    let mut mesher = Mesher::new();
    c.bench_function("full_checkerboard", |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        })
    });
}

pub fn bench_3d_ortho_labyrinth_fixed_8(c: &mut Criterion) {
    let mut tree = Fixed8Octree::new();

    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let mut is_solid = false;

                if y % 4 == 0 && z % 4 == 0 {
                    is_solid = true;
                }

                if x % 4 == 2 && z % 4 == 2 {
                    is_solid = true;
                }

                if x % 4 == 0 && y % 4 == 2 {
                    is_solid = true;
                }

                if is_solid {
                    let coords = Coords3D { x, y, z };
                    tree.update(coords, Some(1));
                }
            }
        }
    }

    let mut mesher = Mesher::new();
    c.bench_function("3d_ortho_labyrinth_fixed_8", |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        })
    });
}

pub fn bench_3d_ortho_labyrinth_compact(c: &mut Criterion) {
    let mut tree = Fixed8Octree::new();

    for z in 0..32 {
        for y in 0..32 {
            for x in 0..32 {
                let mut is_solid = false;

                if y % 4 == 0 && z % 4 == 0 {
                    is_solid = true;
                }

                if x % 4 == 2 && z % 4 == 2 {
                    is_solid = true;
                }

                if x % 4 == 0 && y % 4 == 2 {
                    is_solid = true;
                }

                if is_solid {
                    let coords = Coords3D { x, y, z };
                    tree.update(coords, Some(1));
                }
            }
        }
    }

    let tree = CompactOctree::from_fixed8(&tree);

    let mut mesher = Mesher::new();
    c.bench_function("3d_ortho_labyrinth_compact", |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        })
    });
}

pub fn bench_terrain_single_material(c: &mut Criterion) {
    bench_terrain(c, "terrain_single_material", single_material);
}

pub fn bench_terrain_layered(c: &mut Criterion) {
    bench_terrain(c, "terrain_layered", layered_material);
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
        bench_fixed_8_sphere_single_mat,
        bench_compact_sphere_single_mat,
        bench_full_checkerboard,
        bench_3d_ortho_labyrinth_fixed_8,
        bench_3d_ortho_labyrinth_compact,
        bench_terrain_single_material,
        bench_terrain_layered,
);

criterion_main!(benches);

// ---- terrain helpers ----

fn hash3(x: i32, y: i32, z: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1)
        ^ (y as u32).wrapping_mul(0x85EB_CA77)
        ^ (z as u32).wrapping_mul(0xC2B2_AE3D)
        ^ seed.wrapping_mul(0x27D4_EB2F);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    h
}

/// Smooth 2D value noise in 0..1.
fn value_noise(x: f32, z: f32, seed: u32) -> f32 {
    let (xi, zi) = (x.floor(), z.floor());
    let (fx, fz) = (x - xi, z - zi);
    let (sx, sz) = (fx * fx * (3.0 - 2.0 * fx), fz * fz * (3.0 - 2.0 * fz));
    let corner = |dx: i32, dz: i32| {
        (hash3(xi as i32 + dx, 0, zi as i32 + dz, seed) & 0xFFFF) as f32 / 65535.0
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

fn build_terrain(material_at: impl Fn(i32, i32, i32, i32) -> u16) -> Fixed8Octree<u16> {
    let heights = heightmap();
    let mut tree = Fixed8Octree::new();
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
    tree
}

// ---- materials ----

fn single_material(_x: i32, _y: i32, _z: i32, _surface: i32) -> u16 {
    1
}

/// 1 grass, 2 dirt, 3 stone, 4 ore, 5 sand (low ground)
fn layered_material(x: i32, y: i32, z: i32, surface: i32) -> u16 {
    let depth = surface - y; // 1 = topmost voxel
    let top = if surface < 10 { 5 } else { 1 };
    // Jitter the dirt/stone boundary per column so strata aren't flat.
    let dirt_depth = 3 + (hash3(x, 0, z, 7) % 3) as i32;
    if depth == 1 {
        top
    } else if depth <= dirt_depth {
        2
    } else if hash3(x, y, z, 11) % 48 == 0 {
        4
    } else {
        3
    }
}

// ---- benchmarks ----

fn bench_terrain(c: &mut Criterion, name: &str, material_at: fn(i32, i32, i32, i32) -> u16) {
    let tree = build_terrain(material_at);
    let compact = CompactOctree::from_fixed8(&tree);
    let mut mesher = Mesher::new();

    c.bench_function(&format!("{name}_fixed8"), |b| {
        b.iter(|| mesher.make_triangles(black_box(&tree), |d| *d, |_| ()))
    });
    c.bench_function(&format!("{name}_compact"), |b| {
        b.iter(|| mesher.make_triangles(black_box(&compact), |d| *d, |_| ()))
    });
}
