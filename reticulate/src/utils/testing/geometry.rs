use crate::{
    meshing::{Triangle, bitfields::PLANE_SIDE, triangulation::Triangle2D},
    utils::testing::fixtures::cells,
};

/// Fails unless the triangles cover exactly the cells of `expected` (one
/// word per row, bits over columns), each exactly once.
pub fn assert_covers_exactly(tris: &[Triangle2D], expected: &[u32]) {
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

/// How many triangles cover each cell centre, as `[row][col]`.
fn coverage(tris: &[Triangle2D]) -> [[u8; 32]; 32] {
    let mut counts = [[0u8; 32]; 32];
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
                // Nudge the sample off the cell centre so it never lies
                // exactly on a diagonal shared by two triangles.
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

/// Twice the signed area of a 2D triangle; positive when counter-clockwise.
pub fn area2(t: &Triangle2D) -> i32 {
    let [(x0, y0), (x1, y1), (x2, y2)] = t.vertices.map(|(c, r)| (c as i32, r as i32));
    (x1 - x0) * (y2 - y0) - (x2 - x0) * (y1 - y0)
}

/// Checks a scan of one plane: the claimed cells are exactly `expected`,
/// the triangles cover them once, wind counter-clockwise, and leave no
/// T-junctions.
pub fn assert_scan(filled: &[u32; PLANE_SIDE], tris: &[Triangle2D], expected: &[(usize, usize)]) {
    assert_eq!(filled, &cells(expected), "claimed cells differ");
    assert_covers_exactly(tris, filled);
    for t in tris {
        assert!(area2(t) > 0, "non-canonical winding: {t:?}");
    }
    assert_no_t_junctions(tris);
}

/// Fails if a vertex of one triangle lies strictly inside an edge of another.
pub fn assert_no_t_junctions(tris: &[Triangle2D]) {
    let points = |t: &Triangle2D| t.vertices.map(|(c, r)| (c as i32, r as i32));
    for (i, a) in tris.iter().enumerate() {
        for (j, b) in tris.iter().enumerate() {
            if i == j {
                continue;
            }
            let edges = points(b);
            for p in points(a) {
                for k in 0..3 {
                    let (e0, e1) = (edges[k], edges[(k + 1) % 3]);
                    let (dx, dy) = (e1.0 - e0.0, e1.1 - e0.1);
                    let (px, py) = (p.0 - e0.0, p.1 - e0.1);
                    let cross = dx * py - dy * px;
                    let dot = dx * px + dy * py;
                    if cross == 0 && dot > 0 && dot < dx * dx + dy * dy {
                        panic!("T-junction: {p:?} of {a:?} inside edge {e0:?}-{e1:?} of {b:?}");
                    }
                }
            }
        }
    }
}

/// 3D counterpart of [`assert_no_t_junctions`].
pub fn assert_no_t_junctions_3d(tris: &[Triangle]) {
    let points = |t: &Triangle| {
        t.vertices
            .clone()
            .map(|q| (q.x as i64, q.y as i64, q.z as i64))
    };
    for (i, a) in tris.iter().enumerate() {
        for (j, b) in tris.iter().enumerate() {
            if i == j {
                continue;
            }
            let edges = points(b);
            for p in points(a) {
                for k in 0..3 {
                    let (e0, e1) = (edges[k], edges[(k + 1) % 3]);
                    let d = (e1.0 - e0.0, e1.1 - e0.1, e1.2 - e0.2);
                    let v = (p.0 - e0.0, p.1 - e0.1, p.2 - e0.2);
                    let cross = (
                        d.1 * v.2 - d.2 * v.1,
                        d.2 * v.0 - d.0 * v.2,
                        d.0 * v.1 - d.1 * v.0,
                    );
                    let dot = d.0 * v.0 + d.1 * v.1 + d.2 * v.2;
                    let len2 = d.0 * d.0 + d.1 * d.1 + d.2 * d.2;
                    if cross == (0, 0, 0) && dot > 0 && dot < len2 {
                        panic!("T-junction: {p:?} of {a:?} inside edge {e0:?}-{e1:?} of {b:?}");
                    }
                }
            }
        }
    }
}
