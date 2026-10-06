use crate::{BoundaryMask, CellMask};

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
pub const fn cell_ones(from: u8, to: u8) -> CellMask {
    let upper_bound = ((1u32 << to) << 1).wrapping_sub(1);
    let lower_bound = (1u32 << from).wrapping_sub(1);

    upper_bound & !lower_bound
}

#[inline(always)]
pub const fn boundary_ones(from: u8, to: u8) -> BoundaryMask {
    let upper_bound = ((1u64 << to) << 1).wrapping_sub(1);
    let lower_bound = (1u64 << from).wrapping_sub(1);

    upper_bound & !lower_bound
}

#[inline(always)]
pub const fn segment_boundaries(cells: CellMask) -> BoundaryMask {
    let cells = cells as BoundaryMask;
    cells ^ (cells << 1)
}

#[inline(always)]
pub const fn run_starts(cells: CellMask) -> BoundaryMask {
    let cells = cells as BoundaryMask;
    cells & !(cells << 1)
}

#[inline(always)]
pub const fn run_ends(cells: CellMask) -> BoundaryMask {
    let cells = cells as BoundaryMask;
    (cells << 1) & !cells
}

#[inline(always)]
pub const fn boundary_changes(below: CellMask, above: CellMask) -> BoundaryMask {
    (run_starts(below) ^ run_starts(above)) | (run_ends(below) ^ run_ends(above))
}

#[inline(always)]
pub const fn boundary_points(below: CellMask, above: CellMask) -> BoundaryMask {
    let crossings = (below ^ above) as BoundaryMask;
    segment_boundaries(below) | segment_boundaries(above) | crossings | (crossings << 1)
}
