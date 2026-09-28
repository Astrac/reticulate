use crate::{
    meshing::triangulation::{
        Triangle2D,
        stepper::{INACTIVE_VERTEX, Stepper},
    },
    tests::assert_covers_exactly,
    utils::bitmasks,
};

#[test]
fn init_on_boundary() {
    let stepper = Stepper::from_bottom_edge(0b010010001, 0, 5);
    let mut expected_vertices = [INACTIVE_VERTEX; 32];
    expected_vertices[0] = 5;
    expected_vertices[4] = 5;

    assert_eq!(stepper.runs.count(), 1);
    assert_eq!(stepper.runs.all()[0].start(), 0);
    assert_eq!(stepper.runs.all()[0].end(), 4);
    assert_eq!(stepper.zipper.bottom_column_heights, expected_vertices);
    assert_eq!(stepper.filled, [0; 32]);
    assert_eq!(stepper.runs.fill_mask(), 0b1111);
}

#[test]
fn init_in_middle() {
    let stepper = Stepper::from_bottom_edge(0b010010001, 6, 5);
    let mut expected_vertices = [INACTIVE_VERTEX; 32];
    expected_vertices[4] = 5;
    expected_vertices[7] = 5;

    assert_eq!(stepper.runs.count(), 1);
    assert_eq!(stepper.runs.all()[0].start(), 4);
    assert_eq!(stepper.runs.all()[0].end(), 7);
    assert_eq!(stepper.zipper.bottom_column_heights, expected_vertices);
    assert_eq!(stepper.filled, [0; 32]);
    assert_eq!(stepper.runs.fill_mask(), 0b1110000);
}

#[test]
fn first_step_records_start_row() {
    let mut stepper = Stepper::from_bottom_edge(0b010010001, 0, 5);
    stepper.weave_to(6, 0, |_| {});

    let mut expected = [0; 32];
    expected[5] = 0b1111;
    assert_eq!(stepper.filled, expected);
    assert_eq!(stepper.runs.count(), 0);
}

#[test]
fn weave_strip_simple_rectangle() {
    let grid = Grid::parse(&vec![
        "........................",
        "..................xxxxx.",
    ]);

    assert_eq!(
        grid.weave_triangles(),
        vec![
            Triangle2D {
                vertices: [(1, 1), (1, 0), (6, 0)]
            },
            Triangle2D {
                vertices: [(1, 1), (6, 0), (6, 1)]
            }
        ]
    );
}

#[test]
fn weave_strip_multirow_rectangle() {
    let grid = Grid::parse(&vec![
        "........................",
        "..................xxxxx.",
        "..................xxxxx.",
    ]);

    assert_eq!(
        grid.weave_triangles(),
        vec![
            Triangle2D {
                vertices: [(1, 2), (1, 0), (6, 0)]
            },
            Triangle2D {
                vertices: [(1, 2), (6, 0), (6, 2)]
            }
        ]
    );
}

#[test]
fn weave_strip_shrinking() {
    let grid = Grid::parse(&vec![
        "........................",
        ".....................x..",
        "....................xxx.",
    ]);

    assert_eq!(
        grid.weave_triangles(),
        vec![
            Triangle2D {
                vertices: [(1, 1), (1, 0), (2, 1)]
            },
            Triangle2D {
                vertices: [(2, 1), (1, 0), (3, 1)]
            },
            Triangle2D {
                vertices: [(3, 1), (1, 0), (4, 0)]
            },
            Triangle2D {
                vertices: [(3, 1), (4, 0), (4, 1)]
            },
            Triangle2D {
                vertices: [(2, 2), (2, 1), (3, 1)]
            },
            Triangle2D {
                vertices: [(2, 2), (3, 1), (3, 2)]
            },
        ]
    );
}

