pub mod chunk_material;
pub use simple_terrain::chunk_terrain;

mod simple_terrain;
mod utils;

use reticulate::{ChunkCoord, Coords3D, Fixed8Octree};

pub fn chunk_sphere<Data: Clone + PartialEq>(
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

pub fn chunk_checkerboard<Data: Clone + PartialEq>(
    tree: &mut Fixed8Octree<Data>,
    mut data: impl FnMut(Coords3D) -> Data,
) {
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                if (x + y + z) % 2 == 0 {
                    let coords = Coords3D { x, y, z };
                    tree.update(coords, Some(data(coords)));
                }
            }
        }
    }
}

pub fn chunk_ortho_labyrinth<Data: Clone + PartialEq>(
    tree: &mut Fixed8Octree<Data>,
    mut data: impl FnMut(Coords3D) -> Data,
) {
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
                    tree.update(coords, Some(data(coords)));
                }
            }
        }
    }
}

pub fn chunk_multi_material_non_minimal_triangles_example<Data: Clone + PartialEq>(
    tree: &mut Fixed8Octree<Data>,
    floor: Data,
    wall: Data,
) {
    let voxels = [
        ((2, 1, 1), &floor),
        ((2, 1, 2), &floor),
        ((3, 1, 2), &floor),
        ((3, 1, 1), &wall),
        ((3, 2, 1), &wall),
        ((3, 2, 2), &wall),
    ];

    for ((x, y, z), material) in voxels {
        tree.update(Coords3D { x, y, z }, Some(material.clone()));
    }
}
