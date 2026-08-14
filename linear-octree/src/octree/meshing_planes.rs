use std::ops::Range;

use bitvec::prelude::*;

use crate::{
    coords::Coords3D,
    morton_3d::Morton3D,
    octree::Walker,
    {axis::*, face::*},
};

const MESHING_PLANE_SIZE: usize = 64 * 64;

const MESHING_PLANE_BITFIELD_SIZE: usize = MESHING_PLANE_SIZE * 12;

const OUT_POS_X: usize = 0;
const OUT_POS_Y: usize = MESHING_PLANE_SIZE + 1;
const OUT_POS_Z: usize = OUT_POS_Y + MESHING_PLANE_SIZE;
const OUT_NEG_X: usize = OUT_POS_Z + MESHING_PLANE_SIZE;
const OUT_NEG_Y: usize = OUT_NEG_X + MESHING_PLANE_SIZE;
const OUT_NEG_Z: usize = OUT_NEG_Y + MESHING_PLANE_SIZE;
const INN_POS_X: usize = OUT_NEG_Z + MESHING_PLANE_SIZE;
const INN_POS_Y: usize = INN_POS_X + MESHING_PLANE_SIZE;
const INN_POS_Z: usize = INN_POS_Y + MESHING_PLANE_SIZE;
const INN_NEG_X: usize = INN_POS_Z + MESHING_PLANE_SIZE;
const INN_NEG_Y: usize = INN_NEG_X + MESHING_PLANE_SIZE;
const INN_NEG_Z: usize = INN_NEG_Y + MESHING_PLANE_SIZE;

#[derive(Default, Clone)]
pub struct MeshingPlanes {
    storage: BitArr!(for MESHING_PLANE_BITFIELD_SIZE, in u64, Lsb0),
}

impl MeshingPlanes {
    #[inline(always)]
    pub fn out_pos_x_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_POS_X + morton.drop_axis(AXIS_X) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_pos_x(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_POS_X + morton.drop_axis(AXIS_X) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_pos_y_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_POS_Y + morton.drop_axis(AXIS_Y) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_pos_y(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_POS_Y + morton.drop_axis(AXIS_Y) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_pos_z_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_POS_Z + morton.drop_axis(AXIS_Z) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_pos_z(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_POS_Z + morton.drop_axis(AXIS_Z) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_neg_x_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_NEG_X + morton.drop_axis(AXIS_X) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_neg_x(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_NEG_X + morton.drop_axis(AXIS_X) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_neg_y_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_NEG_Y + morton.drop_axis(AXIS_Y) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_neg_y(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_NEG_Y + morton.drop_axis(AXIS_Y) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_neg_z_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_NEG_Z + morton.drop_axis(AXIS_Z) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn out_neg_z(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = OUT_NEG_Z + morton.drop_axis(AXIS_Z) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_pos_x_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_POS_X + morton.drop_axis(AXIS_X) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_pos_x(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_POS_X + morton.drop_axis(AXIS_X) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_pos_y_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_POS_Y + morton.drop_axis(AXIS_Y) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_pos_y(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_POS_Y + morton.drop_axis(AXIS_Y) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_pos_z_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_POS_Z + morton.drop_axis(AXIS_Z) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_pos_z(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_POS_Z + morton.drop_axis(AXIS_Z) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_neg_x_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_NEG_X + morton.drop_axis(AXIS_X) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_neg_x(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_NEG_X + morton.drop_axis(AXIS_X) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_neg_y_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_NEG_Y + morton.drop_axis(AXIS_Y) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_neg_y(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_NEG_Y + morton.drop_axis(AXIS_Y) as usize;
        &self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_neg_z_mut(&mut self, morton: Morton3D, depth: u8) -> &mut BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_NEG_Z + morton.drop_axis(AXIS_Z) as usize;
        &mut self.storage[base_index..base_index + length]
    }

    #[inline(always)]
    pub fn inn_neg_z(&self, morton: Morton3D, depth: u8) -> &BitSlice<u64, Lsb0> {
        let length = 4096 >> (depth * 2);
        let base_index = INN_NEG_Z + morton.drop_axis(AXIS_Z) as usize;
        &self.storage[base_index..base_index + length]
    }
}
