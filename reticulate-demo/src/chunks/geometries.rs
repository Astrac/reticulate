use reticulate::{Coords3D, MaterialId};

use crate::ChunkDemo;

pub fn sphere(
    center: Coords3D,
    radius: u8,
    mut material: impl FnMut(Coords3D) -> MaterialId + 'static,
) -> ChunkDemo {
    let (center_x, center_y, center_z) = (center.x, center.y, center.z);
    let radius_squared = radius * radius;

    ChunkDemo::new("sphere", move |coords| {
        let dx = coords.x as i32 - center_x as i32;
        let dy = coords.y as i32 - center_y as i32;
        let dz = coords.z as i32 - center_z as i32;
        if dx * dx + dy * dy + dz * dz <= radius_squared as i32 {
            material(coords)
        } else {
            0
        }
    })
}