#[test]
fn weave_strip_growing() {
    let grid = Grid::parse(&vec![
        "........................",
        "....................xxx.",
        ".....................x..",
    ]);

    assert_eq!(
        grid.weave_triangles(),
        vec![
            Triangle2D {
                vertices: [(2, 1), (2, 0), (3, 0)]
            },
            Triangle2D {
                vertices: [(2, 1), (3, 0), (3, 1)]
            },
            Triangle2D {
                vertices: [(1, 2), (1, 1), (2, 1)]
            },
            Triangle2D {
                vertices: [(1, 2), (2, 1), (3, 1)]
            },
            Triangle2D {
                vertices: [(1, 2), (3, 1), (4, 1)]
            },
            Triangle2D {
                vertices: [(1, 2), (4, 1), (4, 2)]
            }
        ]
    );
}

#[test]
fn weave_strip_around_hole() {
    let grid = Grid::parse(&vec![
        "........................",
        "....................xxx.",
        "....................x.x.",
        "....................xxx.",
    ]);

    let triangles = grid.weave_triangles();

    assert_covers_exactly(&triangles, &[0b1110, 0b1010, 0b1110]);

    assert_eq!(
        triangles,
        vec![
            Triangle2D {
                vertices: [(2, 1), (1, 0), (3, 1)]
            },
            Triangle2D {
                vertices: [(3, 1), (1, 0), (4, 0)]
            },
            Triangle2D {
                vertices: [(2, 2), (1, 0), (2, 1)]
            },
            Triangle2D {
                vertices: [(3, 2), (3, 1), (4, 0)]
            },
            Triangle2D {
                vertices: [(1, 3), (1, 0), (2, 2)]
            },
            Triangle2D {
                vertices: [(1, 3), (2, 2), (3, 2)]
            },
            Triangle2D {
                vertices: [(1, 3), (3, 2), (4, 3)]
            },
            Triangle2D {
                vertices: [(4, 3), (3, 2), (4, 0)]
            }
        ]
    );
}

#[test]
fn weave_strip_complex_shape() {
    let grid = Grid::parse(&vec![
        "........................",
        ".....................xx.",
        ".....................xxx",
        ".....................x..",
        ".....................x..",
    ]);

    assert_eq!(
        grid.weave_triangles(),
        vec![
            Triangle2D {
                vertices: [(2, 2), (2, 0), (3, 0)]
            },
            Triangle2D {
                vertices: [(0, 3), (0, 2), (1, 3)]
            },
            Triangle2D {
                vertices: [(1, 3), (0, 2), (2, 2)]
            },
            Triangle2D {
                vertices: [(1, 4), (1, 3), (3, 4)]
            },
            Triangle2D {
                vertices: [(3, 4), (1, 3), (2, 2)]
            },
            Triangle2D {
                vertices: [(3, 4), (2, 2), (3, 0)]
            }
        ]
    );
}

#[test]
fn weave_strip_complex_shape_with_hole() {
    let grid = Grid::parse(&vec![
        "........................",
        "................xx......",
        ".................x.xx...",
        ".................xxxxx..",
        "....................xxx.",
        "....................x.x.",
        "....................xxx.",
    ]);

    assert_eq!(
        grid.weave_triangles(),
        vec![
            Triangle2D {
                vertices: [(2, 1), (1, 0), (3, 1)]
            },
            Triangle2D {
                vertices: [(3, 1), (1, 0), (4, 0)]
            },
            Triangle2D {
                vertices: [(2, 2), (1, 0), (2, 1)]
            },
            Triangle2D {
                vertices: [(3, 2), (3, 1), (4, 0)]
            },
            Triangle2D {
                vertices: [(1, 3), (1, 0), (2, 2)]
            },
            Triangle2D {
                vertices: [(1, 3), (2, 2), (2, 3)]
            },
            Triangle2D {
                vertices: [(2, 3), (2, 2), (3, 2)]
            },
            Triangle2D {
                vertices: [(2, 3), (3, 2), (4, 3)]
            },
            Triangle2D {
                vertices: [(4, 3), (3, 2), (4, 0)]
            },
            Triangle2D {
                vertices: [(2, 4), (2, 3), (3, 4)]
            },
            Triangle2D {
                vertices: [(3, 4), (2, 3), (4, 3)]
            },
            Triangle2D {
                vertices: [(3, 4), (4, 3), (5, 4)]
            },
            Triangle2D {
                vertices: [(5, 4), (4, 3), (6, 4)]
            },
            Triangle2D {
                vertices: [(6, 4), (4, 3), (7, 3)]
            },
            Triangle2D {
                vertices: [(3, 5), (3, 4), (5, 4)]
            },
            Triangle2D {
                vertices: [(3, 5), (5, 4), (5, 5)]
            },
            Triangle2D {
                vertices: [(7, 5), (6, 4), (7, 3)]
            },
            Triangle2D {
                vertices: [(6, 6), (6, 4), (7, 5)]
            },
            Triangle2D {
                vertices: [(6, 6), (7, 5), (8, 5)]
            },
            Triangle2D {
                vertices: [(6, 6), (8, 5), (8, 6)]
            }
        ]
    );
}

