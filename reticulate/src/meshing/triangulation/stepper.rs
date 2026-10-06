use crate::{
    BoundaryMask, CellMask,
    meshing::triangulation::{Triangle2D, runs::Runs, zipper::Zipper},
    utils::bitmasks,
};

pub const BOUNDARIES_COUNT: usize = GRID_SIDE + 1;
pub const GRID_SIDE: usize = 32;
pub const INACTIVE_VERTEX: u8 = 0xFF;

#[derive(Clone, Debug, Default)]
pub struct Stepper {
    pub filled: [u32; GRID_SIDE],
    pub runs: Runs,
    pub zipper: Zipper,
}

impl Stepper {
    // TODO - Can row_index be u8 here?
    #[inline(always)]
    pub const fn from_live(bottom_edge: BoundaryMask, live: CellMask, row_index: u32) -> Stepper {
        debug_assert!(live != 0, "no live columns to start from");
        let runs = Runs::from_blocks(bottom_edge, live);

        let mut bottom_column_heights = [INACTIVE_VERTEX; BOUNDARIES_COUNT];
        let mut vertex_mask: BoundaryMask = 0;

        let mut idx = 0;
        while idx < runs.count() {
            let run = runs.at_index(idx);
            bottom_column_heights[run.start() as usize] = row_index as u8;
            bottom_column_heights[run.end() as usize] = row_index as u8;
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
    pub const fn add_vertices(&mut self, mask: BoundaryMask, row: u32) {
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
        top_cells: CellMask,
        extra_vertices: BoundaryMask,
        mut on_triangle: impl FnMut(Triangle2D),
    ) {
        debug_assert!(
            top_row > 0 && top_row <= GRID_SIDE,
            "Top row is out of the (0; {GRID_SIDE}] range: {top_row}"
        );
        debug_assert_eq!(
            extra_vertices >> BOUNDARIES_COUNT,
            0,
            "Extra vertex found beyond index 32 of a boundary mask"
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
        debug_assert_eq!(
            self.runs.fill_mask(),
            top_cells,
            "The computed runs fill_mask must cover exactly the given top row cells"
        );

        self.zipper
            .advance(top_row, vertices, self.runs.scan_mask());
    }
}
