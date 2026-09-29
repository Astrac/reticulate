use crate::{
    Coords2D,
    axis::*,
    meshing::{
        bitfields::{Claims, ClaimsMask, PLANE_SIDE},
        triangulation::stepper::Stepper,
    },
};

fn storage() -> [Vec<u64>; 3] {
    core::array::from_fn(|_| vec![0u64; PLANE_SIDE * PLANE_SIDE * PLANE_SIDE / 64])
}

fn build(s: &mut [Vec<u64>; 3]) -> Claims<'_> {
    let [x, y, z] = s;
    Claims {
        masks: [
            ClaimsMask::from_u64_slice(x),
            ClaimsMask::from_u64_slice(y),
            ClaimsMask::from_u64_slice(z),
        ],
    }
}

fn uv(u: u32, v: u32) -> Coords2D {
    Coords2D { u, v }
}

fn shape_with(cells: &[(usize, usize)]) -> [u32; PLANE_SIDE] {
    let mut s = [0u32; PLANE_SIDE];
    for &(u, v) in cells {
        s[u] |= 1 << v;
    }
    s
}

#[test]
fn fresh_storage_is_all_pending() {
    let mut s = storage();
    let c = build(&mut s);
    for axis in [AXIS_X, AXIS_Y, AXIS_Z] {
        for coord in 0..PLANE_SIDE {
            for u in 0..PLANE_SIDE as u32 {
                for v in 0..PLANE_SIDE as u32 {
                    assert!(c.is_pending(axis, coord, uv(u, v)));
                }
            }
        }
    }
}

/// Regression test for bug 2: an asymmetric cell must not read back transposed.
#[test]
fn is_pending_is_not_transposed() {
    let mut s = storage();
    let mut c = build(&mut s);
    c.mark_done(AXIS_Y, 5, &shape_with(&[(3, 17)]));
    assert!(!c.is_pending(AXIS_Y, 5, uv(3, 17)));
    assert!(c.is_pending(AXIS_Y, 5, uv(17, 3)));
}

/// Regression test for bug 1: a second strip must not erase the first.
#[test]
fn mark_done_accumulates() {
    let mut s = storage();
    let mut c = build(&mut s);
    c.mark_done(AXIS_X, 0, &shape_with(&[(1, 0), (1, 1)]));
    c.mark_done(AXIS_X, 0, &shape_with(&[(0, 5)]));
    assert!(!c.is_pending(AXIS_X, 0, uv(1, 0)));
    assert!(!c.is_pending(AXIS_X, 0, uv(1, 1)));
    assert!(!c.is_pending(AXIS_X, 0, uv(0, 5)));
}

#[test]
fn mark_done_isolated_to_plane_and_axis() {
    let mut s = storage();
    let mut c = build(&mut s);
    c.mark_done(AXIS_Y, 5, &shape_with(&[(2, 2)]));
    assert!(!c.is_pending(AXIS_Y, 5, uv(2, 2)));
    assert!(c.is_pending(AXIS_Y, 4, uv(2, 2)));
    assert!(c.is_pending(AXIS_Y, 6, uv(2, 2)));
    assert!(c.is_pending(AXIS_X, 5, uv(2, 2)));
    assert!(c.is_pending(AXIS_Z, 5, uv(2, 2)));
}

#[test]
fn claimed_row_matches_is_pending() {
    let mut s = storage();
    let mut c = build(&mut s);
    let shape = shape_with(&[(0, 0), (0, 31), (7, 3), (7, 4), (31, 31)]);
    c.mark_done(AXIS_Z, 9, &shape);
    for u in 0..PLANE_SIDE {
        assert_eq!(c.claimed_row(AXIS_Z, 9, u), shape[u]);
        for v in 0..PLANE_SIDE as u32 {
            let claimed = (shape[u] >> v) & 1 == 1;
            assert_eq!(c.is_pending(AXIS_Z, 9, uv(u as u32, v)), !claimed);
        }
    }
}

#[test]
fn last_plane_and_corner_cell() {
    let mut s = storage();
    let mut c = build(&mut s);
    c.mark_done(AXIS_X, 31, &shape_with(&[(31, 31)]));
    assert!(!c.is_pending(AXIS_X, 31, uv(31, 31)));
    assert!(c.is_pending(AXIS_X, 30, uv(31, 31)));
}

#[test]
fn round_trip_with_strip_weaver_filled() {
    let edge = (1 << 5) | (1 << 16);
    let mut stepper = Stepper::from_bottom_edge(edge, 7, 3);
    stepper.weave_to(4, 0, 0, |_| {}); // an empty row above closes row 3
    let mut s = storage();
    let mut c = build(&mut s);
    c.mark_done(AXIS_X, 2, &stepper.filled);
    for col in 0..PLANE_SIDE as u32 {
        let inside = (5..16).contains(&col);
        assert_eq!(c.is_pending(AXIS_X, 2, uv(3, col)), !inside, "col {col}");
    }
    assert!(
        c.is_pending(AXIS_X, 2, uv(10, 3)),
        "transposed cell must stay pending"
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "cells claimed twice")]
fn double_claim_panics_in_debug() {
    let mut s = storage();
    let mut c = build(&mut s);
    c.mark_done(AXIS_X, 0, &shape_with(&[(4, 4)]));
    c.mark_done(AXIS_X, 0, &shape_with(&[(4, 4)]));
}
