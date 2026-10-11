use reticulate::{Coords3D, MaterialId};

use crate::ChunkDemo;

pub fn checkerboard(mut data: impl FnMut(Coords3D) -> MaterialId + 'static) -> ChunkDemo {
    ChunkDemo::new("checkerboard", move |coords| {
        if (coords.x + coords.y + coords.z) % 2 == 0 {
            data(coords)
        } else {
            0
        }
    })
}

pub fn ortho_labyrinth(mut data: impl FnMut(Coords3D) -> MaterialId + 'static) -> ChunkDemo {
    ChunkDemo::new("ortho_labyrinth", move |coords| {
        let mut is_solid = false;

        if coords.y % 4 == 0 && coords.z % 4 == 0 {
            is_solid = true;
        }

        if coords.x % 4 == 2 && coords.z % 4 == 2 {
            is_solid = true;
        }

        if coords.x % 4 == 0 && coords.y % 4 == 2 {
            is_solid = true;
        }

        if is_solid { data(coords) } else { 0 }
    })
}
