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

    assert_eq!(strip_weaver.runs_count, 1);
    assert_eq!(strip_weaver.runs[0].start, 0);
    assert_eq!(strip_weaver.runs[0].end, 4);
    assert_eq!(strip_weaver.vertices, expected_vertices);
    assert_eq!(strip_weaver.fill_mask(), 0b1111);
    assert_eq!(strip_weaver.vertex_mask(), 0b10001);
}

#[test]
fn init_in_middle() {
    let strip_weaver = StripWeaver::from_bottom_edge(0b010010001, 6, 5);
    let mut expected_vertices = [INACTIVE_VERTEX; 32];
    expected_vertices[4] = 5;
    expected_vertices[7] = 5;

    assert_eq!(strip_weaver.runs_count, 1);
    assert_eq!(strip_weaver.runs[0].start, 4);
    assert_eq!(strip_weaver.runs[0].end, 7);
    assert_eq!(strip_weaver.vertices, expected_vertices);
    assert_eq!(strip_weaver.fill_mask(), 0b1110000);
    assert_eq!(strip_weaver.vertex_mask(), 0b10010000);
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
                vertices: [(1, 2), (2, 1), (4, 2)]
            },
            StripTriangle {
                vertices: [(4, 2), (2, 1), (3, 1)]
            },
            StripTriangle {
                vertices: [(4, 2), (3, 1), (4, 1)]
            },
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
                vertices: [(1, 3), (2, 2), (4, 3)]
            },
            StripTriangle {
                vertices: [(4, 3), (2, 2), (3, 2)]
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
                vertices: [(6, 6), (7, 5), (8, 6)]
            },
            StripTriangle {
                vertices: [(8, 6), (7, 5), (8, 5)]
            }
        ]
    );
}

struct Grid {
    edges: Vec<u32>,
    guards: Vec<u32>,
}

const EMPTY: u8 = b'.';

impl Grid {
    // This assumes the seed of the triangulated shape to be valid at (2, 0)
    fn weave_triangles(&self) -> Vec<StripTriangle> {
        let mut triangles = Vec::new();
        let mut weaver = StripWeaver::from_bottom_edge(self.edges[0], 2, 0);

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
