#[cfg(debug_assertions)]
use crate::utils::debug_assertions::debug_assert_within_footprint;
use crate::{
    Axis, CHUNK_VOLUME, ChunkCoord, Coords2D, DenseWalker, MaterialId, Morton3D, PLANE_AREA,
    PLANE_SIDE, TREE_DEPTH,
    meshing::{
        Triangle,
        bitfields::Material,
        surface_layer::SurfaceLayer,
        surface_scanner::SurfaceScanner,
        surface_walker::SurfaceWalker,
        triangulation::{Triangle2D, edges::Edges, filler::Filler, weaver::Weaver},
    },
    utils::testing::{
        assert_no_t_junctions_3d,
        fixtures::{EMPTY, Fixture, VoxelGrid, claims_of, parity_of, voxel},
    },
};

/// An aligned block of the chunk that is walked as a single node:
/// `((x, y, z), depth)`.
pub type Block = ((usize, usize, usize), usize);

/// Walks `grid` in Morton order as a dense tree. Every voxel is a leaf at
/// maximum depth except inside `blocks`, which are reported as single
/// larger nodes (and must therefore be uniform).
pub fn dense_walk<W: DenseWalker<MaterialId>>(walker: &mut W, grid: &VoxelGrid, blocks: &[Block]) {
    let starts: Vec<Morton3D> = blocks
        .iter()
        .map(|&((x, y, z), d)| {
            let side = PLANE_SIDE >> d;
            assert!(
                x % side == 0 && y % side == 0 && z % side == 0,
                "block must be aligned"
            );
            Morton3D::from_xyz_and_depth(x as u8, y as u8, z as u8, d.into())
        })
        .collect();

    let mut code = 0u32;
    while (code as usize) < CHUNK_VOLUME {
        let depth = starts
            .iter()
            .find(|&&s| s.as_u32() == code)
            .map_or(TREE_DEPTH, |&s| s.depth().as_usize());

        let shift = 3 * (TREE_DEPTH - depth) as u32;
        let span = 1u32 << shift;
        let material = grid.0[code as usize];

        assert!(
            (code..code + span).all(|i| grid.0[i as usize] == material),
            "block at Morton {code} is not uniform"
        );

        let morton = Morton3D::from_code(code >> shift, depth.into());
        if material == EMPTY {
            walker.on_empty(morton);
        } else {
            walker.on_leaf(morton, &material);
        }
        code += span;
    }
}

/// Meshes the chunk once with fresh material planes and the fixture's
/// (brute-force) parity and claims. Claims persist in the fixture, so a
/// second call only emits what the first one did not claim.
pub fn mesh(f: &mut Fixture, blocks: &[Block]) -> Vec<Triangle> {
    let mut tris = Vec::new();
    let mut cache: [[MaterialId; PLANE_AREA]; 3] = [[0; PLANE_AREA]; 3];
    {
        let [mx, my, mz] = &mut cache;
        let mut materials = Material {
            planes: [mx, my, mz],
        };
        let mut claims = claims_of(&mut f.claims);
        let parity = parity_of(&mut f.parity);
        let scanner = SurfaceScanner::new(&mut claims, &parity, |t| tris.push(t));
        let mut to_material = |m: &MaterialId| *m;
        let mut walker = SurfaceWalker::new(&mut materials, scanner, &mut to_material);
        dense_walk(&mut walker, &f.grid, blocks);
    }
    tris
}

