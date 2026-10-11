use crate::{
    Axis, ChunkCoord, Coords3D, Face, MaterialId,
    meshing::{surface_layer::SurfaceLayer, triangulation::Triangle2D},
};

// TODO - Store as layer + x3 2d vertices to save memory
#[derive(Clone, Debug)]
pub struct Triangle {
    pub face: Face,
    pub material: MaterialId,
    pub vertices: [Coords3D; 3],
}

impl Triangle {
    #[inline(always)]
    pub const fn from_strip_triangle(
        face: Face,
        axis_coord: ChunkCoord,
        strip_triangle: Triangle2D,
        material: MaterialId,
    ) -> Triangle {
        let axis = face.axis();

        Triangle {
            face,
            material,
            vertices: [
                Coords3D::from_2d_projection(
                    SurfaceLayer::new(axis, axis_coord),
                    strip_triangle.vertices[0],
                ),
                Coords3D::from_2d_projection(
                    SurfaceLayer::new(axis, axis_coord),
                    strip_triangle.vertices[1],
                ),
                Coords3D::from_2d_projection(
                    SurfaceLayer::new(axis, axis_coord),
                    strip_triangle.vertices[2],
                ),
            ],
        }
    }

    #[inline(always)]
    pub const fn vertices(&self) -> [Coords3D; 3] {
        self.vertices
    }

    #[inline(always)]
    pub const fn face(&self) -> Face {
        self.face
    }

    #[inline(always)]
    pub const fn axis(&self) -> Axis {
        self.face.axis()
    }

    #[inline(always)]
    pub const fn material(&self) -> MaterialId {
        self.material
    }

    #[inline(always)]
    pub const fn is_normal_positive(&self) -> bool {
        self.face.is_positive()
    }

    #[inline(always)]
    pub const fn outward_vertices(&self) -> [Coords3D; 3] {
        let [a, b, c] = self.vertices;

        if self.face.is_positive() {
            [a, c, b]
        } else {
            [a, b, c]
        }
    }
}
