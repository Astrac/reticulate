mod gathering_walker;
mod shaper;
mod shaping_walker;
mod triangulator;

pub mod bitfields;
pub mod triangulation;

pub use triangulator::Triangulator;

use crate::{
    Coords3D,
    axis::*,
    face::Face,
    meshing::{bitfields::MaterialId, triangulation::Triangle2D},
};

#[derive(Clone, Debug, Default)]
pub struct Triangle {
    pub face: Face,
    pub material: MaterialId,
    pub vertices: [Coords3D; 3],
}

impl Triangle {
    pub fn from_strip_triangle(
        face: Face,
        axis_coord: u8,
        strip_triangle: Triangle2D,
        material: MaterialId,
    ) -> Triangle {
        let axis = face.axis();

        Triangle {
            face,
            material,
            vertices: [
                to_coords_3d(axis, axis_coord, strip_triangle.vertices[0]),
                to_coords_3d(axis, axis_coord, strip_triangle.vertices[1]),
                to_coords_3d(axis, axis_coord, strip_triangle.vertices[2]),
            ],
        }
    }
}

fn to_coords_3d(axis: Axis, axis_coord: u8, uv_coords: (u8, u8)) -> Coords3D {
    match axis {
        AXIS_X => Coords3D {
            x: axis_coord,
            y: uv_coords.1 as u8,
            z: uv_coords.0 as u8,
        },
        AXIS_Y => Coords3D {
            x: uv_coords.0 as u8,
            y: axis_coord,
            z: uv_coords.1 as u8,
        },
        AXIS_Z => Coords3D {
            x: uv_coords.1 as u8,
            y: uv_coords.0 as u8,
            z: axis_coord,
        },
        _ => unreachable!(),
    }
}
