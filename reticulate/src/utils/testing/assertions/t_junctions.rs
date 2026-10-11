use std::collections::HashSet;

use crate::meshing::{Triangle, triangulation::Triangle2D};

pub fn assert_no_t_junctions(tris: &[Triangle]) {
    let tris: Vec<_> = tris
        .iter()
        .map(|t| t.vertices().map(|v| [v.x as i64, v.y as i64, v.z as i64]))
        .collect();

    let verts: HashSet<_> = tris.iter().flatten().copied().collect();

    for t in tris {
        for k in 0..3 {
            let (e0, e1) = (t[k], t[(k + 1) % 3]);
            let d = sub(e1, e0);
            let g = gcd(gcd(d[0].abs(), d[1].abs()), d[2].abs());
            if g <= 1 {
                continue;
            } // no interior lattice points
            let step = [d[0] / g, d[1] / g, d[2] / g];
            for s in 1..g {
                let p = [
                    e0[0] + step[0] * s,
                    e0[1] + step[1] * s,
                    e0[2] + step[2] * s,
                ];
                if verts.contains(&p) {
                    panic!("T-junction: vertex {p:?} inside edge {e0:?}-{e1:?} of {t:?}");
                }
            }
        }
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn sub(a: [i64; 3], b: [i64; 3]) -> [i64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn assert_no_t_junctions_2d(tris: &[Triangle2D]) {
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
