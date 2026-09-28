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
    pub const fn from_u_mask(u_mask: u32, seed_u: u32) -> Runs {
        let mut runs = [Run::default(); 16];
        runs[0] = Run::from_u_mask(u_mask, seed_u);

        Runs {
            runs,
            runs_count: 1,
            masks: RunMasks::from_runs(&runs, 1),
        }
    }

    #[inline(always)]
    pub const fn advance(&mut self, edge: u32, guard: u32) {
        let mut blocks = self.fill_mask() ^ guard;
        let mut found_runs = 0;

        while let Some(start) = blocks.lowest_one() {
            let new_run = Run::from_u_mask(edge, start);
            debug_assert!(
                (edge >> new_run.start()) & 1 == 1,
                "No wall found at run start"
            );
            debug_assert!(
                found_runs == 0 || new_run.start() >= self.runs[found_runs - 1].end(),
                "Found out of order or overlapping run"
            );

            self.runs[found_runs] = new_run;
            found_runs += 1;

            // Blocks that merge into this run must not create it again.
            blocks &= !new_run.fill_mask();
        }

        self.masks = RunMasks::from_runs(&self.runs, found_runs);
        self.runs_count = found_runs;
    }

    #[inline(always)]
    pub const fn all(&self) -> &[Run; 16] {
        &self.runs
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
        let mut caps = 0;
        let mut run_index = 0;
        while run_index < self.runs_count {
            let blocked = self.runs[run_index].fill_mask() & guard;
            caps |= bitmasks::segment_boundaries(blocked);
            run_index += 1;
        }
        caps
    }
}
