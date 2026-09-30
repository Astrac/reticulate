use crate::{
    meshing::triangulation::{Triangle2D, stepper::INACTIVE_VERTEX, zipper::Zipper},
    utils::testing::{area2, assert_covers_exactly, weave_rows},
};

/// A zipper whose bottom vertices sit at the given columns, all at `height`.
fn zipper_at(columns: &[u32], height: u8) -> Zipper {
    let mut zipper = Zipper {
        bottom_column_heights: [INACTIVE_VERTEX; 32],
        bottom_columns_mask: 0,
    };
    for &c in columns {
        zipper.bottom_column_heights[c as usize] = height;
        zipper.bottom_columns_mask |= 1 << c;
    }
    zipper
}

fn mask(columns: &[u32]) -> u32 {
    columns.iter().fold(0, |m, &c| m | (1 << c))
}

#[test]
fn zip_joins_two_parallel_edges_with_two_triangles() {
    let mut zipper = zipper_at(&[1, 6], 0);
    let mut tris = Vec::new();

    zipper.zip(mask(&[1, 2, 3, 4, 5, 6]), 1, mask(&[1, 6]), &mut |t| {
        tris.push(t)
    });

    assert_eq!(
        tris,
        vec![
            Triangle2D {
                vertices: [(1, 1), (1, 0), (6, 0)]
            },
            Triangle2D {
                vertices: [(1, 1), (6, 0), (6, 1)]
            },
        ]
    );
}

#[test]
fn zip_leaves_the_last_bottom_vertex_active_for_the_next_strip() {
    let mut zipper = zipper_at(&[1, 6], 0);

    zipper.zip(mask(&[1, 2, 3, 4, 5, 6]), 1, mask(&[1, 6]), &mut |_| {});

    assert!(zipper.bottom_columns_mask & (1 << 6) != 0);
}

#[test]
fn zip_does_nothing_without_top_vertices_in_the_run() {
    let mut zipper = zipper_at(&[1, 6], 0);
    let mut tris = Vec::new();

    zipper.zip(mask(&[1, 2, 3, 4, 5, 6]), 1, mask(&[20]), &mut |t| {
        tris.push(t)
    });

    assert!(tris.is_empty());
    assert_eq!(zipper.bottom_columns_mask, mask(&[1, 6]));
}

#[test]
fn zip_does_nothing_without_bottom_vertices_in_the_run() {
    let mut zipper = zipper_at(&[20], 0);
    let mut tris = Vec::new();

    zipper.zip(mask(&[1, 2, 3, 4]), 1, mask(&[1, 4]), &mut |t| tris.push(t));

    assert!(tris.is_empty());
}

#[test]
fn zip_only_looks_at_vertices_inside_the_scan_mask() {
    let mut zipper = zipper_at(&[1, 6, 20], 0);
    let mut tris = Vec::new();

    zipper.zip(mask(&[1, 2, 3, 4, 5, 6]), 1, mask(&[1, 6, 20]), &mut |t| {
        tris.push(t)
    });

    assert_eq!(tris.len(), 2);
    assert!(zipper.bottom_columns_mask & (1 << 20) != 0, "untouched");
}

#[test]
fn zip_emits_counter_clockwise_triangles() {
    let mut zipper = zipper_at(&[1, 4], 2);
    let mut tris = Vec::new();

    zipper.zip(mask(&[1, 2, 3, 4]), 5, mask(&[1, 4]), &mut |t| tris.push(t));

    assert!(!tris.is_empty());
    assert!(tris.iter().all(|t| area2(t) > 0), "{tris:?}");
}

#[test]
fn advance_moves_the_given_columns_up_to_the_new_row() {
    let mut zipper = zipper_at(&[0, 1, 2, 3], 0);

    zipper.advance(3, mask(&[1, 2]), mask(&[2]));

    assert_eq!(zipper.bottom_column_heights[0], 0);
    assert_eq!(zipper.bottom_column_heights[1], 3);
    assert_eq!(zipper.bottom_column_heights[2], 3);
    assert_eq!(zipper.bottom_column_heights[3], 0);
}

#[test]
fn advance_keeps_only_moved_columns_that_are_still_scanned() {
    let mut zipper = zipper_at(&[0, 1, 2, 3], 0);

    zipper.advance(3, mask(&[1, 2]), mask(&[2]));

    // 1 is gone (moved but no longer scanned), 2 stays, 0 and 3 are untouched.
    assert_eq!(zipper.bottom_columns_mask, mask(&[0, 2, 3]));
}

/// A collinear bottom chain, (1,0) and (2,1), under an apex at (3,2). Without
/// the skip in the zipper this emits the zero-area triangle (3,2),(1,0),(2,1).
#[test]
fn collinear_bottom_vertex_is_skipped_not_connected() {
    let cells = [0b0010, 0b1110, 0b0110]; // .x.. / .xxx / .xx.
    let tris = weave_rows(&cells);
    assert_covers_exactly(&tris, &cells);
    assert!(
        tris.iter().all(|t| area2(t) != 0),
        "zero-area triangle: {tris:?}"
    );
}

/// A staircase growing to the left leaves a pending diagonal, (6,1)-(7,0), and
/// no top vertex to close it. Without the break this emits the zero-area
/// triangle (3,2),(4,1),(5,0).
#[test]
fn pending_diagonal_is_left_for_a_later_strip() {
    let cells = [0b100, 0b110, 0b111].map(|m: u32| m << 2); // ..x / .xx / xxx, moved right
    let tris = weave_rows(&cells);
    assert_covers_exactly(&tris, &cells);
    assert!(
        tris.iter().all(|t| area2(t) != 0),
        "zero-area triangle: {tris:?}"
    );
}
