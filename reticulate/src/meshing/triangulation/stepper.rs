use crate::meshing::triangulation::{WeavedTriangle, run::Run, zipper::Zipper};

pub const GRID_SIDE: usize = 32;
pub const INACTIVE_VERTEX: u8 = 0xFF;

// TODO - Rename to scanner
#[derive(Clone, Debug, Default)]
pub struct Stepper {
    pub bottom_u_mask: u32,
    pub filled: [u32; 32],
    pub runs: [Run; 16],
    pub runs_count: usize,
    pub zipper: Zipper,
}

impl Stepper {
    #[inline(always)]
    pub fn from_bottom_edge(bottom_u_mask: u32, seed_u: u32, seed_v: u32) -> Stepper {
        debug_assert!(bottom_u_mask != 0, "Bottom edge must not be full of 0s");

        let mut runs = [Run::default(); 16];
        runs[0] = Run::from_edge(bottom_u_mask, seed_u);

        let mut vertices = [INACTIVE_VERTEX; 32];
        vertices[runs[0].start as usize] = seed_v as u8;
        vertices[runs[0].end as usize] = seed_v as u8;

        let mut filled = [0; 32];
        filled[seed_v as usize] = runs[0].as_fill_mask();

        Stepper {
            bottom_u_mask,
            filled,
            runs,
            runs_count: 1,
            zipper: Zipper {
                bottom_vs: vertices,
                bottom_u_mask: (1 << runs[0].start) | (1 << runs[0].end),
            },
        }
    }

    // TODO - Benchmark and optimize
    #[inline(always)]
    pub fn step_to(
        &mut self,
        top_row: usize,
        top_edge: u32,
        top_guard: u32,
        on_triangle: impl FnMut(WeavedTriangle),
    ) {
        let caps = self.guard_caps(top_guard);
        let vertices = (self.bottom_u_mask ^ top_edge) | caps;

        if vertices & self.scan_mask() != 0 {
            self.make_triangles(top_row, vertices, on_triangle);
            self.set_new_runs(top_edge, top_guard);
            self.zipper
                .set_new_vertices(top_row, vertices, self.scan_mask());
            self.bottom_u_mask = top_edge;
        }

        self.filled[top_row] = self.fill_mask();
    }

    #[inline(always)]
    pub const fn fill_mask(&self) -> u32 {
        let mut mask = 0;
        let mut run_index = 0;
        while run_index < self.runs_count {
            mask |= self.runs[run_index].as_fill_mask();
            run_index += 1;
        }

        mask
    }

    #[inline(always)]
    pub const fn scan_mask(&self) -> u32 {
        let mut mask = 0;
        let mut run_index = 0;
        while run_index < self.runs_count {
            mask |= self.runs[run_index].as_scan_mask();
            run_index += 1;
        }

        mask
    }

    /// Top vertices introduced by the guard: the boundaries of each blocked
    /// segment, computed per run so that two adjacent closed runs keep the
    /// wall they share. A blocked segment covering columns a..b-1 yields
    /// bits a and b, the same "bit k = boundary between k-1 and k"
    /// convention as the edges.
    #[inline(always)]
    fn guard_caps(&self, guard: u32) -> u32 {
        let mut caps = 0;
        for i in 0..self.runs_count {
            let blocked = self.runs[i].as_fill_mask() & guard;
            caps |= blocked ^ (blocked << 1);
        }
        caps
    }

    fn make_triangles(
        &mut self,
        top_row: usize,
        top_vertices: u32,
        mut on_triangle: impl FnMut(WeavedTriangle),
    ) {
        debug_assert!(self.runs_count > 0, "No bottom runs found");

        for run_index in 0..self.runs_count {
            let run_mask = self.runs[run_index].as_scan_mask();

            self.zipper
                .zip_strip(run_mask, top_row, top_vertices, &mut on_triangle);
        }
    }

    // TODO - fill_mask() here might not need to be recomputed if we
    // carry the index of the last row
    #[inline(always)]
    fn set_new_runs(&mut self, edge: u32, guard: u32) {
        let mut blocks = self.fill_mask() ^ guard;
        let mut found_runs = 0;

        while let Some(start) = blocks.lowest_one() {
            let new_run = Run::from_edge(edge, start);
            debug_assert!(
                (edge >> new_run.start) & 1 == 1,
                "No wall found at run start"
            );
            debug_assert!(
                found_runs == 0 || new_run.start >= self.runs[found_runs - 1].end,
                "Found out of order or overlapping run"
            );

            self.runs[found_runs] = new_run;
            found_runs += 1;

            // Blocks that merge into this run must not create it again.
            blocks &= !new_run.as_fill_mask();
        }

        self.runs_count = found_runs;
    }
}
