use crate::{
    BoundaryMask, CellMask,
    meshing::triangulation::{run::Run, run_mask::RunMasks},
};

#[derive(Clone, Debug, Default)]
pub struct Runs {
    runs: [Run; 16],
    runs_count: usize,
    masks: RunMasks,
}

impl Runs {
    #[inline(always)]
    pub const fn from_blocks(edge: BoundaryMask, mut blocks: CellMask) -> Runs {
        let mut runs = [Run::default(); 16];
        let mut count = 0;
        let mut fill = 0;
        let mut scan = 0;

        while let Some(start) = blocks.lowest_one() {
            let run = Run::from_edges(edge, start);
            debug_assert!((edge >> run.start()) & 1 == 1, "No wall found at run start");
            debug_assert!((edge >> run.end()) & 1 == 1, "No wall found at run end");

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
    pub const fn at_index(&self, index: usize) -> &Run {
        &self.runs[index]
    }

    #[inline(always)]
    pub const fn count(&self) -> usize {
        self.runs_count
    }

    #[inline(always)]
    pub const fn fill_mask(&self) -> CellMask {
        self.masks.fill()
    }

    #[inline(always)]
    pub const fn scan_mask(&self) -> BoundaryMask {
        self.masks.scan()
    }

    #[inline(always)]
    pub fn iterate_runs(&self, mut f: impl FnMut(Run)) {
        for run_index in 0..self.runs_count {
            f(self.runs[run_index]);
        }
    }
}