#[test]
fn weave_strip_two_holes_close_same_row() {
    let rows = vec![
        "................",
        "..xxxxxxxxxxx...",
        "..xxx.x.xxxxx...",
        "..xxxxxxxxxxx...",
    ];
    let grid = Grid::parse(&rows);
    let triangles = grid.weave_triangles();

    let width = rows.iter().map(|r| r.len()).max().unwrap();
    let expected: Vec<u32> = rows
        .iter()
        .rev()
        .map(|row| {
            let bytes = row.as_bytes();
            let mut mask = 0u32;
            for r in 0..width {
                let c = if width - 1 - r < bytes.len() {
                    bytes[width - 1 - r]
                } else {
                    b'.'
                };
                if c == b'x' {
                    mask |= 1 << r;
                }
            }
            mask
        })
        .collect();

    assert_covers_exactly(&triangles, &expected);
}

#[test]
fn weave_strip_branch_3() {
    let rows = vec![
        "..............",
        "......xxxxxx..",
        "......xxxxxx..",
        "....xxxxxxxx..",
        "....xxxxxxxx..",
        ".....xxxxx.x..",
        "....xxxxxxxx..",
        "....xxxxxxxx..",
    ];
    let grid = Grid::parse(&rows);
    let triangles = grid.weave_triangles();

    assert_covers_exactly(
        &triangles,
        &[
            0b00001111111100, // row 0
            0b00001111111100, // row 1
            0b00000111110100, // row 2
            0b00001111111100, // row 3
            0b00001111111100, // row 4
            0b00000011111100, // row 5
            0b00000011111100, // row 6
            0b00000000000000, // row 7
        ],
    );
}

#[test]
fn l_shape_seeded_mid_face() {
    let mut cells = vec![0b1111_1110];
    cells.extend([0xFF; 7]);
    assert_covers_exactly(&weave(&cells), &cells);
}

struct Grid {
    cells: Vec<u32>,
}

const EMPTY: u8 = b'.';

impl Grid {
    fn weave_triangles(&self) -> Vec<Triangle2D> {
        weave(&self.cells)
    }

    fn parse(grid: &[&str]) -> Grid {
        let width = grid.iter().map(|line| line.len()).max().unwrap_or(0);
        debug_assert!(width <= 32, "grid is wider than 32 columns: {width}");
        debug_assert!(
            grid.len() <= 32,
            "grid is taller than 32 rows: {}",
            grid.len()
        );

        let cells = grid
            .iter()
            .rev() // the last line of text is row 0
            .map(|line| {
                let bytes = line.as_bytes();
                (0..width).fold(0u32, |mask, r| {
                    let x = width - 1 - r;
                    let cell = bytes.get(x).copied().unwrap_or(EMPTY);
                    if cell != EMPTY { mask | (1 << r) } else { mask }
                })
            })
            .collect();

        Grid { cells }
    }
}

/// Weaves region rows from the bottom up: `cells[r]` are row r's cells.
/// Everything past the end of `cells` is empty, which closes the last runs.
fn weave(cells: &[u32]) -> Vec<Triangle2D> {
    let mut out = Vec::new();
    let Some(first) = cells.iter().position(|&c| c != 0) else {
        return out;
    };
    let edge = bitmasks::segment_boundaries(cells[first]);
    let mut stepper = Stepper::from_live(edge, cells[first], first as u32);
    for row in first + 1..=cells.len() {
        let top = cells.get(row).copied().unwrap_or(0);
        stepper.weave_to(row, top, |t| out.push(t));
    }
    out
}
