use crate::{
    Coords2D,
    axis::Axis,
    meshing::{
        bitfields::{Claims, ClaimsMask, MaterialId, PARITY_WORDS, PLANE_SIDE, Parity, ParityMask},
        surface_layer::SurfaceLayer,
        triangulation::{Triangle2D, discoverer::Discoverer, stepper::Stepper, weaver::Weaver},
    },
    tests::assert_covers_exactly,
    utils::{bitmasks, debug_assertions::debug_assert_within_footprint},
};

const EMPTY: MaterialId = 0;
const STONE: MaterialId = 1;
const DIRT: MaterialId = 2;

/// Backing storage for one chunk's parity and claims.
struct Fixture {
    parity: [Vec<u64>; 3],
    claims: [Vec<u64>; 3],
}

fn storage() -> [Vec<u64>; 3] {
    core::array::from_fn(|_| vec![0u64; PARITY_WORDS])
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

/// Builds parity the way pass 1 does: a transition along axis A at
/// coordinate c wherever the material at c differs from the one at c - 1,
/// including between two different solids. Outside the chunk counts as
/// empty. (u, v) follow projected_axis_uv: X -> (y, z), Y -> (z, x), Z -> (x, y).
fn fixture_with_materials(voxels: &[((usize, usize, usize), MaterialId)]) -> Fixture {
    let mut grid = vec![EMPTY; PLANE_SIDE * PLANE_SIDE * PLANE_SIDE];
    let idx = |x: usize, y: usize, z: usize| (x * PLANE_SIDE + y) * PLANE_SIDE + z;
    for &((x, y, z), material) in voxels {
        grid[idx(x, y, z)] = material;
    }
    let at = |x: isize, y: isize, z: isize| {
        if x < 0 || y < 0 || z < 0 {
            EMPTY
        } else {
            grid[idx(x as usize, y as usize, z as usize)]
        }
    };

    let mut f = Fixture {
        parity: storage(),
        claims: storage(),
    };
    let mut parity = parity_of(&mut f.parity);
    for x in 0..PLANE_SIDE {
        for y in 0..PLANE_SIDE {
            for z in 0..PLANE_SIDE {
                let (xi, yi, zi) = (x as isize, y as isize, z as isize);
                let here = at(xi, yi, zi);
                let uv = |u: usize, v: usize| Coords2D {
                    u: u as u32,
                    v: v as u32,
                };
                if here != at(xi - 1, yi, zi) {
                    parity.set_transition(Axis::X, x, uv(y, z), 1);
                }
                if here != at(xi, yi - 1, zi) {
                    parity.set_transition(Axis::Y, y, uv(z, x), 1);
                }
                if here != at(xi, yi, zi - 1) {
                    parity.set_transition(Axis::Z, z, uv(x, y), 1);
                }
            }
        }
    }
    drop(parity);
    f
}

/// Single-material shapes: every listed voxel is stone.
fn fixture(solid: &[(usize, usize, usize)]) -> Fixture {
    let voxels: Vec<_> = solid.iter().map(|&p| (p, STONE)).collect();
    fixture_with_materials(&voxels)
}

/// Scans one face plane from `seed` = (row, col). Returns the claimed
/// cells and the emitted triangles.
fn scan(
    f: &mut Fixture,
    axis: Axis,
    axis_coord: usize,
    seed: (u32, u32),
) -> ([u32; PLANE_SIDE], Vec<Triangle2D>) {
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);
    let scanner = Weaver::new(Discoverer::new(
        &parity,
        SurfaceLayer::new(axis, axis_coord),
    ));
    let mut tris = Vec::new();
    let filled = scanner.from_seed(
        &claims,
        SurfaceLayer::new(axis, axis_coord),
        Coords2D {
            u: seed.0,
            v: seed.1,
        },
        |t| tris.push(t),
    );
    debug_assert_within_footprint(&parity, axis, axis_coord, &filled);
    (filled, tris)
}

/// Builds a `filled`-shaped mask from (row, col) cells.
fn cells(list: &[(usize, usize)]) -> [u32; PLANE_SIDE] {
    let mut m = [0u32; PLANE_SIDE];
    for &(row, col) in list {
        m[row] |= 1 << col;
    }
    m
}

fn assert_scan(filled: &[u32; PLANE_SIDE], tris: &[Triangle2D], expected: &[(usize, usize)]) {
    assert_eq!(filled, &cells(expected), "claimed cells differ");
    assert_covers_exactly(tris, filled);
    for t in tris {
        let [(c0, r0), (c1, r1), (c2, r2)] = t.vertices.map(|(c, r)| (c as i32, r as i32));
        let cross = (c1 - c0) * (r2 - r0) - (r1 - r0) * (c2 - c0);
        assert!(cross > 0, "non-canonical winding: {t:?}");
    }
    assert_no_t_junctions(tris);
}

