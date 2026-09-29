use crate::{
    Coords3D,
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
                Coords3D::from_2d_projection(axis, axis_coord, strip_triangle.vertices[0]),
                Coords3D::from_2d_projection(axis, axis_coord, strip_triangle.vertices[1]),
                Coords3D::from_2d_projection(axis, axis_coord, strip_triangle.vertices[2]),
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
