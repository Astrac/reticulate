use crate::utils::bitmasks;

#[derive(Clone, Copy, Debug, Default)]
pub struct Run {
    pub start: u8,
    pub end: u8,
}

impl Run {
    #[inline(always)]
    pub const fn from_edge(edge: u32, seed_u: u32) -> Run {
        let mask_lower = edge & (!0u32 >> (31 - seed_u));
        let start = (31 - (mask_lower | 1).leading_zeros()) as u8;

        let mask_upper = edge & ((!0u32 << seed_u) << 1);
        let end = (mask_upper | (1 << 31)).trailing_zeros() as u8;

        Run { start, end }
    }

    #[inline(always)]
    pub const fn as_fill_mask(&self) -> u32 {
        bitmasks::make_ones_mask(self.start, self.end - 1)
    }

    #[inline(always)]
    pub const fn as_scan_mask(&self) -> u32 {
        bitmasks::make_ones_mask(self.start, self.end)
    }
}
