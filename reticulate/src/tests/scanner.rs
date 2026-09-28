use crate::{
    Coords2D,
    axis::*,
    meshing::{
        bitfields::{Claims, ClaimsMask, PLANE_SIDE, Parity, ParityMask},
        triangulation::{Triangle2D, scanner::Scanner},
    },
    tests::assert_covers_exactly,
};

const WORDS: usize = PLANE_SIDE * PLANE_SIDE * PLANE_SIDE / 64;

/// Backing storage for one chunk's parity and claims.
struct Fixture {
    parity: [Vec<u64>; 3],
    claims: [Vec<u64>; 3],
}

fn storage() -> [Vec<u64>; 3] {
    core::array::from_fn(|_| vec![0u64; WORDS])
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

/// Builds parity from a set of solid voxels, with the same rules as
/// pass 1: a transition along axis A at coordinate c wherever the voxel
/// at c differs from the one at c-1. Outside the chunk counts as empty.
/// (u, v) follow projected_axis_uv: X -> (y, z), Y -> (z, x), Z -> (x, y).
fn fixture(solid: &[(usize, usize, usize)]) -> Fixture {
    let mut grid = vec![false; PLANE_SIDE * PLANE_SIDE * PLANE_SIDE];
    let idx = |x: usize, y: usize, z: usize| (x * PLANE_SIDE + y) * PLANE_SIDE + z;
    for &(x, y, z) in solid {
        grid[idx(x, y, z)] = true;
    }
    let at = |x: isize, y: isize, z: isize| {
        x >= 0 && y >= 0 && z >= 0 && grid[idx(x as usize, y as usize, z as usize)]
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
                    parity.set_transition(AXIS_X, x, uv(y, z), 1);
                }
                if here != at(xi, yi - 1, zi) {
                    parity.set_transition(AXIS_Y, y, uv(z, x), 1);
                }
                if here != at(xi, yi, zi - 1) {
                    parity.set_transition(AXIS_Z, z, uv(x, y), 1);
                }
            }
        }
    }
    drop(parity);
    f
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
    let scanner = Scanner::new(&parity, axis, axis_coord);
    let mut tris = Vec::new();
    let filled = scanner.scan_seed(
        &claims,
        Coords2D {
            u: seed.0,
            v: seed.1,
        },
        |t| tris.push(t),
    );
    scanner.debug_assert_within_footprint(&filled);
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
        (AXIS_X, 2, (5, 9)),
        (AXIS_Y, 5, (9, 2)),
        (AXIS_Z, 9, (2, 5)),
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

    let (filled, tris) = scan(&mut f, AXIS_Y, 4, (5, 3)); // rows z, cols x
    assert_scan(&filled, &tris, &[(5, 3), (5, 4)]);

    let (filled, tris) = scan(&mut f, AXIS_Z, 5, (3, 4)); // rows x, cols y
    assert_scan(&filled, &tris, &[(3, 4), (4, 4)]);

    let (filled, tris) = scan(&mut f, AXIS_X, 3, (4, 5)); // rows y, cols z
    assert_scan(&filled, &tris, &[(4, 5)]);

    let (filled, tris) = scan(&mut f, AXIS_X, 5, (4, 5)); // positive X face
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

    let (filled, tris) = scan(&mut f, AXIS_X, 1, (1, 1));
    assert_scan(&filled, &tris, &[(1, 1)]);

    let (filled, tris) = scan(&mut f, AXIS_X, 0, (0, 0)); // outer face: full 2x2
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
    claims_of(&mut f.claims).mark_done(AXIS_Y, 0, &cells(&[(1, 3)]));

    let (filled, tris) = scan(&mut f, AXIS_Y, 0, (0, 1)); // rows z, cols x
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

    let (filled, tris) = scan(&mut f, AXIS_Y, 0, (28, 5)); // rows z, cols x
    assert_scan(&filled, &tris, &[(28, 5), (29, 5), (30, 5), (31, 5)]);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "strip claimed non-face cells")]
fn footprint_assertion_catches_non_face_cells() {
    let mut f = fixture(&[(2, 5, 9)]);
    let parity = parity_of(&mut f.parity);
    let scanner = Scanner::new(&parity, AXIS_X, 2);
    scanner.debug_assert_within_footprint(&cells(&[(5, 9), (5, 10)]));
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

    let (filled, tris) = scan(&mut f, AXIS_Y, 1, (3, 1)); // rows z, cols x
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

    let (filled, tris) = scan(&mut f, AXIS_X, 0, (1, 0)); // rows y, cols z
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
    let (filled, tris) = scan(&mut f, AXIS_Y, 0, (3, 5));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn downward_pass_starts_with_two_runs() {
    let mut shape: Vec<_> = (1..=6).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 2), (r, 5), (r, 6)]);
    }
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, AXIS_Y, 0, (3, 3));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn downward_pass_goes_around_a_hole() {
    let mut shape = Vec::new();
    for r in 0..4 {
        for c in 1..=4 {
            if (r, c) != (1, 2) {
                shape.push((r, c));
            }
        }
    }
    let mut f = plane_fixture(&shape);
    let (filled, tris) = scan(&mut f, AXIS_Y, 0, (3, 1));
    assert_scan(&filled, &tris, &shape);
}

#[test]
fn downward_pass_respects_claims() {
    let mut shape = Vec::new();
    for r in 0..3 {
        for c in 1..=3 {
            shape.push((r, c));
        }
    }
    let mut f = plane_fixture(&shape);
    claims_of(&mut f.claims).mark_done(AXIS_Y, 0, &cells(&[(1, 3), (0, 3)]));
    let (filled, tris) = scan(&mut f, AXIS_Y, 0, (2, 1));
    assert_scan(
        &filled,
        &tris,
        &[(2, 1), (2, 2), (2, 3), (1, 1), (1, 2), (0, 1), (0, 2)],
    );
}
