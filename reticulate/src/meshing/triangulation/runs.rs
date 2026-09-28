use crate::{
    meshing::triangulation::{run::Run, run_mask::RunMasks},
    utils::bitmasks,
};

#[derive(Clone, Debug, Default)]
pub struct Runs {
    runs: [Run; 16],
    runs_count: usize,
    masks: RunMasks,
}

impl Runs {
    #[inline(always)]
    pub const fn from_blocks(edge: u32, mut blocks: u32) -> Runs {
        let mut runs = [Run::default(); 16];
        let mut count = 0;
        let mut fill = 0;
        let mut scan = 0;

        while let Some(start) = blocks.lowest_one() {
            let run = Run::from_columns_mask(edge, start);
            debug_assert!((edge >> run.start()) & 1 == 1, "No wall found at run start");
            debug_assert!(
                count == 0 || run.start() >= runs[count - 1].end(),
                "Found out of order or overlapping run"
            );

            runs[count] = run;
            count += 1;
            fill |= run.fill_mask();
            scan |= run.scan_mask();

            // Blocks that merge into this run must not create it again.
            blocks &= !run.fill_mask();
        }

        Runs {
            runs,
            runs_count: count,
            masks: RunMasks::new(fill, scan),
        }
    }

    #[inline(always)]
    pub const fn from_columns_mask(columns_mask: u32, seed_columns: u32) -> Runs {
        Self::from_blocks(columns_mask, 1 << seed_columns)
    }

    #[inline(always)]
    pub const fn advance(&mut self, edge: u32, guard: u32) {
        *self = Self::from_blocks(edge, self.fill_mask() ^ guard);
    }

    #[inline(always)]
    pub const fn all(&self) -> &[Run; 16] {
        &self.runs
    }

    #[inline(always)]
    pub const fn at_index(&self, index: usize) -> &Run {
        &self.runs[index]
    }

    #[inline(always)]
    pub const fn count(&self) -> usize {
        self.runs_count
    }

    #[inline(always)]
    pub const fn fill_mask(&self) -> u32 {
        self.masks.fill()
    }

    #[inline(always)]
    pub const fn scan_mask(&self) -> u32 {
        self.masks.scan()
    }

    #[inline(always)]
    pub fn iterate_runs(&self, mut f: impl FnMut(Run)) {
        for run_index in 0..self.runs_count {
            f(self.runs[run_index]);
        }
    }

    #[inline(always)]
    pub const fn guard_caps(&self, guard: u32) -> u32 {
        bitmasks::segment_boundaries(self.fill_mask() & guard)
    }
}
