use std::ops::BitOrAssign;

use crate::axis::*;

pub const OUT_NEG_X: usize = 0;
pub const OUT_NEG_Y: usize = 1;
pub const OUT_NEG_Z: usize = 2;
pub const OUT_POS_X: usize = 3;
pub const OUT_POS_Y: usize = 4;
pub const OUT_POS_Z: usize = 5;
pub const INN_NEG_X: usize = 6;
pub const INN_NEG_Y: usize = 7;
pub const INN_NEG_Z: usize = 8;
pub const INN_POS_X: usize = 9;
pub const INN_POS_Y: usize = 10;
pub const INN_POS_Z: usize = 11;

// Each of the first 4 bits of a mask represents a
// quadrant in a 2x2 subdivision.
#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct WorkingMask(u8);

impl BitOrAssign<u8> for WorkingMask {
    fn bitor_assign(&mut self, rhs: u8) {
        self.0 |= rhs
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorkingBoundary {
    // One per potential branching layer 0 to 5 included
    masks: [[WorkingMask; 12]; 6],
}
impl WorkingBoundary {
    // Write one bit to each of the targeted working masks?
    pub fn set_leaf(&mut self, depth: u8, octant: u8) {
        if depth == 0 {
            todo!();
        }

        let depth_idx = (depth as usize) - 1;

        let quadrant_x = (octant & (AXIS_Z | AXIS_Y)) >> 1;
        let x_bit = 1 << quadrant_x;
        if octant & AXIS_X == 0 {
            self.masks[depth_idx][OUT_NEG_X] |= x_bit;
            self.masks[depth_idx][INN_NEG_X] |= x_bit;
        } else {
            self.masks[depth_idx][OUT_POS_X] |= x_bit;
            self.masks[depth_idx][INN_POS_X] |= x_bit;
        }

        // let quadrant_y = ((octant & AXIS_Z) >> 1) | (octant & AXIS_X);
        // let quadrant_z = octant & (AXIS_Y | AXIS_X);
    }

    pub fn reset_branch(&mut self, depth: u8) {
        self.masks[depth as usize] = [WorkingMask::default(); 12];
    }
}
