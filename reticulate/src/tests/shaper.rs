use bitvec::prelude::*;

use crate::{
    Coords2D, DenseWalker, Morton3D,
    axis::*,
    face::*,
    meshing::{
        Triangle,
        bitfields::{
            CHUNK_VOLUME, Claims, ClaimsMask, MATERIAL_WORDS, Material, MaterialId, PARITY_WORDS,
            PLANE_SIDE, Parity, ParityMask,
        },
        surface_scanner::SurfaceScanner,
        surface_walker::SurfaceWalker,
    },
    octree::TREE_DEPTH,
};

const EMPTY: MaterialId = 0;
const STONE: MaterialId = 1;
const DIRT: MaterialId = 2;
const POSITIVE: [Face; 3] = [
    Face::Positive(Axis::X),
    Face::Positive(Axis::Y),
    Face::Positive(Axis::Z),
];
const NEGATIVE: [Face; 3] = [
    Face::Negative(Axis::X),
    Face::Negative(Axis::Y),
    Face::Negative(Axis::Z),
];

struct Grid(Vec<MaterialId>);

impl Grid {
    fn new(voxels: &[((u8, u8, u8), MaterialId)]) -> Grid {
        let mut g = vec![EMPTY; CHUNK_VOLUME];
        for &((x, y, z), m) in voxels {
            g[Morton3D::from_xyz_max_depth(x, y, z).as_u32() as usize] = m;
        }
        Grid(g)
    }

    /// Material at (x, y, z). Outside the chunk is empty.
    fn at(&self, (x, y, z): (i32, i32, i32)) -> MaterialId {
        let side = PLANE_SIDE as i32;
        if x < 0 || y < 0 || z < 0 || x >= side || y >= side || z >= side {
            return EMPTY;
        }
        self.0[Morton3D::from_xyz_max_depth(x as u8, y as u8, z as u8).as_u32() as usize]
    }
}

/// The voxel at coordinate `c` along axis `a`, at plane position (u, v),
/// using projected_axis_uv's convention: X -> (y, z), Y -> (z, x), Z -> (x, y).
fn voxel(a: usize, c: i32, u: i32, v: i32) -> (i32, i32, i32) {
    match a {
        0 => (c, u, v),
        1 => (v, c, u),
        _ => (u, v, c),
    }
}

/// A 3D point as (plane coordinate along axis `a`, u, v).
fn project(a: usize, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
    match a {
        0 => (x, y, z),
        1 => (y, z, x),
        _ => (z, x, y),
    }
}

struct Fixture {
    grid: Grid,
    parity: [Vec<u64>; 3],
    claims: [Vec<u64>; 3],
}

fn words(n: usize) -> [Vec<u64>; 3] {
    core::array::from_fn(|_| vec![0u64; n])
}

fn parity_of(s: &mut [Vec<u64>; 3]) -> Parity<'_> {
    let [x, y, z] = s;
    Parity {
        masks: [
            ParityMask::from_u64_slice(x),
            ParityMask::from_u64_slice(y),
            ParityMask::from_u64_slice(z),
        ],
    }
}

fn claims_of(s: &mut [Vec<u64>; 3]) -> Claims<'_> {
    let [x, y, z] = s;
    Claims {
        masks: [
            ClaimsMask::from_u64_slice(x),
            ClaimsMask::from_u64_slice(y),
            ClaimsMask::from_u64_slice(z),
        ],
    }
}

fn fixture(voxels: &[((u8, u8, u8), MaterialId)]) -> Fixture {
    let mut f = Fixture {
        grid: Grid::new(voxels),
        parity: words(PARITY_WORDS),
        claims: words(PARITY_WORDS),
    };
    let mut parity = parity_of(&mut f.parity);
    for a in 0..3 {
        for c in 0..PLANE_SIDE as i32 {
            for u in 0..PLANE_SIDE as i32 {
                for v in 0..PLANE_SIDE as i32 {
                    let below = f.grid.at(voxel(a, c - 1, u, v));
                    let here = f.grid.at(voxel(a, c, u, v));
                    if below != here {
                        let uv = Coords2D {
                            u: u as u32,
                            v: v as u32,
                        };
                        parity.set_transition(AXES[a], c as usize, uv, 1);
                    }
                }
            }
        }
    }
    drop(parity);
    f
}

