use crate::{BlockFaces, Coords, Octree, block_faces::*, tests::init_logging};

pub const ALL_FACES: u8 = 0b00111111;

fn has_face(face: &BlockFaces, mask: u8) -> bool {
    (face.face_mask & mask) != 0
}

#[test]
fn test_empty_octree_returns_empty_vec() {
    let octree = Octree::new();
    let faces = BlockFaces::from_octree(&octree, 0);
    assert!(faces.is_empty(), "Empty octree should yield no faces");
}

#[test]
fn test_single_block_exposes_all_faces() {
    let mut octree = Octree::new();
    octree.update(
        Coords {
            x: 50,
            y: 50,
            z: 50,
        },
        Some(()),
    );

    let faces = BlockFaces::from_octree(&octree, 0);
    assert_eq!(faces.len(), 1);

    let face = &faces[0];
    assert_eq!(
        face.coords,
        Coords {
            x: 50,
            y: 50,
            z: 50
        }
    );
    assert_eq!(
        face.face_mask, ALL_FACES,
        "Isolated block must expose all 6 faces"
    );
}

#[test]
fn test_adjacent_blocks_hide_shared_face_x_axis() {
    let mut octree = Octree::new();
    octree.update(
        Coords {
            x: 10,
            y: 10,
            z: 10,
        },
        Some(()),
    );
    octree.update(
        Coords {
            x: 11,
            y: 10,
            z: 10,
        },
        Some(()),
    );

    let faces = BlockFaces::from_octree(&octree, 0);
    assert_eq!(faces.len(), 2);

    let left = faces.iter().find(|f| f.coords.x == 10).unwrap();
    let right = faces.iter().find(|f| f.coords.x == 11).unwrap();

    assert!(
        !has_face(left, FACE_RIGHT),
        "Left block should not expose +X (RIGHT)"
    );
    assert!(
        !has_face(right, FACE_LEFT),
        "Right block should not expose -X (LEFT)"
    );

    assert_eq!(left.face_mask, ALL_FACES ^ FACE_RIGHT);
    assert_eq!(right.face_mask, ALL_FACES ^ FACE_LEFT);
}

#[test]
fn test_adjacent_blocks_hide_shared_face_y_axis() {
    let mut octree = Octree::new();
    octree.update(
        Coords {
            x: 10,
            y: 10,
            z: 10,
        },
        Some(()),
    );
    octree.update(
        Coords {
            x: 10,
            y: 11,
            z: 10,
        },
        Some(()),
    );

    let faces = BlockFaces::from_octree(&octree, 0);

    let bottom = faces.iter().find(|f| f.coords.y == 10).unwrap();
    let top = faces.iter().find(|f| f.coords.y == 11).unwrap();

    assert!(
        !has_face(bottom, FACE_TOP),
        "Bottom block should not expose +Y (TOP)"
    );
    assert!(
        !has_face(top, FACE_BOTTOM),
        "Top block should not expose -Y (BOTTOM)"
    );
}

#[test]
fn test_adjacent_blocks_hide_shared_face_z_axis() {
    let mut octree = Octree::new();
    octree.update(
        Coords {
            x: 10,
            y: 10,
            z: 10,
        },
        Some(()),
    );
    octree.update(
        Coords {
            x: 10,
            y: 10,
            z: 11,
        },
        Some(()),
    );

    let faces = BlockFaces::from_octree(&octree, 0);

    let back = faces.iter().find(|f| f.coords.z == 10).unwrap();
    let front = faces.iter().find(|f| f.coords.z == 11).unwrap();

    assert!(
        !has_face(back, FACE_FRONT),
        "Back block should not expose +Z (FRONT)"
    );
    assert!(
        !has_face(front, FACE_BACK),
        "Front block should not expose -Z (BACK)"
    );
}

#[test]
fn test_boundary_blocks_expose_outer_faces() {
    let mut octree = Octree::new();
    octree.update(Coords { x: 0, y: 0, z: 0 }, Some(()));

    let faces = BlockFaces::from_octree(&octree, 0);
    let face = &faces[0];

    // Assumes coordinates < 0 are treated as empty space
    assert!(has_face(face, FACE_LEFT), "Min X should expose LEFT face");
    assert!(
        has_face(face, FACE_BOTTOM),
        "Min Y should expose BOTTOM face"
    );
    assert!(has_face(face, FACE_BACK), "Min Z should expose BACK face");
}

