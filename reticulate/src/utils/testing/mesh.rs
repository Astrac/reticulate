#[cfg(debug_assertions)]
use crate::utils::debug_assertions::debug_assert_within_footprint;
use crate::{
    Axis, CHUNK_VOLUME, ChunkCoord, Coords2D, DenseWalker, Face, MaterialId, Morton3D, PLANE_AREA,
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
        fixtures::{EMPTY, Fixture, VoxelGrid, claims_of, parity_of, project, voxel},
    },
};

pub const POSITIVE: [Face; 3] = [
    Face::Positive(Axis::X),
    Face::Positive(Axis::Y),
    Face::Positive(Axis::Z),
];
pub const NEGATIVE: [Face; 3] = [
    Face::Negative(Axis::X),
    Face::Negative(Axis::Y),
    Face::Negative(Axis::Z),
];

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

/// Checks the mesh against the grid alone. Every exposed face cell on
/// planes 0..32 is covered exactly once, carries the solid side's
/// material, and points away from the solid. Nothing else is covered.
/// Plane 32 belongs to the neighbouring chunk. Winding must be
/// consistent: the same sign relative to the face direction everywhere.
pub fn assert_mesh_matches(grid: &VoxelGrid, tris: &[Triangle]) {
    let n = PLANE_SIDE;
    let slot = |a: usize, c: usize, u: usize, v: usize| ((a * n + c) * n + u) * n + v;
    let mut count = vec![0u8; 3 * n * n * n];
    let mut material = vec![EMPTY; 3 * n * n * n];
    let mut face = vec![Face::Positive(Axis::X); 3 * n * n * n];

    for t in tris {
        let a = (0..3)
            .find(|&a| t.face == POSITIVE[a] || t.face == NEGATIVE[a])
            .expect("unknown face");
        let p: Vec<(f64, f64, f64)> = t
            .vertices
            .iter()
            .map(|q| project(a, q.x as f64, q.y as f64, q.z as f64))
            .collect();
        let c = p[0].0;
        assert!(
            p.iter().all(|q| q.0 == c),
            "triangle not on one plane: {t:?}"
        );
        assert!(c >= 0.0 && (c as usize) < n, "triangle on plane {c}: {t:?}");

        let cross = (p[1].1 - p[0].1) * (p[2].2 - p[0].2) - (p[1].2 - p[0].2) * (p[2].1 - p[0].1);
        assert!(cross != 0.0, "degenerate triangle: {t:?}");
        assert!(cross < 0.0, "non-canonical winding: {t:?}");

        // outward_vertices must face away from the solid: +axis for positive
        // faces, -axis for negative ones.
        let o: Vec<(f64, f64, f64)> = t
            .outward_vertices()
            .iter()
            .map(|q| project(a, q.x as f64, q.y as f64, q.z as f64))
            .collect();

        let outward = (o[1].1 - o[0].1) * (o[2].2 - o[0].2) - (o[1].2 - o[0].2) * (o[2].1 - o[0].1);
        let expected = if t.face == POSITIVE[a] { 1.0 } else { -1.0 };
        assert_eq!(
            outward.signum(),
            expected,
            "outward_vertices faces inward: {t:?}"
        );

        let (u0, u1) = (
            p.iter().map(|q| q.1).fold(f64::MAX, f64::min),
            p.iter().map(|q| q.1).fold(0.0, f64::max),
        );
        let (v0, v1) = (
            p.iter().map(|q| q.2).fold(f64::MAX, f64::min),
            p.iter().map(|q| q.2).fold(0.0, f64::max),
        );
        for u in u0 as usize..(u1 as usize).min(n) {
            for v in v0 as usize..(v1 as usize).min(n) {
                let (x, y) = (u as f64 + 0.5 + 1e-4, v as f64 + 0.5 + 1.414e-4);
                let side = |a: (f64, f64, f64), b: (f64, f64, f64)| {
                    (b.1 - a.1) * (y - a.2) - (b.2 - a.2) * (x - a.1)
                };
                let (s0, s1, s2) = (side(p[0], p[1]), side(p[1], p[2]), side(p[2], p[0]));
                if (s0 > 0.0 && s1 > 0.0 && s2 > 0.0) || (s0 < 0.0 && s1 < 0.0 && s2 < 0.0) {
                    let i = slot(a, c as usize, u, v);
                    count[i] += 1;
                    material[i] = t.material;
                    face[i] = t.face;
                }
            }
        }
    }

    for a in 0..3 {
        for c in 0..n {
            for u in 0..n {
                for v in 0..n {
                    let (ci, ui, vi) = (c as i32, u as i32, v as i32);
                    let below = grid.at(voxel(a, ci - 1, ui, vi));
                    let here = grid.at(voxel(a, ci, ui, vi));
                    let exposed = (below == EMPTY) != (here == EMPTY);
                    let i = slot(a, c, u, v);
                    let at = format!("axis {a}, plane {c}, (u {u}, v {v})");
                    assert_eq!(count[i], exposed as u8, "{at}: covered {} times", count[i]);
                    if exposed {
                        let solid = below.max(here);
                        let expected = if below != EMPTY {
                            POSITIVE[a]
                        } else {
                            NEGATIVE[a]
                        };
                        assert_eq!(material[i], solid, "{at}: wrong material");
                        assert!(
                            face[i] == expected,
                            "{at}: face {face:?}, expected {expected:?}",
                        );
                    }
                }
            }
        }
    }

    assert_no_t_junctions_3d(tris);
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
