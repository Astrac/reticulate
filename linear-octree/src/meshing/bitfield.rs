use bitvec::prelude::*;

use crate::{
    Morton3D,
    axis::*,
    meshing::{BITFIELD_FACE_FULL_SIZE, BITFIELD_SIZE, routing::Routing},
};

#[derive(Default, Clone)]
pub struct Bitfield {
    pub storage: BitArr!(for BITFIELD_SIZE, in u64, Lsb0),
    pub routing: Routing,
}

impl Bitfield {
    #[inline(always)]
    pub fn face_slice(
        &self,
        axis: Axis,
        mut selector: impl FnMut(&Routing) -> usize,
        morton: Morton3D,
        depth: u8,
    ) -> &BitSlice<u64, Lsb0> {
        let base_index = selector(&self.routing);
        let face_size = face_size(depth);

        let global_2d_key = morton.drop_axis(axis) as usize;
        let face_index = base_index + (global_2d_key * face_size);

        &self.storage[face_index..face_index + face_size]
    }

    #[inline(always)]
    pub fn face_slice_mut(
        &mut self,
        axis: Axis,
        mut selector: impl FnMut(&Routing) -> usize,
        morton: Morton3D,
        depth: u8,
    ) -> &mut BitSlice<u64, Lsb0> {
        let base_index = selector(&self.routing);
        let face_size = face_size(depth);

        let global_2d_key = morton.drop_axis(axis) as usize;
        let face_index = base_index + (global_2d_key * face_size);

        &mut self.storage[face_index..face_index + face_size]
    }

}

// This is 1 for depth 6 and 4096 for depth 0
#[inline(always)]
pub const fn face_size(depth: u8) -> usize {
    BITFIELD_FACE_FULL_SIZE >> (2 * depth)
}
