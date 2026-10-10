use crate::{
    Axis, ChunkCoord, Coords3D, Face, MaterialId,
    meshing::{surface_layer::SurfaceLayer, triangulation::Triangle2D},
};

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct PackedTriangle(pub u64);

impl PackedTriangle {
    pub const PLANE_MASK: u64 = (1 << Self::PLANE_BITS) - 1;
    pub const UV_MASK: u64 = (1 << Self::UV_BITS) - 1;
    pub const FACE_BITS: u32 = 1;
    pub const AXIS_BITS: u32 = 2;
    pub const PLANE_BITS: u32 = 5;
    pub const UV_BITS: u32 = 6;
    pub const AXIS_MASK: u64 = (1 << Self::AXIS_BITS) - 1;

    pub const FACE_SHIFT: u32 = 0;
    pub const AXIS_SHIFT: u32 = Self::FACE_SHIFT + Self::FACE_BITS;
    pub const MAT_SHIFT: u32 = Self::AXIS_SHIFT + Self::AXIS_BITS;
    pub const PLANE_SHIFT: u32 = Self::MAT_SHIFT + MaterialId::BITS;
    pub const V2V: u32 = Self::PLANE_SHIFT + Self::PLANE_BITS;
    pub const V2U: u32 = Self::V2V + Self::UV_BITS;
    pub const V1V: u32 = Self::V2U + Self::UV_BITS;
    pub const V1U: u32 = Self::V1V + Self::UV_BITS;
    pub const V0V: u32 = Self::V1U + Self::UV_BITS;
    pub const V0U: u32 = Self::V0V + Self::UV_BITS;

    const _ASSERT: () = assert!(Self::V0U + Self::UV_BITS <= 64);

    #[inline(always)]
    pub const fn as_u64(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn from_strip_triangle(
        face: Face,
        axis_coord: ChunkCoord,
        strip_triangle: Triangle2D,
        material: MaterialId,
    ) -> PackedTriangle {
        Self::pack_from(face, axis_coord, strip_triangle.vertices, material)
    }

    #[inline(always)]
    pub const fn axis(&self) -> Axis {
        Axis::from_usize(((self.0 >> Self::AXIS_SHIFT) & Self::AXIS_MASK) as usize)
    }

    #[inline(always)]
    pub const fn is_normal_positive(&self) -> bool {
        (self.0 & 1) != 0
    }

    #[inline(always)]
    pub const fn face(&self) -> Face {
        if self.is_normal_positive() {
            Face::Positive(self.axis())
        } else {
            Face::Negative(self.axis())
        }
    }

    #[inline(always)]
    pub const fn pack_from(
        face: Face,
        plane: u8,
        uv: [(u8, u8); 3],
        material: MaterialId,
    ) -> PackedTriangle {
        debug_assert!(plane as u64 <= Self::PLANE_MASK);
        debug_assert!(
            uv[0].0 as u64 <= Self::UV_MASK
                && uv[0].1 as u64 <= Self::UV_MASK
                && uv[1].0 as u64 <= Self::UV_MASK
                && uv[1].1 as u64 <= Self::UV_MASK
                && uv[2].0 as u64 <= Self::UV_MASK
                && uv[2].1 as u64 <= Self::UV_MASK
        );

        PackedTriangle(
            ((uv[0].0 as u64) << Self::V0U)
                | ((uv[0].1 as u64) << Self::V0V)
                | ((uv[1].0 as u64) << Self::V1U)
                | ((uv[1].1 as u64) << Self::V1V)
                | ((uv[2].0 as u64) << Self::V2U)
                | ((uv[2].1 as u64) << Self::V2V)
                | ((plane as u64) << Self::PLANE_SHIFT)
                | ((material as u64) << Self::MAT_SHIFT)
                | ((face.axis().as_usize() as u64) << Self::AXIS_SHIFT)
                | ((face.is_positive() as u64) << Self::FACE_SHIFT),
        )
    }

    #[inline(always)]
    pub const fn material(&self) -> MaterialId {
        (self.0 >> Self::MAT_SHIFT) as u16
    }

    #[inline(always)]
    pub const fn plane(&self) -> u8 {
        ((self.0 >> Self::PLANE_SHIFT) & Self::PLANE_MASK) as u8
    }

    #[inline(always)]
    pub const fn vertices(&self) -> [Coords3D; 3] {
        let p = self.0;
        let layer = SurfaceLayer::new(self.axis(), self.plane());
        [
            Coords3D::from_2d_projection(layer, Self::uv(p, Self::V0U, Self::V0V)),
            Coords3D::from_2d_projection(layer, Self::uv(p, Self::V1U, Self::V1V)),
            Coords3D::from_2d_projection(layer, Self::uv(p, Self::V2U, Self::V2V)),
        ]
    }

    #[inline(always)]
    pub const fn outward_vertices(&self) -> [Coords3D; 3] {
        let [a, b, c] = self.vertices();

        if self.is_normal_positive() {
            [a, c, b]
        } else {
            [a, b, c]
        }
    }

    #[inline(always)]
    const fn uv(p: u64, u_shift: u32, v_shift: u32) -> (u8, u8) {
        (
            ((p >> u_shift) & Self::UV_MASK) as u8,
            ((p >> v_shift) & Self::UV_MASK) as u8,
        )
    }
}
