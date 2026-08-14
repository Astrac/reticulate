use bitvec::prelude::*;

use crate::{axis::*, morton_3d::Morton3D};

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

#[cfg(test)]
mod tests {
    use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_default_all_false() {
        let planes = MeshingPlanes::default();
        let morton = Morton3D::from(0);

        assert!(!planes.out_pos_x(morton, 6)[0]);
        assert!(!planes.out_pos_y(morton, 6)[0]);
        assert!(!planes.out_pos_z(morton, 6)[0]);
        assert!(!planes.out_neg_x(morton, 6)[0]);
        assert!(!planes.out_neg_y(morton, 6)[0]);
        assert!(!planes.out_neg_z(morton, 6)[0]);
        assert!(!planes.inn_pos_x(morton, 6)[0]);
        assert!(!planes.inn_pos_y(morton, 6)[0]);
        assert!(!planes.inn_pos_z(morton, 6)[0]);
        assert!(!planes.inn_neg_x(morton, 6)[0]);
        assert!(!planes.inn_neg_y(morton, 6)[0]);
        assert!(!planes.inn_neg_z(morton, 6)[0]);
    }

    #[test]
    fn test_mut_and_read_consistency() {
        let mut planes = MeshingPlanes::default();
        let morton = Morton3D::from(0);

        planes.out_pos_x_mut(morton, 6).set(0, true);
        assert!(planes.out_pos_x(morton, 6)[0]);

        planes.out_pos_x_mut(morton, 6).set(0, false);
        assert!(!planes.out_pos_x(morton, 6)[0]);
    }

    #[test]
    fn test_plane_independence() {
        let mut planes = MeshingPlanes::default();
        let morton = Morton3D::from(0);

        planes.out_pos_x_mut(morton, 6).set(0, true);

        assert!(!planes.inn_pos_x(morton, 6)[0]);
        assert!(!planes.out_pos_y(morton, 6)[0]);

        planes.inn_neg_z_mut(morton, 5).set(0, true);

        assert!(planes.out_pos_x(morton, 6)[0]);
    }

    #[test]
    fn test_depth_slice_length() {
        let planes = MeshingPlanes::default();
        let morton = Morton3D::from(0);

        assert_eq!(planes.out_pos_x(morton, 0).len(), 4096);
        assert_eq!(planes.out_pos_x(morton, 1).len(), 1024);
        assert_eq!(planes.out_pos_x(morton, 2).len(), 256);
        assert_eq!(planes.out_pos_x(morton, 3).len(), 64);
        assert_eq!(planes.out_pos_x(morton, 4).len(), 16);
        assert_eq!(planes.out_pos_x(morton, 5).len(), 4);
        assert_eq!(planes.out_pos_x(morton, 6).len(), 1);
    }

    #[test]
    fn test_offset_isolation() {
        let mut planes = MeshingPlanes::default();
        let m0 = Morton3D::from(0);
        let m1 = Morton3D::from(2);

        planes.out_pos_x_mut(m0, 6).set(0, true);
        assert!(!planes.out_pos_x(m1, 6)[0]);

        planes.out_pos_x_mut(m1, 6).set(0, true);
        assert!(planes.out_pos_x(m0, 6)[0]);
    }
}