#[test]
fn test_removing_block_updates_faces() {
    let mut octree = Octree::new();
    octree.update(
        Coords {
            x: 10,
            y: 10,
            z: 10,
        },
        Some(()),
    );
    octree.update(
        Coords {
            x: 11,
            y: 10,
            z: 10,
        },
        Some(()),
    );

    let faces_before = BlockFaces::from_octree(&octree, 0);
    assert_eq!(faces_before.len(), 2);

    octree.update(
        Coords {
            x: 11,
            y: 10,
            z: 10,
        },
        None,
    );

    let faces_after = BlockFaces::from_octree(&octree, 0);
    assert_eq!(faces_after.len(), 1);

    let remaining = &faces_after[0];
    assert_eq!(remaining.coords.x, 10);
    assert!(
        has_face(remaining, FACE_RIGHT),
        "Removing neighbor should re-expose the shared face"
    );
}

#[test]
fn test_hollow_chunk_exposes_inner_and_outer_surfaces() {
    let mut octree: Octree<()> = Octree::new();

    // Build a 3x3x3 hollow shell from (10,10,10) to (12,12,12)
    // The center at (11,11,11) remains empty.
    for x in 10..=12 {
        for y in 10..=12 {
            for z in 10..=12 {
                let is_shell = x == 10 || x == 12 || y == 10 || y == 12 || z == 10 || z == 12;
                if is_shell {
                    octree.update(Coords { x, y, z }, Some(()));
                }
            }
        }
    }

    let faces = BlockFaces::from_octree(&octree, 0);

    // A 3x3x3 cube has 27 positions. Minus 1 center = 26 blocks.
    assert_eq!(
        faces.len(),
        26,
        "Should return exactly one face entry per block"
    );

    // 1. Verify inner faces face the empty center (11,11,11)
    let inner_left = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 10,
                    y: 11,
                    z: 11,
                }
        })
        .unwrap();
    assert!(
        has_face(inner_left, FACE_RIGHT),
        "Left wall should expose inner +X face"
    );

    let inner_right = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 12,
                    y: 11,
                    z: 11,
                }
        })
        .unwrap();
    assert!(
        has_face(inner_right, FACE_LEFT),
        "Right wall should expose inner -X face"
    );

    let inner_bottom = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 11,
                    y: 10,
                    z: 11,
                }
        })
        .unwrap();
    assert!(
        has_face(inner_bottom, FACE_TOP),
        "Bottom wall should expose inner +Y face"
    );

    let inner_top = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 11,
                    y: 12,
                    z: 11,
                }
        })
        .unwrap();
    assert!(
        has_face(inner_top, FACE_BOTTOM),
        "Top wall should expose inner -Y face"
    );

    let inner_back = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 11,
                    y: 11,
                    z: 10,
                }
        })
        .unwrap();
    assert!(
        has_face(inner_back, FACE_FRONT),
        "Back wall should expose inner +Z face"
    );

    let inner_front = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 11,
                    y: 11,
                    z: 12,
                }
        })
        .unwrap();
    assert!(
        has_face(inner_front, FACE_BACK),
        "Front wall should expose inner -Z face"
    );

    // 2. Verify outer corner exposes only exterior faces
    let corner = faces
        .iter()
        .find(|f| {
            f.coords
                == Coords {
                    x: 10,
                    y: 10,
                    z: 10,
                }
        })
        .unwrap();
    assert!(
        has_face(corner, FACE_LEFT),
        "Corner should expose outer -X face"
    );
    assert!(
        has_face(corner, FACE_BOTTOM),
        "Corner should expose outer -Y face"
    );
    assert!(
        has_face(corner, FACE_BACK),
        "Corner should expose outer -Z face"
    );

    // 3. Verify shared faces between shell blocks are correctly hidden
    assert!(
        !has_face(corner, FACE_RIGHT),
        "Corner should not expose +X (shared with neighbor)"
    );
    assert!(
        !has_face(corner, FACE_TOP),
        "Corner should not expose +Y (shared with neighbor)"
    );
    assert!(
        !has_face(corner, FACE_FRONT),
        "Corner should not expose +Z (shared with neighbor)"
    );
}

