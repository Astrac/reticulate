use crate::{
    Axis, ChunkCoord, Coords2D, Coords3D, Depth, TREE_DEPTH, morton_2d::Morton2D, utils::bitmasks,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Morton3D {
    pub code: u32,
    pub depth: Depth,
}

impl Morton3D {
    #[inline(always)]
    pub const fn from_xyz_and_depth(
        x: ChunkCoord,
        y: ChunkCoord,
        z: ChunkCoord,
        depth: Depth,
    ) -> Morton3D {
        Morton3D {
            code: (bitmasks::interleave_two_zeroes(z) << 2)
                | (bitmasks::interleave_two_zeroes(y) << 1)
                | bitmasks::interleave_two_zeroes(x),
            depth,
        }
    }

    #[inline(always)]
    pub const fn from_xyz_max_depth(x: ChunkCoord, y: ChunkCoord, z: ChunkCoord) -> Morton3D {
        Morton3D {
            code: (bitmasks::interleave_two_zeroes(z) << 2)
                | (bitmasks::interleave_two_zeroes(y) << 1)
                | bitmasks::interleave_two_zeroes(x),
            depth: Depth::MAX,
        }
    }

    #[inline(always)]
    pub const fn from_coords_max_depth(coords: Coords3D) -> Morton3D {
        Self::from_xyz_max_depth(coords.x, coords.y, coords.z)
    }

    #[inline(always)]
    pub fn from_coords_and_depth(coords: Coords3D, depth: Depth) -> Morton3D {
        Self::from_xyz_and_depth(coords.x, coords.y, coords.z, depth)
    }

    #[inline(always)]
    pub const fn from_code(code: u32, depth: Depth) -> Morton3D {
        Morton3D { code, depth }
    }

    pub const fn from_code_max_depth(code: u32) -> Morton3D {
        Self::from_code(code, Depth::MAX)
    }

    // TODO - Rename this to make it clear that depth is ignored
    #[inline(always)]
    pub const fn as_u32(&self) -> u32 {
        self.code
    }

    #[inline(always)]
    pub const fn depth(&self) -> Depth {
        self.depth
    }

    #[inline(always)]
    pub const fn octant(&self) -> u8 {
        (self.code & 0b111) as u8
    }

    #[inline(always)]
    pub const fn parent(&self) -> Morton3D {
        Morton3D {
            code: self.code >> 3,
            depth: Depth(self.depth.0 - 1),
        }
    }

    #[inline(always)]
    pub const fn child_octant(&self, octant: ChunkCoord) -> Morton3D {
        Morton3D {
            code: (self.code << 3) | (octant as u32),
            depth: Depth(self.depth.0 + 1),
        }
    }

    #[inline(always)]
    pub const fn projected_axis_coords(&self, axis: Axis) -> (Coords2D, u8) {
        let ((u, v), layer) = self.projected_axis_uv(axis);
        (Coords2D { u, v }, layer)
    }

    #[inline(always)]
    pub const fn projected_axis_morton(&self, axis: Axis) -> (Morton2D, u8) {
        let ((u, v), layer) = self.projected_axis_uv(axis);
        (Morton2D::from_uv_and_depth(u, v, self.depth), layer)
    }

    #[inline(always)]
    pub const fn drop_axis(&self, axis: Axis) -> Morton2D {
        self.projected_axis_morton(axis).0
    }

    #[inline(always)]
    const fn projected_axis_uv(&self, axis: Axis) -> ((ChunkCoord, ChunkCoord), u8) {
        let morton_3d = self.code << ((TREE_DEPTH - self.depth.as_usize()) * 3);
        let (u, v, layer) = match axis {
            Axis::X => (
                bitmasks::take_one_every_3_bits(morton_3d >> 1),
                bitmasks::take_one_every_3_bits(morton_3d >> 2),
                bitmasks::take_one_every_3_bits(morton_3d),
            ),
            Axis::Y => (
                bitmasks::take_one_every_3_bits(morton_3d >> 2),
                bitmasks::take_one_every_3_bits(morton_3d),
                bitmasks::take_one_every_3_bits(morton_3d >> 1),
            ),
            Axis::Z => (
                bitmasks::take_one_every_3_bits(morton_3d),
                bitmasks::take_one_every_3_bits(morton_3d >> 1),
                bitmasks::take_one_every_3_bits(morton_3d >> 2),
            ),
        };

        ((u as ChunkCoord, v as ChunkCoord), layer as ChunkCoord)
    }
}
