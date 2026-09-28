mod gathering_walker;
mod triangulator;

pub mod bitfields;
pub mod shaper;
pub mod shaping_walker;
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
    #[inline(always)]
    pub const fn from_strip_triangle(
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

    #[inline(always)]
    pub const fn outward_vertices(&self) -> [Coords3D; 3] {
        let [a, b, c] = self.vertices;

        if self.face.0 & 0b1000 == 0 {
            [a, c, b]
        } else {
            [a, b, c]
        }
    }
}

#[inline(always)]
const fn to_coords_3d(axis: Axis, axis_coord: u8, uv_coords: (u8, u8)) -> Coords3D {
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
