#[inline(always)]
pub const fn take_one_every_3_bits(mut x: u32) -> u32 {
    x &= 0x0924_9249;
    x = (x | (x >> 2)) & 0x030C_30C3;
    x = (x | (x >> 4)) & 0x0300_F00F;
    x = (x | (x >> 8)) & 0x0300_00FF;
    x = (x | (x >> 16)) & 0x0000_03FF;
    x
}

#[inline(always)]
pub const fn interleave_zeroes(mut x: u32) -> u32 {
    x &= 0x0000_FFFF;
    x = (x | (x << 8)) & 0x00FF_00FF;
    x = (x | (x << 4)) & 0x0F0F_0F0F;
    x = (x | (x << 2)) & 0x3333_3333;
    x = (x | (x << 1)) & 0x5555_5555;
    x
}

#[inline(always)]
pub const fn take_one_every_2_bits(mut x: u32) -> u32 {
    x &= 0x5555_5555;
    x = (x | (x >> 1)) & 0x3333_3333;
    x = (x | (x >> 2)) & 0x0F0F_0F0F;
    x = (x | (x >> 4)) & 0x00FF_00FF;
    x = (x | (x >> 8)) & 0x0000_FFFF;
    x
}

#[inline(always)]
pub const fn interleave_two_zeroes(n: u8) -> u32 {
    let mut n: u32 = n as u32;
    n = (n | (n << 8)) & 0x00F00F;
    n = (n | (n << 4)) & 0x0C30C3;
    (n | (n << 2)) & 0x249249
}

#[inline(always)]
pub const fn make_ones_mask(from: u8, to: u8) -> u32 {
    let upper_bound = ((1u32 << to) << 1).wrapping_sub(1);
    let lower_bound = (1u32 << from).wrapping_sub(1);

    upper_bound & !lower_bound
}

// TODO - Move to test suite
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_one_every_3_bits() {
        assert_eq!(take_one_every_3_bits(0b100_100_100), 0b000);
        assert_eq!(take_one_every_3_bits(0b010_010_010), 0b000);
        assert_eq!(take_one_every_3_bits(0b001_001_001), 0b111);
        assert_eq!(take_one_every_3_bits(0b100_010_001), 0b001);
        assert_eq!(take_one_every_3_bits(0), 0);
    }

    #[test]
    fn test_interleave_zeroes() {
        assert_eq!(interleave_zeroes(0b111), 0b010101);
        assert_eq!(interleave_zeroes(0b101), 0b010001);
        assert_eq!(interleave_zeroes(0b010), 0b000100);
        assert_eq!(interleave_zeroes(0b001), 0b000001);
        assert_eq!(interleave_zeroes(0), 0);
    }

    #[test]
    fn test_take_one_every_2_bits() {
        assert_eq!(take_one_every_2_bits(0b101010), 0b000);
        assert_eq!(take_one_every_2_bits(0b010101), 0b111);
        assert_eq!(take_one_every_2_bits(0b100100), 0b010);
        assert_eq!(take_one_every_2_bits(0), 0);
    }

    #[test]
    fn test_interleave_two_zeroes() {
        assert_eq!(interleave_two_zeroes(0b111_u8), 0b001_001_001);
        assert_eq!(interleave_two_zeroes(0b101_u8), 0b001_000_001);
        assert_eq!(interleave_two_zeroes(0b010_u8), 0b000_001_000);
        assert_eq!(interleave_two_zeroes(0b001_u8), 0b000_000_001);
        assert_eq!(interleave_two_zeroes(0), 0);
    }
}
