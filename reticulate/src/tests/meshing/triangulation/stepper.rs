use crate::{
    meshing::triangulation::{
        Triangle2D,
        stepper::{INACTIVE_VERTEX, Stepper},
    },
    utils::testing::{AsciiGrid, assert_covers_exactly, weave_rows},
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
    stepper.weave_to(6, 0, 0, |_| {});

    let mut expected = [0; 32];
    expected[5] = 0b1111;
    assert_eq!(stepper.filled, expected);
    assert_eq!(stepper.runs.count(), 0);
}

#[test]
fn weave_strip_simple_rectangle() {
    let grid = AsciiGrid::parse(&["........................", "..................xxxxx."]);

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
    let grid = AsciiGrid::parse(&[
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
    let grid = AsciiGrid::parse(&[
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
    let grid = AsciiGrid::parse(&[
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
    let grid = AsciiGrid::parse(&[
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
    let grid = AsciiGrid::parse(&[
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
    let grid = AsciiGrid::parse(&[
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
    let grid = AsciiGrid::parse(&rows);
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
    let grid = AsciiGrid::parse(&rows);
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
fn l_shape_with_a_missing_corner_cell() {
    let mut cells = vec![0b1111_1110];
    cells.extend([0xFF; 7]);
    assert_covers_exactly(&weave_rows(&cells), &cells);
}

#[test]
fn from_live_starts_one_run_per_group_of_live_columns() {
    // Walls at 1, 3, 5 and 8; live columns 2 and 6 belong to different runs.
    let edge = 0b1_0010_1010;
    let stepper = Stepper::from_live(edge, (1 << 2) | (1 << 6), 4);

    assert_eq!(stepper.runs.count(), 2);

    let mut expected = [INACTIVE_VERTEX; 32];
    for c in [1, 3, 5, 8] {
        expected[c] = 4;
    }
    assert_eq!(stepper.zipper.bottom_column_heights, expected);
    assert_eq!(
        stepper.zipper.bottom_columns_mask,
        (1 << 1) | (1 << 3) | (1 << 5) | (1 << 8)
    );
}

#[test]
fn add_vertices_records_their_height_and_activates_them() {
    let mut stepper = Stepper::from_bottom_edge(0b1_0001, 2, 1);

    stepper.add_vertices(0b0_1100, 3);

    assert_eq!(stepper.zipper.bottom_column_heights[2], 3);
    assert_eq!(stepper.zipper.bottom_column_heights[3], 3);
    assert_eq!(stepper.zipper.bottom_column_heights[0], 1);
    assert_eq!(stepper.zipper.bottom_columns_mask, 0b1_1101);
}

#[test]
fn weaving_an_identical_row_adds_no_triangles() {
    let mut stepper = Stepper::from_bottom_edge(0b1_0001, 2, 0);
    let mut tris = Vec::new();

    // The row above has the same cells 0..4, so the strip just keeps growing.
    stepper.weave_to(1, 0b1111, 0, |t| tris.push(t));

    assert!(tris.is_empty());
    assert_eq!(stepper.filled[0], 0b1111);
    assert_eq!(stepper.runs.count(), 1);
}

#[test]
fn extra_vertices_force_triangles_on_a_straight_edge() {
    let mut stepper = Stepper::from_bottom_edge(0b1_0001, 2, 0);
    let mut tris = Vec::new();

    // A vertex requested at column 2 (e.g. to meet a neighbouring region's
    // corner) splits the otherwise untouched strip.
    stepper.weave_to(1, 0b1111, 1 << 2, |t| tris.push(t));

    assert!(!tris.is_empty());
    assert!(tris.iter().any(|t| t.vertices.contains(&(2, 1))));
}

#[test]
fn weaving_a_closed_region_empties_the_runs() {
    let mut stepper = Stepper::from_bottom_edge(0b1_0001, 2, 0);

    stepper.weave_to(1, 0, 0, |_| {});

    assert_eq!(stepper.runs.count(), 0);
    assert_eq!(stepper.filled[0], 0b1111);
}