/// Checks the mesh against the grid alone.
///
/// For every plane `c` in 0..PLANE_SIDE and every cell (u, v) on it:
///  * if the cell is exposed (exactly one of the voxels on either side of
///    the plane is solid), the triangles must cover it with a total area of
///    exactly 1, carry the solid side's material, and have the face sign
///    pointing away from the solid;
///  * otherwise nothing may touch it.
///
/// Every triangle must also lie on one axis-aligned plane inside the chunk
/// (plane 32 belongs to the neighbouring chunk) and use the canonical winding.
pub fn assert_mesh_matches(grid: &VoxelGrid, tris: &[Triangle]) {
    let n = PLANE_SIDE as i32;
    let slot = |a: usize, c: i32, u: i32, v: i32| (((a as i32 * n + c) * n + u) * n + v) as usize;

    // Some((solid material, face_positive)) if the face between planes c-1 and c
    // at (u, v) is exposed, None otherwise.
    let expected = |a: usize, c: i32, u: i32, v: i32| {
        let below = grid.at(voxel(a, c - 1, u, v));
        let here = grid.at(voxel(a, c, u, v));
        match (below == EMPTY, here == EMPTY) {
            (false, true) => Some((below, true)), // solid below: faces +axis
            (true, false) => Some((here, false)), // solid above: faces -axis
            _ => None,
        }
    };

    let mut covered = vec![0.0f64; 3 * (n * n * n) as usize];

    for t in tris {
        let face = t.face();
        let a = face.axis() as usize; // must match the axis index `voxel(a, ..)` expects
        let verts = t.vertices().map(|q| [q.x as i32, q.y as i32, q.z as i32]);

        // All three vertices must lie on the plane perpendicular to the face's axis.
        let c = verts[0][a];
        assert!(
            verts.iter().all(|q| q[a] == c),
            "triangle not on the plane of its face {face:?}: {t:?}"
        );
        assert!((0..n).contains(&c), "triangle on plane {c}: {t:?}");

        // In-plane coordinates (u, v). Must be the inverse of `voxel`.
        let p = verts.map(|q| (q[(a + 1) % 3], q[(a + 2) % 3]));
        assert!(
            p.iter()
                .all(|&(u, v)| (0..=n).contains(&u) && (0..=n).contains(&v)),
            "triangle leaves the chunk: {t:?}"
        );

        // Same winding everywhere in the (u, v) projection.
        let cross = (p[1].0 - p[0].0) * (p[2].1 - p[0].1) - (p[1].1 - p[0].1) * (p[2].0 - p[0].0);
        assert!(cross < 0, "degenerate or non-canonical winding: {t:?}");

        let (u0, u1) = (
            p.iter().map(|q| q.0).min().unwrap(),
            p.iter().map(|q| q.0).max().unwrap(),
        );
        let (v0, v1) = (
            p.iter().map(|q| q.1).min().unwrap(),
            p.iter().map(|q| q.1).max().unwrap(),
        );
        for u in u0..u1 {
            for v in v0..v1 {
                let area = area_in_cell(p, u, v);
                if area < 1e-9 {
                    continue;
                }
                let at = format!("axis {a}, plane {c}, (u {u}, v {v})");
                let Some((material, positive)) = expected(a, c, u, v) else {
                    panic!("{at}: covers a cell that is not exposed: {t:?}");
                };
                assert_eq!(
                    face.is_positive(),
                    positive,
                    "{at}: wrong face direction: {t:?}"
                );
                assert_eq!(t.material(), material, "{at}: wrong material: {t:?}");
                assert_eq!(
                    t.is_normal_positive(),
                    positive,
                    "{at}: wrong face direction: {t:?}"
                );
                covered[slot(a, c, u, v)] += area;
            }
        }
    }

    for a in 0..3 {
        for c in 0..n {
            for u in 0..n {
                for v in 0..n {
                    let want = expected(a, c, u, v).is_some() as u8 as f64;
                    let got = covered[slot(a, c, u, v)];
                    assert!(
                        (got - want).abs() < 1e-9,
                        "axis {a}, plane {c}, (u {u}, v {v}): covered area {got}, expected {want}"
                    );
                }
            }
        }
    }

    assert_no_t_junctions_3d(tris);
}

/// Area of the intersection of triangle `tri` with the unit cell [u,u+1]x[v,v+1].
fn area_in_cell(tri: [(i32, i32); 3], u: i32, v: i32) -> f64 {
    let mut poly: Vec<(f64, f64)> = tri.iter().map(|&(x, y)| (x as f64, y as f64)).collect();
    for (axis, bound, keep_greater) in [
        (0, u, true),
        (0, u + 1, false),
        (1, v, true),
        (1, v + 1, false),
    ] {
        poly = clip(&poly, axis, bound as f64, keep_greater);
    }
    let twice: f64 = (0..poly.len())
        .map(|i| {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            a.0 * b.1 - a.1 * b.0
        })
        .sum();
    twice.abs() / 2.0
}

/// One Sutherland–Hodgman step against the half-plane coord[axis] >= / <= bound.
fn clip(poly: &[(f64, f64)], axis: usize, bound: f64, keep_greater: bool) -> Vec<(f64, f64)> {
    let coord = |p: (f64, f64)| if axis == 0 { p.0 } else { p.1 };
    let inside = |p| {
        if keep_greater {
            coord(p) >= bound
        } else {
            coord(p) <= bound
        }
    };
    let mut out = Vec::with_capacity(poly.len() + 2);
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if inside(a) {
            out.push(a);
        }
        if inside(a) != inside(b) {
            let s = (bound - coord(a)) / (coord(b) - coord(a));
            out.push((a.0 + s * (b.0 - a.0), a.1 + s * (b.1 - a.1)));
        }
    }
    out
}

/// Fills and weaves one face plane from `seed` = (row, col), without
/// claiming anything. Returns the filled cells and the emitted triangles.
pub fn scan_plane(
    f: &mut Fixture,
    axis: Axis,
    axis_coord: ChunkCoord,
    seed: (u32, u32),
) -> ([u32; PLANE_SIDE], Vec<Triangle2D>) {
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);
    let weaver = Weaver::new(&parity);
    let mut tris = Vec::new();
    let layer = SurfaceLayer::new(axis, axis_coord);
    let filler = Filler::new(&claims, &parity);
    let edges = Edges::from_parity(&parity, layer);

    let filled = filler.fill_region(
        layer,
        &edges,
        Coords2D {
            u: seed.0 as u8,
            v: seed.1 as u8,
        },
    );

    weaver.weave_region(layer, &edges, &filled, |t| tris.push(t));
    #[cfg(debug_assertions)]
    debug_assert_within_footprint(&parity, layer, &filled);

    (filled, tris)
}
