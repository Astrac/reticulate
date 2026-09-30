use crate::{
    axis::Axis,
    meshing::{bitfields::PLANE_SIDE, surface_layer::SurfaceLayer},
    utils::testing::{Fixture, assert_scan, cells, claims_of, mesh::scan_plane},
};

/// A voxel at an asymmetric position. Each of its six faces must be
/// exactly one cell, at the projected (row, col), on both the negative
/// face plane (c) and the positive one (c + 1). The positive face only
/// works if the previous slab's walls are used.
#[test]
fn single_voxel_all_six_faces() {
    let mut f = Fixture::solid(&[(2, 5, 9)]);
    // (axis, voxel coordinate on that axis, (row, col) of its projection)
    for (axis, c, cell) in [
        (Axis::X, 2, (5, 9)),
        (Axis::Y, 5, (9, 2)),
        (Axis::Z, 9, (2, 5)),
    ] {
        for plane in [c, c + 1] {
            let (filled, tris) = scan_plane(&mut f, axis, plane, (cell.0 as u32, cell.1 as u32));
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
    let mut f = Fixture::solid(&[(3, 4, 5), (4, 4, 5)]);

    let (filled, tris) = scan_plane(&mut f, Axis::Y, 4, (5, 3)); // rows z, cols x
    assert_scan(&filled, &tris, &[(5, 3), (5, 4)]);

    let (filled, tris) = scan_plane(&mut f, Axis::Z, 5, (3, 4)); // rows x, cols y
    assert_scan(&filled, &tris, &[(3, 4), (4, 4)]);

    let (filled, tris) = scan_plane(&mut f, Axis::X, 3, (4, 5)); // rows y, cols z
    assert_scan(&filled, &tris, &[(4, 5)]);

    let (filled, tris) = scan_plane(&mut f, Axis::X, 5, (4, 5)); // positive X face
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
    let mut f = Fixture::solid(&solid);

    let (filled, tris) = scan_plane(&mut f, Axis::X, 1, (1, 1));
    assert_scan(&filled, &tris, &[(1, 1)]);

    let (filled, tris) = scan_plane(&mut f, Axis::X, 0, (0, 0)); // outer face: full 2x2
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
    let mut f = Fixture::solid(&solid);
    claims_of(&mut f.claims).claim(SurfaceLayer::new(Axis::Y, 0), &cells(&[(1, 3)]));

    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (0, 1)); // rows z, cols x
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
    let mut f = Fixture::solid(&solid);

    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (28, 5)); // rows z, cols x
    assert_scan(&filled, &tris, &[(28, 5), (29, 5), (30, 5), (31, 5)]);
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
    let mut f = Fixture::solid(&voxels);

    let (filled, tris) = scan_plane(&mut f, Axis::Y, 1, (3, 1)); // rows z, cols x
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
    let mut f = Fixture::solid(&solid);

    let (filled, tris) = scan_plane(&mut f, Axis::X, 0, (1, 0)); // rows y, cols z
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
