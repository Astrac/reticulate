use crate::{
    meshing::triangulation::{Triangle2D, runs::Runs, zipper::Zipper},
    utils::bitmasks,
};

pub const GRID_SIDE: usize = 32;
pub const INACTIVE_VERTEX: u8 = 0xFF;

#[derive(Clone, Debug, Default)]
pub struct Stepper {
    pub bottom_columns_mask: u32,
    pub filled: [u32; 32],
    pub runs: Runs,
    pub zipper: Zipper,
}

impl Stepper {
    #[inline(always)]
    pub const fn from_live(bottom_edge: u32, live: u32, row: u32) -> Stepper {
        debug_assert!(live != 0, "no live columns to start from");
        let runs = Runs::from_blocks(bottom_edge, live);

        let mut bottom_column_heights = [INACTIVE_VERTEX; GRID_SIDE];
        let mut vertex_mask = 0u32;

        let mut idx = 0;
        while idx < runs.count() {
            let run = runs.at_index(idx);
            bottom_column_heights[run.start() as usize] = row as u8;
            bottom_column_heights[run.end() as usize] = row as u8;
            vertex_mask |= run.boundary_mask();
            idx += 1;
        }

        Stepper {
            bottom_columns_mask: bottom_edge,
            filled: [0; GRID_SIDE],
            runs,
            zipper: Zipper {
                bottom_column_heights,
                bottom_columns_mask: vertex_mask,
            },
        }
    }

    #[inline(always)]
    pub fn from_bottom_edge(bottom_edge: u32, seed_col: u32, row: u32) -> Stepper {
        Self::from_live(bottom_edge, 1 << seed_col, row)
    }

    // TODO - Benchmark and optimize
    #[inline(always)]
    pub fn step_to(
        &mut self,
        top_row: usize,
        top_edge: u32,
        top_guard: u32,
        mut on_triangle: impl FnMut(Triangle2D),
    ) {
        debug_assert!(
            top_row > 0 && top_row <= GRID_SIDE,
            "Top-row should be in 1..={GRID_SIDE}: {top_row}"
        );

        self.filled[top_row - 1] = self.runs.fill_mask();

        let caps = self.runs.guard_caps(top_guard);
        let vertices = (self.bottom_columns_mask ^ top_edge) | caps;

        if vertices & self.runs.scan_mask() != 0 {
            self.runs.iterate_runs(|run| {
                let scan_mask = run.scan_mask();
                self.zipper
                    .zip(scan_mask, top_row, vertices, &mut on_triangle);
            });

            self.runs.advance(top_edge, top_guard);
            let scan_mask = self.runs.scan_mask();

            self.zipper.advance(top_row, vertices, scan_mask);

            self.bottom_columns_mask = top_edge;
        }
    }

    // TODO - Benchmark and optimize
    pub fn weave_to(
        &mut self,
        top_row: usize,
        top_cells: u32,
        mut on_triangle: impl FnMut(Triangle2D),
    ) {
        debug_assert!(
            top_row > 0 && top_row <= GRID_SIDE,
            "top_row out of range: {top_row}"
        );
        self.filled[top_row - 1] = self.runs.fill_mask();

        let top_columns_mask = bitmasks::segment_boundaries(top_cells);
        let vertices = self.bottom_columns_mask ^ top_columns_mask;
        if vertices == 0 {
            return;
        }

        self.runs.iterate_runs(|run| {
            self.zipper
                .zip(run.scan_mask(), top_row, vertices, &mut on_triangle);
        });
        self.runs = Runs::from_blocks(top_columns_mask, top_cells);
        self.zipper
            .advance(top_row, vertices, self.runs.scan_mask());
        self.bottom_columns_mask = top_columns_mask;
    }
}
