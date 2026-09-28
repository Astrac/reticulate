use crate::{meshing::triangulation::run_mask::RunMasks, utils::bitmasks};

#[derive(Clone, Copy, Debug, Default)]
pub struct Run {
    start: u8,
    end: u8,
    masks: RunMasks,
}

impl Run {
    #[inline(always)]
    pub const fn default() -> Run {
        Run {
            start: 0,
            end: 0,
            masks: RunMasks::new(0, 0),
        }
    }

    #[inline(always)]
    pub const fn from_u_mask(u_mask: u32, seed_u: u32) -> Run {
        let mask_lower = u_mask & (!0u32 >> (31 - seed_u));
        let start = (31 - (mask_lower | 1).leading_zeros()) as u8;

        let mask_upper = u_mask & ((!0u32 << seed_u) << 1);
        let end = (mask_upper | (1 << 31)).trailing_zeros() as u8;

        let fill = bitmasks::make_ones_mask(start, end - 1);
        let scan = bitmasks::make_ones_mask(start, end);

        Run {
            end,
            start,
            masks: RunMasks::new(fill, scan),
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
        self.masks.fill()
    }

    #[inline(always)]
    pub const fn scan_mask(&self) -> u32 {
        self.masks.scan()
    }
}
