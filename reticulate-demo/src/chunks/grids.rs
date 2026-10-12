use reticulate::{Coords3D, MaterialId};

use crate::ChunkDemo;

pub fn checkerboard(
    material: impl Fn(Coords3D) -> MaterialId + 'static + Send + Sync,
) -> ChunkDemo {
    ChunkDemo::new("checkerboard", move |coords| {
        if (coords.x + coords.y + coords.z) % 2 == 0 {
            material(coords)
        } else {
            0
        }
    })
}

pub fn ortho_labyrinth(
    material: impl Fn(Coords3D) -> MaterialId + 'static + Send + Sync,
) -> ChunkDemo {
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

        if is_solid { material(coords) } else { 0 }
    })
}

pub fn gyroid(material: impl Fn(Coords3D) -> MaterialId + 'static + Send + Sync) -> ChunkDemo {
    let period_wavelength = 16.0;
    let k = (2.0 * std::f64::consts::PI) / period_wavelength;

    ChunkDemo::new("gyroid", move |coords| {
        let z_f = coords.z as f64 * k;
        let sin_z = z_f.sin();
        let cos_z = z_f.cos();
        let y_f = coords.y as f64 * k;
        let sin_y = y_f.sin();
        let cos_y = y_f.cos();
        let x_f = coords.x as f64 * k;
        let sin_x = x_f.sin();
        let cos_x = x_f.cos();

        let value = sin_x * cos_y + sin_y * cos_z + sin_z * cos_x;

        if value > 0.0 { material(coords) } else { 0 }
    })
}
