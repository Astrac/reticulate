use crate::utils::bitmasks;

#[derive(Clone, Copy, Debug, Default)]
pub struct Run {
    start: u8,
    end: u8,
    fill_mask: u32,
    scan_mask: u32,
}

impl Run {
    #[inline(always)]
    pub const fn default() -> Run {
        Run {
            start: 0,
            end: 0,
            fill_mask: 0,
            scan_mask: 0,
        }
    }

    #[inline(always)]
    pub const fn from_u_mask(u_mask: u32, seed_u: u32) -> Run {
        let mask_lower = u_mask & (!0u32 >> (31 - seed_u));
        let start = (31 - (mask_lower | 1).leading_zeros()) as u8;

        let mask_upper = u_mask & ((!0u32 << seed_u) << 1);
        let end = (mask_upper | (1 << 31)).trailing_zeros() as u8;

        let fill_mask = bitmasks::make_ones_mask(start, end - 1);
        let scan_mask = bitmasks::make_ones_mask(start, end);

        Run {
            end,
            fill_mask,
            scan_mask,
            start,
        }
    }

    #[inline(always)]
    pub const fn start(&self) -> u8 {
        self.start
    }

    #[inline(always)]
    pub const fn end(&self) -> u8 {
        self.end
    }

    #[inline(always)]
    pub const fn fill_mask(&self) -> u32 {
        self.fill_mask
    }

    #[inline(always)]
    pub const fn scan_mask(&self) -> u32 {
        self.scan_mask
    }
}
