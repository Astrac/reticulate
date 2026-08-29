use bitvec::prelude::*;

use crate::{Depth, Morton3D, axis::Axis};

pub const BOUNDARY_PLANE_SIZE: usize = 32 * 32;
const BOUNDARY_SIZE: usize = BOUNDARY_PLANE_SIZE * 3;
const BITFIELD_SIZE: usize = BOUNDARY_SIZE;

#[derive(Clone, Copy, Debug, Default)]
pub struct DecisionBoundaries {
    masks_bitfield: BitArr!(for BITFIELD_SIZE, in u64, Lsb0),
}

impl DecisionBoundaries {
    #[inline(always)]
    pub fn face_mut(
        &mut self,
        axis: Axis,
        morton: Morton3D,
        depth: Depth,
    ) -> &mut BitSlice<u64, Lsb0> {
        let face_size = BOUNDARY_PLANE_SIZE >> (depth.as_usize() * 2);
        let morton_2d = morton.drop_axis_2(axis).as_usize() * face_size;
        &mut self.boundary_mut(axis)[morton_2d..morton_2d + face_size]
    }

    #[inline(always)]
    pub fn flip(&mut self, axis: Axis, morton: Morton3D, depth: Depth) -> &BitSlice<u64, Lsb0> {
        !self.face_mut(axis, morton, depth)
    }

    #[inline(always)]
    pub fn fill(&mut self, axis: Axis, morton: Morton3D, depth: Depth) {
        self.face_mut(axis, morton, depth).fill(true);
    }

    #[inline(always)]
    fn boundary_mut(&mut self, axis: Axis) -> &mut BitSlice<u64, Lsb0> {
        let idx = axis.as_index() * BOUNDARY_PLANE_SIZE;
        &mut self.masks_bitfield[idx..idx + BOUNDARY_PLANE_SIZE]
    }
}
