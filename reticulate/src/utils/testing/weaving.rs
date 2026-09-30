use crate::{
    meshing::triangulation::{Triangle2D, stepper::Stepper},
    utils::bitmasks,
};

/// Weaves region rows from the bottom up: `cells[r]` are row r's cells.
/// Everything past the end of `cells` is empty, which closes the last runs.
pub fn weave_rows(cells: &[u32]) -> Vec<Triangle2D> {
    let mut out = Vec::new();
    let Some(first) = cells.iter().position(|&c| c != 0) else {
        return out;
    };
    let edge = bitmasks::segment_boundaries(cells[first]);
    let mut stepper = Stepper::from_live(edge, cells[first], first as u32);
    for row in first + 1..=cells.len() {
        let top = cells.get(row).copied().unwrap_or(0);
        stepper.weave_to(row, top, 0, |t| out.push(t));
    }
    out
}

const BLANK: u8 = b'.';

/// A region drawn as ASCII art: any character other than `.` is a cell.
/// The last line of text is row 0, and the last character of a line is
/// column 0, so the drawing reads like the plane seen from above.
pub struct AsciiGrid {
    pub cells: Vec<u32>,
}

impl AsciiGrid {
    pub fn parse(grid: &[&str]) -> AsciiGrid {
        let width = grid.iter().map(|line| line.len()).max().unwrap_or(0);
        debug_assert!(width <= 32, "grid is wider than 32 columns: {width}");
        debug_assert!(
            grid.len() <= 32,
            "grid is taller than 32 rows: {}",
            grid.len()
        );

        let cells = grid
            .iter()
            .rev()
            .map(|line| {
                let bytes = line.as_bytes();
                (0..width).fold(0u32, |mask, r| {
                    let x = width - 1 - r;
                    let cell = bytes.get(x).copied().unwrap_or(BLANK);
                    if cell != BLANK { mask | (1 << r) } else { mask }
                })
            })
            .collect();

        AsciiGrid { cells }
    }

    pub fn weave_triangles(&self) -> Vec<Triangle2D> {
        weave_rows(&self.cells)
    }
}
