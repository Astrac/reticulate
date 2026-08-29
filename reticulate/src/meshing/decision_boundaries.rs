use bitvec::prelude::*;

use crate::{Morton3D, axis::Axis};

const BOUNDARY_FACE_SIZE: usize = 64 * 64;
const BOUNDARY_SIZE: usize = BOUNDARY_FACE_SIZE * 3;
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
        depth: u8,
    ) -> &mut BitSlice<u64, Lsb0> {
        let face_size = 4096 >> (depth * 2) as usize;
        let morton_2d = morton.drop_axis_2(axis).as_usize() * face_size;
        &mut self.boundary_mut(axis)[morton_2d..morton_2d + face_size]
    }

    #[inline(always)]
    pub fn flip(&mut self, axis: Axis, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        !self.face_mut(axis, morton, depth)
    }

    #[inline(always)]
    pub fn fill(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        self.face_mut(axis, morton, depth).fill(true);
    }

    #[inline(always)]
    fn boundary_mut(&mut self, axis: Axis) -> &mut BitSlice<u64, Lsb0> {
        let idx = axis.as_index() * BOUNDARY_FACE_SIZE;
        &mut self.masks_bitfield[idx..idx + BOUNDARY_FACE_SIZE]
    }
}
