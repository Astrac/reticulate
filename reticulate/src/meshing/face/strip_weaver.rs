use crate::meshing::face::{StripTriangle, strip_run::StripRun};

pub const GRID_SIDE: usize = 32;
pub const INACTIVE_VERTEX: u8 = 0xFF;

#[derive(Clone, Copy, Debug, Default)]
pub struct StripWeaver {
    pub bottom_edge: u32,
    pub filled: [u32; 32],
    pub present: u32,
    pub runs: [StripRun; 16],
    pub runs_count: usize,
    pub vertices: [u8; 32],
}

impl StripWeaver {
    #[inline(always)]
    pub fn from_bottom_edge(bottom_edge: u32, seed_u: u32, seed_v: u32) -> StripWeaver {
        debug_assert!(bottom_edge != 0, "Bottom edge must not be full of 0s");

        let mut runs = [StripRun::default(); 16];
        runs[0] = StripRun::from_edge(bottom_edge, seed_u);

        let mut vertices = [INACTIVE_VERTEX; 32];
        vertices[runs[0].start as usize] = seed_v as u8;
        vertices[runs[0].end as usize] = seed_v as u8;

        let mut filled = [0; 32];
        filled[seed_v as usize] = runs[0].as_fill_mask();

        StripWeaver {
            bottom_edge,
            filled,
            present: (1 << runs[0].start) | (1 << runs[0].end),
            runs,
            runs_count: 1,
            vertices,
        }
    }

    // TODO - Benchmark and optimize
    pub fn weave_strip(
        &mut self,
        top_row: usize,
        top_edge: u32,
        top_guard: u32,
        on_triangle: impl FnMut(StripTriangle),
    ) {
        let vertices = self.bottom_edge ^ top_edge;

        if vertices & self.scan_mask() != 0 {
            self.make_triangles(top_row, vertices, on_triangle);
            self.set_new_runs(top_edge, top_guard);
            self.set_new_vertices(top_row, vertices);
            self.bottom_edge = top_edge;
        }

        self.filled[top_row] = self.fill_mask();
    }

    fn make_triangles(
        &mut self,
        top_row: usize,
        top_vertices: u32,
        mut on_triangle: impl FnMut(StripTriangle),
    ) {
        debug_assert!(self.runs_count > 0, "No bottom runs found");

        let mut triangle = StripTriangle::default();

        for run_index in 0..self.runs_count {
            let run_mask = self.runs[run_index].as_scan_mask();
            let mut relevant_top = top_vertices & run_mask;
            let mut relevant_bottom = self.present & run_mask;

            let Some(mut top_vertex) = relevant_top.lowest_one() else {
                continue;
            };

            let Some(mut bottom_vertex) = relevant_bottom.lowest_one() else {
                continue;
            };

            relevant_top -= relevant_top.isolate_lowest_one();
            relevant_bottom -= relevant_bottom.isolate_lowest_one();

            loop {
                triangle.vertices[0] = (top_vertex as u8, top_row as u8);
                triangle.vertices[1] = (bottom_vertex as u8, self.vertices[bottom_vertex as usize]);

                let next_top = relevant_top
                    .lowest_one()
                    .map(|v| v as u8)
                    .unwrap_or(INACTIVE_VERTEX);
                let next_bottom = relevant_bottom
                    .lowest_one()
                    .map(|v| v as u8)
                    .unwrap_or(INACTIVE_VERTEX);

                let down = next_bottom != INACTIVE_VERTEX
                    && self.vertices[next_bottom as usize] < self.vertices[bottom_vertex as usize];

                if next_bottom == INACTIVE_VERTEX && next_top == INACTIVE_VERTEX {
                    break;
                } else if next_top != INACTIVE_VERTEX && (next_top < next_bottom || down) {
                    relevant_top -= relevant_top.isolate_lowest_one();
                    top_vertex = next_top as u32;

                    triangle.vertices[2] = (top_vertex as u8, top_row as u8);
                    on_triangle(triangle);
                } else if down && bottom_vertex > top_vertex {
                    break;
                } else if !down && (next_bottom as u32) < top_vertex {
                    relevant_bottom -= relevant_bottom.isolate_lowest_one();
                    bottom_vertex = next_bottom as u32;
                } else {
                    relevant_bottom -= relevant_bottom.isolate_lowest_one();
                    bottom_vertex = next_bottom as u32;
                    self.present &= !(1 << bottom_vertex);

                    triangle.vertices[2] =
                        (bottom_vertex as u8, self.vertices[bottom_vertex as usize]);
                    on_triangle(triangle);
                }
            }

            self.present |= 1 << bottom_vertex;
        }
    }

    // TODO - fill_mask() here might not need to be recomputed if we
    // carry the index of the last row
    #[inline(always)]
    fn set_new_runs(&mut self, edge: u32, guard: u32) {
        let mut blocks = self.fill_mask() ^ guard;
        let mut found_runs = 0;
        let mut current_shift = 0;

        while let Some(next_one) = blocks.lowest_one() {
            blocks = blocks >> next_one;

            let start = current_shift + next_one;
            let length = blocks.trailing_ones();
            current_shift = start + length;

            let new_run = StripRun::from_edge(edge, start);
            self.runs[found_runs] = new_run;

            blocks = blocks >> length;
            found_runs += 1;
        }

        self.runs_count = found_runs;
    }

    #[inline(always)]
    const fn set_new_vertices(&mut self, row: usize, mut mask: u32) {
        self.present = (self.present & !mask) | (mask & self.scan_mask());
        while let Some(x) = mask.lowest_one() {
            mask &= !mask.isolate_lowest_one();
            self.vertices[x as usize] = row as u8;
        }
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
}