fn dense_walk<W: DenseWalker<MaterialId>>(
    walker: &mut W,
    grid: &Grid,
    blocks: &[((u8, u8, u8), u8)],
) {
    let starts: Vec<Morton3D> = blocks
        .iter()
        .map(|&((x, y, z), d)| {
            let side = (PLANE_SIDE >> d) as u8;
            assert!(
                x % side == 0 && y % side == 0 && z % side == 0,
                "block must be aligned"
            );
            Morton3D::from_xyz_and_depth(x, y, z, d.into())
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

/// Meshes the chunk once with a fresh sweep cache and the fixture's claims.
fn mesh(f: &mut Fixture, blocks: &[((u8, u8, u8), u8)]) -> Vec<Triangle> {
    let mut tris = Vec::new();
    let mut cache = words(MATERIAL_WORDS);
    {
        let [mx, my, mz] = &mut cache;
        let materials = Material {
            cells: [
                mx.view_bits_mut::<Lsb0>(),
                my.view_bits_mut::<Lsb0>(),
                mz.view_bits_mut::<Lsb0>(),
            ],
        };
        let shaper = SurfaceScanner::new(claims_of(&mut f.claims), parity_of(&mut f.parity), |t| {
            tris.push(t)
        });
        let mut to_material = |m: &MaterialId| *m;
        let mut walker = SurfaceWalker::new(materials, shaper, &mut to_material);
        dense_walk(&mut walker, &f.grid, blocks);
    }
    tris
}

fn assert_no_t_junctions_3d(tris: &[Triangle]) {
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

/// Checks the mesh against the grid alone. Every exposed face cell on
/// planes 0..32 is covered exactly once, carries the solid side's
/// material, and points away from the solid. Nothing else is covered.
/// Plane 32 belongs to the neighbouring chunk. Winding must be
/// consistent: the same sign relative to the face direction everywhere.
fn assert_mesh_matches(grid: &Grid, tris: &[Triangle]) {
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

#[test]
fn single_voxel_in_the_interior() {
    let mut f = fixture(&[((2, 3, 4), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 12);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn single_voxel_at_chunk_origin() {
    let mut f = fixture(&[((0, 0, 0), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 12);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
#[ignore = "Disabled: touching chunk boundary not yet implemented"]
fn single_voxel_at_far_corner_emits_only_near_faces() {
    let mut f = fixture(&[((31, 31, 31), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 6);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn same_material_domino_merges_and_skips_internal_face() {
    let mut f = fixture(&[((2, 3, 4), STONE), ((3, 3, 4), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 12);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn tunnel() {
    let mut voxels = Vec::new();
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                if (x, y) != (1, 1) {
                    voxels.push(((x, y, z), STONE));
                }
            }
        }
    }
    let mut f = fixture(&voxels);
    let tris = mesh(&mut f, &[]);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn l_prism_at_odd_offsets() {
    let mut voxels = Vec::new();
    for x in 2..5 {
        for y in 3..6 {
            for z in 2..5 {
                if !((y == 4 || y == 5) && (x == 3 || x == 4)) {
                    voxels.push(((x, y, z), STONE));
                }
            }
        }
    }
    let mut f = fixture(&voxels);
    let tris = mesh(&mut f, &[]);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn large_solid_leaf_with_neighbour() {
    let mut voxels = Vec::new();
    for x in 4..8 {
        for y in 4..8 {
            for z in 4..8 {
                voxels.push(((x, y, z), STONE));
            }
        }
    }
    voxels.push(((8, 5, 5), STONE));
    let mut f = fixture(&voxels);
    let tris = mesh(&mut f, &[((4, 4, 4), 3)]);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn large_empty_leaf_over_mixed_neighbours() {
    let mut voxels = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                if (x, y, z) != (1, 1, 1) {
                    voxels.push(((x, y, z), STONE));
                }
            }
        }
    }
    let mut f = fixture(&voxels);
    let tris = mesh(&mut f, &[((2, 0, 0), 4)]);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn rewalking_with_same_claims_emits_nothing() {
    let mut f = fixture(&[((2, 3, 4), STONE), ((3, 3, 4), STONE)]);
    assert_eq!(mesh(&mut f, &[]).len(), 12);
    assert_eq!(mesh(&mut f, &[]).len(), 0);
}

#[test]
fn edge_touching_voxels_are_seamless() {
    let mut f = fixture(&[((0, 0, 0), STONE), ((0, 0, 1), STONE), ((1, 1, 0), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn different_materials_split_and_skip_occluded_face() {
    let mut f = fixture(&[((2, 3, 4), STONE), ((3, 3, 4), DIRT)]);
    let tris = mesh(&mut f, &[]);
    let stone = tris.iter().filter(|t| t.material == STONE).count();
    let dirt = tris.iter().filter(|t| t.material == DIRT).count();
    assert_eq!((stone, dirt), (10, 10));
    assert_mesh_matches(&f.grid, &tris);
}

#[test]
fn material_boundary_corner_inside_neighbour_edge() {
    let mut f = fixture(&[((2, 3, 4), STONE), ((2, 3, 5), STONE), ((3, 3, 4), DIRT)]);
    let tris = mesh(&mut f, &[]);
    assert_mesh_matches(&f.grid, &tris);
    assert_no_t_junctions_3d(&tris);
}

/// Random blobs of up to three materials, meshed end to end: every face is
/// covered once with the right material and orientation, and there are no
/// T-junctions anywhere, including across creases. Kept away from the
/// chunk's far edges, which are the pinned multi-chunk cases.
#[test]
fn random_multi_material_blobs_are_seamless() {
    let mut state: u32 = 0x9E37_79B9;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    };

    for _ in 0..10 {
        let mut voxels = Vec::new();
        for x in 1..7u8 {
            for y in 1..7u8 {
                for z in 1..7u8 {
                    let r = next() % 8;
                    if r < 5 {
                        voxels.push(((x, y, z), 1 + (r % 3) as MaterialId));
                    }
                }
            }
        }
        let mut f = fixture(&voxels);
        let tris = mesh(&mut f, &[]);
        assert_mesh_matches(&f.grid, &tris);
        assert_no_t_junctions_3d(&tris);
    }
}