/// Fails if a vertex of one triangle lies strictly inside an edge of another.
fn assert_no_t_junctions(tris: &[Triangle2D]) {
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

/// A voxel at an asymmetric position. Each of its six faces must be
/// exactly one cell, at the projected (row, col), on both the negative
/// face plane (c) and the positive one (c + 1). The positive face only
/// works if the previous slab's walls are used.
#[test]
fn single_voxel_all_six_faces() {
    let mut f = fixture(&[(2, 5, 9)]);
    // (axis, voxel coordinate on that axis, (row, col) of its projection)
    for (axis, c, cell) in [
        (Axis::X, 2, (5, 9)),
        (Axis::Y, 5, (9, 2)),
        (Axis::Z, 9, (2, 5)),
    ] {
        for plane in [c, c + 1] {
            let (filled, tris) = scan(&mut f, axis, plane, (cell.0 as u32, cell.1 as u32));
            assert_scan(&filled, &tris, &[cell]);
            assert_eq!(
                tris.len(),
                2,
                "{axis:?}@{plane}: a unit quad is two triangles"
            );
        }
    }
}

/// A 2-voxel domino along X. Faces perpendicular to Y and Z are 2x1,
/// so a u/v transposition shows up as 1x2.
#[test]
fn domino_faces_are_not_transposed() {
    let mut f = fixture(&[(3, 4, 5), (4, 4, 5)]);

    let (filled, tris) = scan(&mut f, Axis::Y, 4, (5, 3)); // rows z, cols x
    assert_scan(&filled, &tris, &[(5, 3), (5, 4)]);

    let (filled, tris) = scan(&mut f, Axis::Z, 5, (3, 4)); // rows x, cols y
    assert_scan(&filled, &tris, &[(3, 4), (4, 4)]);

    let (filled, tris) = scan(&mut f, Axis::X, 3, (4, 5)); // rows y, cols z
    assert_scan(&filled, &tris, &[(4, 5)]);

    let (filled, tris) = scan(&mut f, Axis::X, 5, (4, 5)); // positive X face
    assert_scan(&filled, &tris, &[(4, 5)]);
}

/// A 2x2x2 cube missing (1,1,1). On plane x = 1, both slabs are solid
/// everywhere except the missing corner, so the face is that one cell.
/// Reading either slab alone would give a larger region.
#[test]
fn face_exists_only_where_the_two_slabs_differ() {
    let mut solid = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                if (x, y, z) != (1, 1, 1) {
                    solid.push((x, y, z));
                }
            }
        }
    }
    let mut f = fixture(&solid);

    let (filled, tris) = scan(&mut f, Axis::X, 1, (1, 1));
    assert_scan(&filled, &tris, &[(1, 1)]);

    let (filled, tris) = scan(&mut f, Axis::X, 0, (0, 0)); // outer face: full 2x2
    assert_scan(&filled, &tris, &[(0, 0), (0, 1), (1, 0), (1, 1)]);
}

/// A 3x3 face with one cell already claimed. The scan must go around it,
/// never claiming it, and still reach the cells beyond it.
#[test]
fn claimed_cells_are_skipped() {
    let mut solid = Vec::new();
    for x in 1..=3 {
        for z in 0..=2 {
            solid.push((x, 0, z));
        }
    }
    let mut f = fixture(&solid);
    claims_of(&mut f.claims).claim(SurfaceLayer::new(Axis::Y, 0), &cells(&[(1, 3)]));

    let (filled, tris) = scan(&mut f, Axis::Y, 0, (0, 1)); // rows z, cols x
    assert_scan(
        &filled,
        &tris,
        &[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 1),
            (1, 2),
            (2, 1),
            (2, 2),
            (2, 3),
        ],
    );
}

#[test]
fn face_touching_chunk_top_closes_cleanly() {
    let solid: Vec<_> = (28..PLANE_SIDE).map(|z| (5, 0, z)).collect();
    let mut f = fixture(&solid);

    let (filled, tris) = scan(&mut f, Axis::Y, 0, (28, 5)); // rows z, cols x
    assert_scan(&filled, &tris, &[(28, 5), (29, 5), (30, 5), (31, 5)]);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "strip claimed non-face cells")]
fn footprint_assertion_catches_non_face_cells() {
    let mut f = fixture(&[(2, 5, 9)]);
    let parity = parity_of(&mut f.parity);
    debug_assert_within_footprint(&parity, Axis::X, 2, &cells(&[(5, 9), (5, 10)]));
}

