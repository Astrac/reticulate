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

#[inline(always)]
pub const fn segment_boundaries(cells: u32) -> u32 {
    cells ^ (cells << 1)
}

#[inline(always)]
pub const fn run_starts(cells: u32) -> u32 {
    cells & !(cells << 1)
}

#[inline(always)]
pub const fn run_ends(cells: u32) -> u32 {
    (cells << 1) & !cells
}

#[inline(always)]
pub const fn boundary_changes(below: u32, above: u32) -> u32 {
    (run_starts(below) ^ run_starts(above)) | (run_ends(below) ^ run_ends(above))
}

#[inline(always)]
pub const fn boundary_points(below: u32, above: u32) -> u32 {
    let crossings = below ^ above;
    segment_boundaries(below) | segment_boundaries(above) | crossings | (crossings << 1)
}

#[inline(always)]
pub const fn transpose32(mut a: [u32; 32]) -> [u32; 32] {
    let mut j = 16u32;
    let mut m: u32 = 0x0000_FFFF;
    while j != 0 {
        let mut k = 0;
        while k < 32 {
            let mut i = k;
            while i < (k + j as usize) {
                let t = (a[i] ^ (a[i + j as usize] >> j)) & m;
                a[i] ^= t;
                a[i + j as usize] ^= t << j;
                i += 1;
            }
            k += (j << 1) as usize;
        }
        j >>= 1;
        m ^= m << j;
    }
    a
}
