#[inline(always)]
pub const fn drop_one_every_3_bits(mut x: u32) -> u32 {
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
pub const fn take_one_every_3_bits(mut n: u32) -> u8 {
    n = n & 0x249249;
    n = (n | (n >> 2)) & 0x0C30C3;
    n = (n | (n >> 4)) & 0x00F00F;
    n = (n | (n >> 8)) & 0x0000FF;
    n as u8
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