#[test]
fn test_from_octree_3x3x3_cube() {
    init_logging();
    let mut octree = Octree::<()>::new();

    // Populate a solid 3x3x3 block starting at (0, 0, 0)
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                octree.update(Coords { x, y, z }, Some(()));
            }
        }
    }

    // Generate faces (assuming root node index is 0)
    let root_index = 0;
    let mut faces = BlockFaces::from_octree(&octree, root_index);

    // Sort for deterministic testing order
    faces.sort_by_key(|f| (f.coords.x, f.coords.y, f.coords.z));

    // A solid 3x3x3 cube contains 27 blocks, but only the 26 surface blocks expose faces.
    assert_eq!(faces.len(), 26, "Expected exactly 26 surface blocks");

    // Helper closure to safely retrieve the face mask for a given coordinate
    let get_mask = |x, y, z| {
        faces
            .iter()
            .find(|f| f.coords.x == x && f.coords.y == y && f.coords.z == z)
            .map(|f| f.face_mask)
            .unwrap_or(0)
    };

    // --- Corners (3 exposed faces each) ---
    assert_eq!(
        get_mask(0, 0, 0),
        FACE_LEFT | FACE_BOTTOM | FACE_BACK,
        "Min corner"
    );
    assert_eq!(
        get_mask(2, 0, 0),
        FACE_RIGHT | FACE_BOTTOM | FACE_BACK,
        "Max X, Min Y/Z corner"
    );
    assert_eq!(
        get_mask(0, 2, 0),
        FACE_LEFT | FACE_TOP | FACE_BACK,
        "Min X/Z, Max Y corner"
    );
    assert_eq!(
        get_mask(2, 2, 2),
        FACE_RIGHT | FACE_TOP | FACE_FRONT,
        "Max corner"
    );

    // --- Edges (2 exposed faces each) ---
    assert_eq!(get_mask(0, 0, 1), FACE_LEFT | FACE_BOTTOM, "X/Y edge");
    assert_eq!(get_mask(0, 1, 0), FACE_LEFT | FACE_BACK, "X/Z edge");
    assert_eq!(
        get_mask(2, 1, 2),
        FACE_RIGHT | FACE_FRONT,
        "Max X/Z edge"
    );

    // --- Face Centers (1 exposed face each) ---
    assert_eq!(get_mask(0, 1, 1), FACE_LEFT, "Left face center");
    assert_eq!(get_mask(2, 1, 1), FACE_RIGHT, "Right face center");
    assert_eq!(get_mask(1, 0, 1), FACE_BOTTOM, "Bottom face center");
    assert_eq!(get_mask(1, 2, 1), FACE_TOP, "Top face center");
    assert_eq!(get_mask(1, 1, 0), FACE_BACK, "Back face center");
    assert_eq!(get_mask(1, 1, 2), FACE_FRONT, "Front face center");

    // --- Internal blocks should NOT appear in the result ---
    assert!(
        !faces
            .iter()
            .any(|f| f.coords.x == 1 && f.coords.y == 1 && f.coords.z == 1),
        "Internal blocks should be culled and not generate faces"
    );
}

#[test]
fn test_from_octree_2x2x2_cube() {
    init_logging();
    let mut octree = Octree::<()>::new();
    let start_x = 0;
    let start_y = 0;
    let start_z = 0;

    for x in start_x..(start_x + 2) {
        for y in start_y..(start_y + 2) {
            for z in start_z..(start_z + 2) {
                octree.update(Coords { x, y, z }, Some(()));
            }
        }
    }

    let faces = BlockFaces::from_octree(&octree, 0);

    // A solid 2x2x2 cube contains 8 blocks, but only the 12 faces are exposed.
    assert_eq!(faces.len(), 8, "Expected exactly 8 surface blocks");

    // Helper closure to safely retrieve the face mask for a given coordinate
    let get_mask = |x, y, z| {
        faces
            .iter()
            .find(|f| f.coords.x == x && f.coords.y == y && f.coords.z == z)
            .map(|f| f.face_mask)
            .unwrap_or(0)
    };

    assert_eq!(
        get_mask(start_x, start_y, start_z),
        FACE_LEFT | FACE_BOTTOM | FACE_BACK,
        "(0,0,0)"
    );
    assert_eq!(
        get_mask(start_x, start_y, start_z + 1),
        FACE_LEFT | FACE_BOTTOM | FACE_FRONT,
        "(0,0,1)"
    );
    assert_eq!(
        get_mask(start_x, start_y + 1, start_z),
        FACE_LEFT | FACE_TOP | FACE_BACK,
        "(0,1,0)"
    );
    assert_eq!(
        get_mask(start_x, start_y + 1, start_z + 1),
        FACE_LEFT | FACE_TOP | FACE_FRONT,
        "(0,1,1)"
    );
    assert_eq!(
        get_mask(start_x + 1, start_y, start_z),
        FACE_RIGHT | FACE_BOTTOM | FACE_BACK,
        "(1,0,0)"
    );
    assert_eq!(
        get_mask(start_x + 1, start_y, start_z + 1),
        FACE_RIGHT | FACE_BOTTOM | FACE_FRONT,
        "(1,0,1)"
    );
    assert_eq!(
        get_mask(start_x + 1, start_y + 1, start_z),
        FACE_RIGHT | FACE_TOP | FACE_BACK,
        "(1,1,0)"
    );
    assert_eq!(
        get_mask(start_x + 1, start_y + 1, start_z + 1),
        FACE_RIGHT | FACE_TOP | FACE_FRONT,
        "(1,1,1)"
    );
}
