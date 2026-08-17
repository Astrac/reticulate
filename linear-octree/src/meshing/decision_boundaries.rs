use bitvec::prelude::*;

use crate::{Morton3D, face::Face, meshing::bitfield::face_size};

const BOUNDARY_FACE_SIZE: usize = 64 * 64;
const BOUNDARY_SIZE: usize = BOUNDARY_FACE_SIZE * 6;
const BITFIELD_SIZE: usize = BOUNDARY_SIZE * 2;

#[derive(Clone, Copy, Debug, Default)]
pub struct DecisionBoundaries {
    bitfield: BitArr!(for BITFIELD_SIZE, in u64, Lsb0),
}

impl DecisionBoundaries {
    #[inline(always)]
    pub fn committed(&self, face: Face) -> &BitSlice<u64, Lsb0> {
        let idx = face.as_index() * BOUNDARY_FACE_SIZE;
        &self.bitfield[idx..idx + BOUNDARY_FACE_SIZE]
    }

    #[inline(always)]
    pub fn committed_mut(&mut self, face: Face) -> &mut BitSlice<u64, Lsb0> {
        let idx = face.as_index() * BOUNDARY_FACE_SIZE;
        &mut self.bitfield[idx..idx + BOUNDARY_FACE_SIZE]
    }

    #[inline(always)]
    pub fn pending(&self, face: Face) -> &BitSlice<u64, Lsb0> {
        let idx = BOUNDARY_SIZE + face.as_index() * BOUNDARY_FACE_SIZE;
        &self.bitfield[idx..idx + BOUNDARY_FACE_SIZE]
    }

    #[inline(always)]
    pub fn pending_mut(&mut self, face: Face) -> &mut BitSlice<u64, Lsb0> {
        let idx = BOUNDARY_SIZE + face.as_index() * BOUNDARY_FACE_SIZE;
        &mut self.bitfield[idx..idx + BOUNDARY_FACE_SIZE]
    }

    #[inline(always)]
    pub fn committed_face_mut(
        &mut self,
        face: Face,
        morton: Morton3D,
        depth: u8,
    ) -> &mut BitSlice<u64, Lsb0> {
        let face_size = face_size(depth);
        let morton_2d = morton.drop_axis_2(face.axis()).as_usize() * face_size;
        &mut self.committed_mut(face)[morton_2d..morton_2d + face_size]
    }

    #[inline(always)]
    pub fn flip_committed_face(
        &mut self,
        face: Face,
        morton: Morton3D,
        depth: u8,
    ) -> &BitSlice<u64, Lsb0> {
        !self.committed_face_mut(face, morton, depth)
    }

    #[inline(always)]
    pub fn fill_committed_face(&mut self, face: Face, morton: Morton3D, depth: u8) {
        self.committed_face_mut(face, morton, depth).fill(true);
    }
}
