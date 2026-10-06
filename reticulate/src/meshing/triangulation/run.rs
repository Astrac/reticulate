use crate::{
    BoundaryMask, CellMask, PLANE_SIDE, meshing::triangulation::run_mask::RunMasks, utils::bitmasks,
};

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
    pub const fn from_edges(edges: BoundaryMask, seed_cell_index: u32) -> Run {
        let below = edges & (!0u64 >> (63 - seed_cell_index));
        let above = edges & ((!0u64 << seed_cell_index) << 1);

        // These imply a parity violation (i.e. an implicit wall) at indices
        // 0 and 32 of the chunk, forcing vertices on the chunk boundaries.
        let start = (63 - (below | 1).leading_zeros()) as u8;
        let end = (above | (1u64 << PLANE_SIDE)).trailing_zeros() as u8;

        let fill = bitmasks::cell_ones(start, end - 1);
        let scan = bitmasks::boundary_ones(start, end);

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
    pub const fn boundary_mask(&self) -> BoundaryMask {
        (1u64 << self.start) | (1u64 << self.end)
    }

    #[inline(always)]
    pub const fn fill_mask(&self) -> CellMask {
        self.masks.fill()
    }

    #[inline(always)]
    pub const fn scan_mask(&self) -> BoundaryMask {
        self.masks.scan()
    }
}
