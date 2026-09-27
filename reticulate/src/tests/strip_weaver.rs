use crate::meshing::face::{
    StripTriangle,
    strip_weaver::{INACTIVE_VERTEX, StripWeaver},
};

#[test]
fn init_on_boundary() {
    let strip_weaver = StripWeaver::from_bottom_edge(0b010010001, 0, 5);
    let mut expected_vertices = [INACTIVE_VERTEX; 32];
    expected_vertices[0] = 5;
    expected_vertices[4] = 5;

    let mut filled = [0; 32];
    filled[5] = 0b1111;

    assert_eq!(strip_weaver.runs_count, 1);
    assert_eq!(strip_weaver.runs[0].start, 0);
    assert_eq!(strip_weaver.runs[0].end, 4);
    assert_eq!(strip_weaver.zipper.bottom_vs, expected_vertices);
    assert_eq!(strip_weaver.fill_mask(), 0b1111);
    assert_eq!(strip_weaver.filled, filled);
}

#[test]
fn init_in_middle() {
    let strip_weaver = StripWeaver::from_bottom_edge(0b010010001, 6, 5);
    let mut expected_vertices = [INACTIVE_VERTEX; 32];
    expected_vertices[4] = 5;
    expected_vertices[7] = 5;

    let mut filled = [0; 32];
    filled[5] = 0b1110000;

    assert_eq!(strip_weaver.runs_count, 1);
    assert_eq!(strip_weaver.runs[0].start, 4);
    assert_eq!(strip_weaver.runs[0].end, 7);
    assert_eq!(strip_weaver.zipper.bottom_vs, expected_vertices);
    assert_eq!(strip_weaver.fill_mask(), 0b1110000);
    assert_eq!(strip_weaver.filled, filled);
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
            StripTriangle {
                vertices: [(1, 1), (1, 0), (6, 0)]
            },
            StripTriangle {
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
            StripTriangle {
                vertices: [(1, 2), (1, 0), (6, 0)]
            },
            StripTriangle {
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
            StripTriangle {
                vertices: [(1, 1), (1, 0), (2, 1)]
            },
            StripTriangle {
                vertices: [(2, 1), (1, 0), (3, 1)]
            },
            StripTriangle {
                vertices: [(3, 1), (1, 0), (4, 0)]
            },
            StripTriangle {
                vertices: [(3, 1), (4, 0), (4, 1)]
            },
            StripTriangle {
                vertices: [(2, 2), (2, 1), (3, 1)]
            },
            StripTriangle {
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
            StripTriangle {
                vertices: [(2, 1), (2, 0), (3, 0)]
            },
            StripTriangle {
                vertices: [(2, 1), (3, 0), (3, 1)]
            },
            StripTriangle {
                vertices: [(1, 2), (1, 1), (2, 1)]
            },
            StripTriangle {
                vertices: [(1, 2), (2, 1), (3, 1)]
            },
            StripTriangle {
                vertices: [(1, 2), (3, 1), (4, 1)]
            },
            StripTriangle {
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
            StripTriangle {
                vertices: [(2, 1), (1, 0), (3, 1)]
            },
            StripTriangle {
                vertices: [(3, 1), (1, 0), (4, 0)]
            },
            StripTriangle {
                vertices: [(2, 2), (1, 0), (2, 1)]
            },
            StripTriangle {
                vertices: [(3, 2), (3, 1), (4, 0)]
            },
            StripTriangle {
                vertices: [(1, 3), (1, 0), (2, 2)]
            },
            StripTriangle {
                vertices: [(1, 3), (2, 2), (3, 2)]
            },
            StripTriangle {
                vertices: [(1, 3), (3, 2), (4, 3)]
            },
            StripTriangle {
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
            StripTriangle {
                vertices: [(2, 2), (2, 0), (3, 0)]
            },
            StripTriangle {
                vertices: [(0, 3), (0, 2), (1, 3)]
            },
            StripTriangle {
                vertices: [(1, 3), (0, 2), (2, 2)]
            },
            StripTriangle {
                vertices: [(1, 4), (1, 3), (3, 4)]
            },
            StripTriangle {
                vertices: [(3, 4), (1, 3), (2, 2)]
            },
            StripTriangle {
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
            StripTriangle {
                vertices: [(2, 1), (1, 0), (3, 1)]
            },
            StripTriangle {
                vertices: [(3, 1), (1, 0), (4, 0)]
            },
            StripTriangle {
                vertices: [(2, 2), (1, 0), (2, 1)]
            },
            StripTriangle {
                vertices: [(3, 2), (3, 1), (4, 0)]
            },
            StripTriangle {
                vertices: [(1, 3), (1, 0), (2, 2)]
            },
            StripTriangle {
                vertices: [(1, 3), (2, 2), (2, 3)]
            },
            StripTriangle {
                vertices: [(2, 3), (2, 2), (3, 2)]
            },
            StripTriangle {
                vertices: [(2, 3), (3, 2), (4, 3)]
            },
            StripTriangle {
                vertices: [(4, 3), (3, 2), (4, 0)]
            },
            StripTriangle {
                vertices: [(2, 4), (2, 3), (3, 4)]
            },
            StripTriangle {
                vertices: [(3, 4), (2, 3), (4, 3)]
            },
            StripTriangle {
                vertices: [(3, 4), (4, 3), (5, 4)]
            },
            StripTriangle {
                vertices: [(5, 4), (4, 3), (6, 4)]
            },
            StripTriangle {
                vertices: [(6, 4), (4, 3), (7, 3)]
            },
            StripTriangle {
                vertices: [(3, 5), (3, 4), (5, 4)]
            },
            StripTriangle {
                vertices: [(3, 5), (5, 4), (5, 5)]
            },
            StripTriangle {
                vertices: [(7, 5), (6, 4), (7, 3)]
            },
            StripTriangle {
                vertices: [(6, 6), (6, 4), (7, 5)]
            },
            StripTriangle {
                vertices: [(6, 6), (7, 5), (8, 5)]
            },
            StripTriangle {
                vertices: [(6, 6), (8, 5), (8, 6)]
            }
        ]
    );
}

#[test]
fn guard_closure_with_continuing_wall() {
    // TODO - Use grid
    let edges = [0b1_0000_0011, 0b1_0000_0001, 0];
    let guards = [0, 0b1, 0];
    let tris = weave(&edges, &guards, 0, 0);
    assert_covers_exactly(&tris, &[0b1]);
}

#[test]
fn hidden_floor_caps_strip() {
    // TODO - Use grid
    let edges = [0b1_0001, 0b1_0001, 0b1_0001, 0b1_0001, 0];
    let guards = [0, 0, 0b1111, 0, 0b1111];
    let tris = weave(&edges, &guards, 0, 0);
    assert_covers_exactly(&tris, &[0b1111, 0b1111]);
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
#[ignore = "needs downward growth from the seed (step 3)"]
fn l_shape_seeded_mid_face() {
    // TODO - Use grid
    let mut edges = vec![0b1_0000_0010];
    edges.extend([0b1_0000_0001; 7]);
    edges.push(0);
    let mut guards = vec![0u32; 9];
    guards[1] = 0b1;
    guards[8] = 0xFF;
    let tris = weave(&edges, &guards, 0, 1);
    let mut expected = vec![0b1111_1110];
    expected.extend([0xFF; 7]);
    assert_covers_exactly(&tris, &expected);
}

struct Grid {
    edges: Vec<u32>,
    guards: Vec<u32>,
}

const EMPTY: u8 = b'.';

impl Grid {
    fn weave_triangles(&self) -> Vec<StripTriangle> {
        let mut triangles = Vec::new();
        let seed_col = self.edges[0].trailing_zeros();
        let mut weaver = StripWeaver::from_bottom_edge(self.edges[0], seed_col, 0);

        let mut index = 1;
        while index < self.edges.len() {
            weaver.weave_strip(index, self.edges[index], self.guards[index], |t| {
                triangles.push(t)
            });
            index += 1;
        }

        triangles
    }

    fn parse(grid: &[&str]) -> Grid {
        let height = grid.len();
        if height == 0 {
            return Grid {
                edges: vec![],
                guards: vec![],
            };
        }

        let rows: Vec<&[u8]> = grid.iter().map(|s| s.as_bytes()).collect();
        let width = rows.iter().map(|r| r.len()).max().unwrap_or(0);

        let get_cell = |x: usize, y: usize| -> u8 {
            let row = rows[height - 1 - y];
            if x >= row.len() { EMPTY } else { row[x] }
        };

        let get_cell_rev = |r: usize, y: usize| -> u8 {
            if width == 0 {
                return EMPTY;
            }
            get_cell(width - 1 - r, y)
        };

        let mut horizontal = Vec::with_capacity(height);
        for y in 0..height {
            let mut mask: u32 = 0;
            for r in 0..width {
                let prev = if r == 0 {
                    EMPTY
                } else {
                    get_cell_rev(r - 1, y)
                };
                let curr = get_cell_rev(r, y);
                if prev != curr {
                    mask |= 1 << r;
                }
            }
            horizontal.push(mask);
        }

        let mut vertical_by_column = vec![0u32; width];
        for x in 0..width {
            let mut mask: u32 = 0;
            for y in 0..height {
                let below = if y == 0 { EMPTY } else { get_cell(x, y - 1) };
                let above = get_cell(x, y);
                if below != above {
                    mask |= 1 << y;
                }
            }
            vertical_by_column[x] = mask;
        }

        let mut vertical = vec![0u32; height];
        for x in 0..width {
            let r = width - 1 - x;
            for y in 0..height {
                if (vertical_by_column[x] >> y) & 1 == 1 {
                    vertical[y] |= 1 << r;
                }
            }
        }

        Grid {
            edges: horizontal,
            guards: vertical,
        }
    }
}

fn weave(edges: &[u32], guards: &[u32], seed_col: u32, seed_row: u32) -> Vec<StripTriangle> {
    let mut out = Vec::new();
    let mut w = StripWeaver::from_bottom_edge(edges[seed_row as usize], seed_col, seed_row);
    let mut row = seed_row as usize + 1;
    while w.runs_count > 0 && row < edges.len() {
        w.weave_strip(row, edges[row], guards[row], |t| out.push(t));
        row += 1;
    }
    out
}

fn coverage(tris: &[StripTriangle]) -> [[u8; 32]; 32] {
    let mut counts = [[0u8; 32]; 32]; // [row][col]
    for t in tris {
        let p: Vec<(f64, f64)> = t
            .vertices
            .iter()
            .map(|&(c, r)| (c as f64, r as f64))
            .collect();
        let area2 = (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[2].0 - p[0].0) * (p[1].1 - p[0].1);
        assert!(area2 != 0.0, "degenerate triangle: {:?}", t);
        for row in 0..32 {
            for col in 0..32 {
                let (x, y) = (col as f64 + 0.5 + 1e-4, row as f64 + 0.5 + 1.414e-4);
                let side = |a: (f64, f64), b: (f64, f64)| {
                    (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0)
                };
                let (s0, s1, s2) = (side(p[0], p[1]), side(p[1], p[2]), side(p[2], p[0]));
                if (s0 > 0.0 && s1 > 0.0 && s2 > 0.0) || (s0 < 0.0 && s1 < 0.0 && s2 < 0.0) {
                    counts[row][col] += 1;
                }
            }
        }
    }
    counts
}

fn assert_covers_exactly(tris: &[StripTriangle], expected: &[u32]) {
    let counts = coverage(tris);
    for row in 0..32 {
        for col in 0..32 {
            let want = expected.get(row).map_or(0, |m| (m >> col) & 1) as u8;
            assert_eq!(
                counts[row][col], want,
                "cell (col {col}, row {row}) covered {} times, expected {want}",
                counts[row][col]
            );
        }
    }
}
