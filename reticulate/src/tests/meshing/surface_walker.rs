use crate::{
    MaterialId,
    utils::testing::{DIRT, Fixture, STONE, SeededRandom, assertions, mesh::mesh},
};

#[test]
fn single_voxel_in_the_interior() {
    let mut f = Fixture::new(&[((2, 3, 4), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 12);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

#[test]
fn single_voxel_at_chunk_origin() {
    let mut f = Fixture::new(&[((0, 0, 0), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 12);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

#[test]
fn single_voxel_at_far_corner_emits_only_near_faces() {
    let mut f = Fixture::new(&[((31, 31, 31), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 6);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

#[test]
fn same_material_domino_merges_and_skips_internal_face() {
    let mut f = Fixture::new(&[((2, 3, 4), STONE), ((3, 3, 4), STONE)]);
    let tris = mesh(&mut f, &[]);
    assert_eq!(tris.len(), 12);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
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
    let mut f = Fixture::new(&voxels);
    let tris = mesh(&mut f, &[]);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
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
    let mut f = Fixture::new(&voxels);
    let tris = mesh(&mut f, &[]);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
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
    let mut f = Fixture::new(&voxels);
    let tris = mesh(&mut f, &[((4, 4, 4), 3)]);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
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
    let mut f = Fixture::new(&voxels);
    let tris = mesh(&mut f, &[((2, 0, 0), 4)]);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

#[test]
fn rewalking_with_same_claims_emits_nothing() {
    let mut f = Fixture::new(&[((2, 3, 4), STONE), ((3, 3, 4), STONE)]);
    assert_eq!(mesh(&mut f, &[]).len(), 12);
    assert_eq!(mesh(&mut f, &[]).len(), 0);
}

#[test]
fn edge_touching_voxels_are_seamless() {
    let mut f = Fixture::new(&[((0, 0, 0), STONE), ((0, 0, 1), STONE), ((1, 1, 0), STONE)]);
    let tris = mesh(&mut f, &[]);
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

#[test]
fn different_materials_split_and_skip_occluded_face() {
    let mut f = Fixture::new(&[((2, 3, 4), STONE), ((3, 3, 4), DIRT)]);
    let tris = mesh(&mut f, &[]);
    let stone = tris.iter().filter(|t| t.material() == STONE).count();
    let dirt = tris.iter().filter(|t| t.material() == DIRT).count();

    assert_eq!((stone, dirt), (10, 10));
    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

#[test]
fn material_boundary_corner_inside_neighbour_edge() {
    let mut f = Fixture::new(&[((2, 3, 4), STONE), ((2, 3, 5), STONE), ((3, 3, 4), DIRT)]);
    let tris = mesh(&mut f, &[]);

    assertions::assert_canonical_mesh(&f.grid, &tris, false);
}

/// Random blobs of up to three materials, meshed end to end: every face is
/// covered once with the right material and orientation, and there are no
/// T-junctions anywhere, including across creases. Kept away from the
/// chunk's far edges, which are the pinned multi-chunk cases.
#[test]
fn random_multi_material_blobs_are_seamless() {
    let seed = 699069;
    let mut rng = SeededRandom::new(seed);

    let mut voxels = Vec::new();
    for x in 1..4 {
        for y in 1..4 {
            for z in 1..4 {
                let r = rng.next(8);
                if r < 3 {
                    voxels.push(((x, y, z), 1 + (r % 3) as MaterialId));
                }
            }
        }
    }
    let mut f = Fixture::new(&voxels);
    let tris = mesh(&mut f, &[]);

    assertions::assert_canonical_mesh(&f.grid, &tris, true);
}
