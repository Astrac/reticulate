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

pub fn bench_3d_ortho_labyrinth(c: &mut Criterion) {
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
    c.bench_function("3d_ortho_labyrinth", |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
        bench_fixed_8_sphere_single_mat,
        bench_compact_sphere_single_mat,
        bench_full_checkerboard,
        bench_3d_ortho_labyrinth,
);

criterion_main!(benches);
