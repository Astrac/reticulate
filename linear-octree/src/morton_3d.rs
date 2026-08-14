use crate::{axis, bitmask_utils::*, coords::Coords3D, morton_2d::Morton2D};

// It might be a good idea to left-align it on the u32 inestead of using lsb
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Morton3D(u32);

impl Morton3D {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn mask(&mut self, mask: u32) {
        self.0 = self.0 & mask;
    }

    pub fn drop_axis(&self, axis: axis::Axis) -> u32 {
        let morton_3d = self.0;
        let (coord_1, coord_2) = match axis {
            axis::AXIS_X => (
                drop_one_every_3_bits(morton_3d >> 1),
                drop_one_every_3_bits(morton_3d >> 2),
            ),
            axis::AXIS_Y => (
                drop_one_every_3_bits(morton_3d),
                drop_one_every_3_bits(morton_3d >> 2),
            ),
            axis::AXIS_Z => (
                drop_one_every_3_bits(morton_3d),
                drop_one_every_3_bits(morton_3d >> 1),
            ),
            _ => unreachable!(),
        };

        interleave_zeroes(coord_1) | (interleave_zeroes(coord_2) << 1)
    }

    pub fn drop_axis_2(&self, axis: axis::Axis) -> Morton2D {
        Morton2D::from(self.drop_axis(axis))
    }
}

impl From<Morton3D> for Coords3D {
    fn from(value: Morton3D) -> Self {
        Coords3D {
            x: take_one_every_3_bits(value.0),
            y: take_one_every_3_bits(value.0 >> 1),
            z: take_one_every_3_bits(value.0 >> 2),
        }
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

// TODO: Add this for supported architectures
// use std::arch::x86_64::_pext_u32;
//
// pub fn drop_axis(&self, axis: Axis) -> u32 {
//     unsafe {
//         match axis {
//             // Mask out all Z bits (every 3rd bit starting at bit 2)
//             AXIS_Z => _pext_u32(self.0, 0xDB6D_B6DB),
//             // Mask out all Y bits (every 3rd bit starting at bit 1)
//             AXIS_Y => _pext_u32(self.0, 0xB6DB_6DB6),
//             // Mask out all X bits (every 3rd bit starting at bit 0)
//             AXIS_X => _pext_u32(self.0, 0x6DB6_DB6D),
//             _ => unreachable!(),
//         }
//     }
// }
