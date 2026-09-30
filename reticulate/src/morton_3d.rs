use crate::{
    Coords2D, Depth, axis::Axis, coords_3d::Coords3D, morton_2d::Morton2D, octree::TREE_DEPTH,
    utils::bitmasks::*,
};

// It might be a good idea to left-align it on the u32 inestead of using lsb
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Morton3D(u32);

// TODO - Cleanup repetitions!
impl Morton3D {
    #[inline(always)]
    pub const fn from_xyz(x: u8, y: u8, z: u8) -> Morton3D {
        Morton3D(
            (interleave_two_zeroes(z) << 2)
                | (interleave_two_zeroes(y) << 1)
                | interleave_two_zeroes(x),
        )
    }

    #[inline(always)]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn octant(&self) -> u8 {
        (self.0 & 0b111) as u8
    }

    #[inline(always)]
    pub const fn parent(&self) -> Morton3D {
        Morton3D(self.0 >> 3)
    }

    #[inline(always)]
    pub const fn child_octant(&self, octant: u8) -> Morton3D {
        Morton3D((self.0 << 3) | (octant as u32))
    }

    #[inline(always)]
    pub const fn projected_axis_coords(&self, axis: Axis, depth: Depth) -> (Coords2D, usize) {
        let ((u, v), layer) = self.projected_axis_uv(axis, depth);
        (Coords2D { u, v }, layer as usize)
    }

    #[inline(always)]
    pub const fn projected_axis_morton(&self, axis: Axis, depth: Depth) -> (Morton2D, usize) {
        let ((u, v), layer) = self.projected_axis_uv(axis, depth);
        (Morton2D::from_uv(u, v), layer as usize)
    }

    #[inline(always)]
    pub const fn drop_axis(&self, axis: Axis) -> Morton2D {
        let morton_3d = self.0;
        let (coord_1, coord_2) = match axis {
            Axis::X => (
                take_one_every_3_bits(morton_3d >> 1),
                take_one_every_3_bits(morton_3d >> 2),
            ),
            Axis::Y => (
                take_one_every_3_bits(morton_3d >> 2),
                take_one_every_3_bits(morton_3d),
            ),
            Axis::Z => (
                take_one_every_3_bits(morton_3d),
                take_one_every_3_bits(morton_3d >> 1),
            ),
        };

        Morton2D(interleave_zeroes(coord_1) | (interleave_zeroes(coord_2) << 1))
    }

    const fn projected_axis_uv(&self, axis: Axis, depth: Depth) -> ((u32, u32), usize) {
        let morton_3d = self.0 << ((TREE_DEPTH - depth.as_usize()) * 3);
        let (u, v, layer) = match axis {
            Axis::X => (
                take_one_every_3_bits(morton_3d >> 1),
                take_one_every_3_bits(morton_3d >> 2),
                take_one_every_3_bits(morton_3d),
            ),
            Axis::Y => (
                take_one_every_3_bits(morton_3d >> 2),
                take_one_every_3_bits(morton_3d),
                take_one_every_3_bits(morton_3d >> 1),
            ),
            Axis::Z => (
                take_one_every_3_bits(morton_3d),
                take_one_every_3_bits(morton_3d >> 1),
                take_one_every_3_bits(morton_3d >> 2),
            ),
        };

        ((u, v), layer as usize)
    }
}

impl From<Coords3D> for Morton3D {
    fn from(value: Coords3D) -> Self {
        Morton3D(
            (interleave_two_zeroes(value.z) << 2)
                | (interleave_two_zeroes(value.y) << 1)
                | interleave_two_zeroes(value.x),
        )
    }
}

impl From<u32> for Morton3D {
    fn from(value: u32) -> Self {
        Morton3D(value)
    }
}
