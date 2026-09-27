use crate::meshing::triangulation::{WeavedTriangle, runs::Runs, zipper::Zipper};

pub const GRID_SIDE: usize = 32;
pub const INACTIVE_VERTEX: u8 = 0xFF;

#[derive(Clone, Debug, Default)]
pub struct Stepper {
    pub bottom_u_mask: u32,
    pub filled: [u32; 32],
    pub runs: Runs,
    pub zipper: Zipper,
}

impl Stepper {
    #[inline(always)]
    pub const fn from_bottom_edge(bottom_u_mask: u32, seed_u: u32, seed_v: u32) -> Stepper {
        debug_assert!(bottom_u_mask != 0, "Bottom edge must not be full of 0s");

        let runs = Runs::from_u_mask(bottom_u_mask, seed_u);
        let start = runs.all()[0].start() as usize;
        let end = runs.all()[0].end() as usize;

        let mut vertices = [INACTIVE_VERTEX; 32];
        vertices[start] = seed_v as u8;
        vertices[end] = seed_v as u8;

        let mut filled = [0; 32];
        filled[seed_v as usize] = runs.fill_mask();

        Stepper {
            bottom_u_mask,
            filled,
            runs: runs,
            zipper: Zipper {
                bottom_vs: vertices,
                bottom_u_mask: (1 << start) | (1 << end),
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
        mut on_triangle: impl FnMut(WeavedTriangle),
    ) {
        let caps = self.runs.guard_caps(top_guard);
        let vertices = (self.bottom_u_mask ^ top_edge) | caps;

        if vertices & self.runs.scan_mask() != 0 {
            debug_assert!(self.runs.count() > 0, "No bottom runs found");

            self.runs.iterate_runs(|run| {
                let scan_mask = run.scan_mask();
                self.zipper
                    .zip(scan_mask, top_row, vertices, &mut on_triangle);
            });

            self.runs.advance(top_edge, top_guard);
            let scan_mask = self.runs.scan_mask();

            self.zipper.advance(top_row, vertices, scan_mask);

            self.bottom_u_mask = top_edge;
        }

        self.filled[top_row] = self.runs.fill_mask();
    }
}