#[test]
fn unconnected_shapes_below_the_seed_are_not_claimed() {
    let mut voxels = Vec::new();
    for x in 1..=3 {
        voxels.push((x, 0, 3)); // P faces on plane y = 1, row 3
        voxels.push((x, 1, 2)); // N faces on plane y = 1, row 2
    }
    for z in 0..=2 {
        voxels.push((5, 0, z)); // separate P-face column, rows 0..2
    }
    let mut f = fixture(&voxels);

    let (filled, tris) = scan(&mut f, Axis::Y, 1, (3, 1)); // rows z, cols x
    assert_scan(&filled, &tris, &[(3, 1), (3, 2), (3, 3)]);
}

#[test]
fn valley_below_seed_is_claimed() {
    let mut solid = Vec::new();
    for y in 0..3 {
        for z in 0..3 {
            if (y, z) != (0, 0) {
                solid.push((0, y, z));
            }
        }
    }
    let mut f = fixture(&solid);

    let (filled, tris) = scan(&mut f, Axis::X, 0, (1, 0)); // rows y, cols z
    assert_scan(
        &filled,
        &tris,
        &[
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 2),
            (2, 0),
            (2, 1),
            (2, 2),
        ],
    );
}

fn plane_fixture(cells_rc: &[(usize, usize)]) -> Fixture {
    let voxels: Vec<_> = cells_rc.iter().map(|&(r, c)| (c, 0, r)).collect();
    fixture(&voxels)
}

#[test]
fn column_seeded_at_its_top() {
    let shape: Vec<_> = (0..4).map(|r| (r, 5)).collect();
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (3, 5));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn seed_above_two_runs() {
    let mut shape: Vec<_> = (1..=6).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 2), (r, 5), (r, 6)]);
    }
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (3, 3));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn seed_above_hole() {
    let mut shape = Vec::new();
    for r in 0..4 {
        for c in 1..=4 {
            if (r, c) != (1, 2) {
                shape.push((r, c));
            }
        }
    }
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (3, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn inverted_u_is_covered() {
    let mut shape: Vec<_> = (1..=5).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 5)]);
    }
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (0, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn dip_below_start_is_covered() {
    let mut shape = vec![(2, 1), (2, 2), (2, 4), (3, 4)];
    shape.extend((1..=4).map(|c| (1, c)));
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (2, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn pinch_start_below_end_above() {
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 1), (2, 2)];
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (0, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn pinch_end_below_start_above() {
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 2), (2, 3)];
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (0, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn pinch_between_stacked_runs() {
    let shape = [
        (0, 1),
        (0, 2),
        (0, 3),
        (0, 4),
        (1, 1),
        (1, 3),
        (1, 4),
        (2, 1),
        (2, 2),
    ];
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, Axis::Y, 0, (0, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn scanner_weave_matches_reference_weave() {
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 1), (2, 2)];
    let mut f = plane_fixture(&shape);
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);
    let scanner = Weaver::new(Discoverer::new(&parity, SurfaceLayer::new(Axis::Y, 0)));
    let seed = Coords2D { u: 0, v: 1 };

    let region = scanner
        .discoverer
        .discover(SurfaceLayer::new(Axis::Y, 0), &claims, seed);
    let reference = weave(&region); // the helper the stepper tests use

    let mut tris = Vec::new();
    scanner.from_seed(&claims, SurfaceLayer::new(Axis::Y, 0), seed, |t| {
        tris.push(t)
    });
    assert_eq!(tris, reference);
}

/// Two materials on the plane under y = 0 (rows z, cols x): a stone column A
/// at x = 2, z = 4..5, and a dirt cell B at x = 3, z = 4, beside A's lower
/// half. The material wall makes them separate regions. B's corner at
/// (3, 5) lies on A's edge from (3, 4) to (3, 6), so A must have a vertex
/// there too.
#[test]
fn adjacent_regions_share_boundary_vertices() {
    let mut f =
        fixture_with_materials(&[((2, 0, 4), STONE), ((2, 0, 5), STONE), ((3, 0, 4), DIRT)]);

    let (a_filled, a_tris) = scan(&mut f, Axis::Y, 0, (4, 2));
    assert_eq!(a_filled, cells(&[(4, 2), (5, 2)]), "stone region");
    claims_of(&mut f.claims).claim(SurfaceLayer::new(Axis::Y, 0), &a_filled);

    let (b_filled, b_tris) = scan(&mut f, Axis::Y, 0, (4, 3));
    assert_eq!(b_filled, cells(&[(4, 3)]), "dirt region");

    let all: Vec<_> = a_tris.into_iter().chain(b_tris).collect();
    assert_covers_exactly(&all, &cells(&[(4, 2), (5, 2), (4, 3)]));
    assert_no_t_junctions(&all);
}

// TODO - Remove duplication with stepper tests
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
        stepper.weave_to(row, top, 0, |t| out.push(t));
    }
    out
}
