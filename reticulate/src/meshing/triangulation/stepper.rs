use crate::{
    meshing::triangulation::{Triangle2D, runs::Runs, zipper::Zipper},
    utils::bitmasks,
};

pub const GRID_SIDE: usize = 32;
pub const INACTIVE_VERTEX: u8 = 0xFF;

#[derive(Clone, Debug, Default)]
pub struct Stepper {
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

    #[inline(always)]
    pub const fn add_vertices(&mut self, mask: u32, row: u32) {
        let mut m = mask;
        while let Some(c) = m.lowest_one() {
            self.zipper.bottom_column_heights[c as usize] = row as u8;
            m &= m - 1;
        }
        self.zipper.bottom_columns_mask |= mask;
    }

    // TODO - Benchmark and optimize
    pub fn weave_to(
        &mut self,
        top_row: usize,
        top_cells: u32,
        extra_vertices: u32,
        mut on_triangle: impl FnMut(Triangle2D),
    ) {
        debug_assert!(
            top_row > 0 && top_row <= GRID_SIDE,
            "top_row out of range: {top_row}"
        );
        debug_assert_eq!(
            bitmasks::segment_boundaries(top_cells),
            top_cells ^ (top_cells << 1)
        );

        let bottom_cells = self.runs.fill_mask();
        self.filled[top_row - 1] = bottom_cells;

        let vertices = bitmasks::boundary_changes(bottom_cells, top_cells) | extra_vertices;
        if vertices == 0 {
            return;
        }

        self.runs.iterate_runs(|run| {
            self.zipper
                .zip(run.scan_mask(), top_row, vertices, &mut on_triangle);
        });
        self.runs = Runs::from_blocks(bitmasks::segment_boundaries(top_cells), top_cells);
        self.zipper
            .advance(top_row, vertices, self.runs.scan_mask());
    }
}
