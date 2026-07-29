use crate::Coords;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Morton(u32);

impl Morton {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
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
