use crate::meshing::triangulation::Triangle2D;

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
