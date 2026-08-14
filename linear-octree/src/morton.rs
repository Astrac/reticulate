use crate::Coords;
use crate::octree::axis::*;

// It might be a good idea to left-align it on the u32 inestead of using lsb
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Morton(u32);

impl Morton {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn mask(&mut self, mask: u32) {
        self.0 = self.0 & mask;
    }

    pub fn project_on_plane(&self, axis: Axis) -> u32 {
        match axis.as_u8() {
            0b00100 => {
                // AXIS_Z: Keep X (bit 0) and Y (bit 1), discard Z (bit 2)
                let x = compact_3d_channel(self.0 >> 0);
                let y = compact_3d_channel(self.0 >> 1);
                interleave_2d_planes(x, y)
            }
            0b00010 => {
                // AXIS_Y: Keep X (bit 0) and Z (bit 2), discard Y (bit 1)
                let x = compact_3d_channel(self.0 >> 0);
                let z = compact_3d_channel(self.0 >> 2);
                interleave_2d_planes(x, z)
            }
            _ => {
                // AXIS_X: Keep Y (bit 1) and Z (bit 2), discard X (bit 0)
                let y = compact_3d_channel(self.0 >> 1);
                let z = compact_3d_channel(self.0 >> 2);
                interleave_2d_planes(y, z)
            }
        }
    }
}
///
/// Correctly compacts a single 1D coordinate out of a 3D Morton layout.
/// This flawlessly isolates bits at positions 0, 3, 6, 9... up to 30 bits.
#[inline(always)]
fn compact_3d_channel(mut x: u32) -> u32 {
    x &= 0x0924_9249; // Isolate the specific 1D channel bits
    x = (x | (x >> 2)) & 0x030C_30C3; // Shift right by 2, group bits into pairs
    x = (x | (x >> 4)) & 0x0300_F00F; // Shift right by 4
    x = (x | (x >> 8)) & 0xFF00_00FF; // Shift right by 8
    x = (x | (x >> 16)) & 0x0000_03FF; // Squeeze down to the bottom 10 bits
    x
}

/// Interleaves two flat 10-bit coordinates into a single 2D Morton code (YXYX sequence).
#[inline(always)]
fn interleave_2d_planes(coord_a: u32, coord_b: u32) -> u32 {
    let mut a = coord_a & 0x0000_03FF;
    a = (a | (a << 8)) & 0x00FF_00FF;
    a = (a | (a << 4)) & 0x0F0F_0F0F;
    a = (a | (a << 2)) & 0x3333_3333;
    a = (a | (a << 1)) & 0x5555_5555; // Spreads out to alternating slots: -a-a-a-a

    let mut b = coord_b & 0x0000_03FF;
    b = (b | (b << 8)) & 0x00FF_00FF;
    b = (b | (b << 4)) & 0x0F0F_0F0F;
    b = (b | (b << 2)) & 0x3333_3333;
    b = (b | (b << 1)) & 0x5555_5555;

    a | (b << 1) // Merge them into a tight, packed 2D code (b_a_b_a_b_a)
}

impl From<Morton> for Coords {
    fn from(value: Morton) -> Self {
        Coords {
            x: compress(value.0),
            y: compress(value.0 >> 1),
            z: compress(value.0 >> 2),
        }
    }
}

impl From<Coords> for Morton {
    fn from(value: Coords) -> Self {
        Morton((spread(value.z) << 2) | (spread(value.y) << 1) | spread(value.x))
    }
}

impl From<u32> for Morton {
    fn from(value: u32) -> Self {
        Morton(value)
    }
}

// N             = 0000000000a00b00c00d00e00f00g00h
// MASK          = 00000000001001001001001001001001
// N | (N >> 2)  = 0000000000a0ab0bc0cd0de0ef0fg0gh
// MASK          = 00000000000011000011000011000011
// N             = 000000000000ab0000cd0000ef0000gh
// N | (N >> 4)  = 000000000000ab00abcd00cdef00efgh
// MASK          = 00000000000000001111000000001111
// N             = 0000000000000000abcd00000000efgh
// N | (N >> 8)  = 0000000000000000abcd0000abcdefgh
// MASK          = 00000000000000000000000011111111
// N             = 000000000000000000000000abcdefgh
fn compress(mut n: u32) -> u8 {
    n = n & 0x249249;
    n = (n | (n >> 2)) & 0x0C30C3;
    n = (n | (n >> 4)) & 0x00F00F;
    n = (n | (n >> 8)) & 0x0000FF;
    n as u8
}

/// Spreads an u8 over an u32 leaving two bits of space in between
// N             = 000000000000000000000000abcdefgh
// N | (N << 8)  = 0000000000000000abcdefghabcdefgh
// MASK          = 00000000000000001111000000001111 (0x00F00F)
// N             = 0000000000000000abcd00000000efgh
// N | (N << 4)  = 000000000000abcdabcd0000efghefgh
// MASK          = 00000000000011000011000011000011 (0x0C30C3)
// N             = 000000000000ab0000cd0000ef0000gh
// N | (N << 2)  = 0000000000abab00cdcd00efef00ghgh
// MASK          = 00000000001001001001001001001001 (0x249249)
// RESULT        = 0000000000a00b00c00d00e00f00g00h
fn spread(n: u8) -> u32 {
    let mut n: u32 = n as u32;
    n = (n | (n << 8)) & 0x00F00F;
    n = (n | (n << 4)) & 0x0C30C3;
    (n | (n << 2)) & 0x249249
}

// TODO: Add this for supported architectures
// use std::arch::x86_64::_pext_u32;
//
// pub fn project_on_plane(&self, axis: Axis) -> u32 {
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
