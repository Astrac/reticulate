use reticulate::{Coords3D, MaterialId};

use crate::ChunkDemo;

pub fn sphere(
    center: Coords3D,
    radius: u8,
    material: impl Fn(Coords3D) -> MaterialId + 'static,
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

pub fn tunnel(material: impl Fn(Coords3D) -> MaterialId + 'static) -> ChunkDemo {
    let mut demo = ChunkDemo::new("tunnel", move |coords| {
        if (coords.y == 1) && (coords.x == 1) {
            0
        } else {
            material(coords)
        }
    });

    demo.set_range_x(0..3);
    demo.set_range_y(0..3);
    demo.set_range_z(0..3);

    demo
}

pub fn nested_cubes(material: impl Fn(Coords3D) -> MaterialId + 'static) -> ChunkDemo {
    let mut demo = ChunkDemo::new("nested_cubes", move |coords| {
        if (coords.y == 3 || coords.y == 4)
            && (coords.x == 3 || coords.x == 4)
            && (coords.z == 3 || coords.z == 4)
        {
            0
        } else {
            material(coords)
        }
    });

    demo.set_range_x(0..8);
    demo.set_range_y(0..8);
    demo.set_range_z(0..8);

    demo
}

pub fn cube_2x2_missing_corner(material: impl Fn(Coords3D) -> MaterialId + 'static) -> ChunkDemo {
    let mut demo = ChunkDemo::new("2x2_missing_corner", move |coords| {
        if coords.x == 1 && coords.y == 1 && coords.z == 1 {
            0
        } else {
            material(coords)
        }
    });

    demo.set_range_x(0..2);
    demo.set_range_y(0..2);
    demo.set_range_z(0..2);

    demo
}
