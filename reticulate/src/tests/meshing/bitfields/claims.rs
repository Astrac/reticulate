use crate::{
    Axis, ChunkCoord, Coords2D, PLANE_SIDE,
    meshing::{surface_layer::SurfaceLayer, triangulation::stepper::Stepper},
    utils::testing::{cells, claims_of, storage},
};

fn uv(u: u32, v: u32) -> Coords2D {
    Coords2D {
        u: u as ChunkCoord,
        v: v as ChunkCoord,
    }
}

#[test]
fn fresh_storage_is_all_pending() {
    let mut s = storage();
    let c = claims_of(&mut s);
    for axis in Axis::AXES {
        for coord in 0..PLANE_SIDE {
            for u in 0..PLANE_SIDE as u32 {
                for v in 0..PLANE_SIDE as u32 {
                    assert!(c.is_unclaimed(SurfaceLayer::new(axis, coord as ChunkCoord), uv(u, v)));
                }
            }
        }
    }
}

/// Regression test for bug 2: an asymmetric cell must not read back transposed.
#[test]
fn is_pending_is_not_transposed() {
    let mut s = storage();
    let mut c = claims_of(&mut s);
    c.claim(SurfaceLayer::new(Axis::Y, 5), &cells(&[(3, 17)]));
    assert!(!c.is_unclaimed(SurfaceLayer::new(Axis::Y, 5), uv(3, 17)));
    assert!(c.is_unclaimed(SurfaceLayer::new(Axis::Y, 5), uv(17, 3)));
}

/// Regression test for bug 1: a second strip must not erase the first.
#[test]
fn mark_done_accumulates() {
    let mut s = storage();
    let mut c = claims_of(&mut s);
    c.claim(SurfaceLayer::new(Axis::X, 0), &cells(&[(1, 0), (1, 1)]));
    c.claim(SurfaceLayer::new(Axis::X, 0), &cells(&[(0, 5)]));
    assert!(!c.is_unclaimed(SurfaceLayer::new(Axis::X, 0), uv(1, 0)));
    assert!(!c.is_unclaimed(SurfaceLayer::new(Axis::X, 0), uv(1, 1)));
    assert!(!c.is_unclaimed(SurfaceLayer::new(Axis::X, 0), uv(0, 5)));
}

#[test]
fn mark_done_isolated_to_plane_and_axis() {
    let mut s = storage();
    let mut c = claims_of(&mut s);
    c.claim(SurfaceLayer::new(Axis::Y, 5), &cells(&[(2, 2)]));
    assert!(!c.is_unclaimed(SurfaceLayer::new(Axis::Y, 5), uv(2, 2)));
    assert!(c.is_unclaimed(SurfaceLayer::new(Axis::Y, 4), uv(2, 2)));
    assert!(c.is_unclaimed(SurfaceLayer::new(Axis::Y, 6), uv(2, 2)));
    assert!(c.is_unclaimed(SurfaceLayer::new(Axis::X, 5), uv(2, 2)));
    assert!(c.is_unclaimed(SurfaceLayer::new(Axis::Z, 5), uv(2, 2)));
}

#[test]
fn claimed_row_matches_is_pending() {
    let mut s = storage();
    let mut c = claims_of(&mut s);
    let shape = cells(&[(0, 0), (0, 31), (7, 3), (7, 4), (31, 31)]);
    c.claim(SurfaceLayer::new(Axis::Z, 9), &shape);
    for u in 0..PLANE_SIDE {
        assert_eq!(c.claimed_row(SurfaceLayer::new(Axis::Z, 9), u), shape[u]);
        for v in 0..PLANE_SIDE as u32 {
            let claimed = (shape[u] >> v) & 1 == 1;
            assert_eq!(
                c.is_unclaimed(SurfaceLayer::new(Axis::Z, 9), uv(u as u32, v)),
                !claimed
            );
        }
    }
}

#[test]
fn last_plane_and_corner_cell() {
    let mut s = storage();
    let mut c = claims_of(&mut s);
    c.claim(SurfaceLayer::new(Axis::X, 31), &cells(&[(31, 31)]));
    assert!(!c.is_unclaimed(SurfaceLayer::new(Axis::X, 31), uv(31, 31)));
    assert!(c.is_unclaimed(SurfaceLayer::new(Axis::X, 30), uv(31, 31)));
}

#[test]
fn round_trip_with_strip_weaver_filled() {
    let edge = (1 << 5) | (1 << 16);
    let mut stepper = Stepper::from_live(edge, 1 << 7, 3);
    stepper.weave_to(4, 0, 0, |_| {}); // an empty row above closes row 3
    let mut s = storage();
    let mut c = claims_of(&mut s);
    c.claim(SurfaceLayer::new(Axis::X, 2), &stepper.filled);
    for col in 0..PLANE_SIDE as u32 {
        let inside = (5..16).contains(&col);
        assert_eq!(
            c.is_unclaimed(SurfaceLayer::new(Axis::X, 2), uv(3, col)),
            !inside,
            "col {col}"
        );
    }
    assert!(
        c.is_unclaimed(SurfaceLayer::new(Axis::X, 2), uv(10, 3)),
        "transposed cell must stay pending"
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "cells claimed twice")]
fn double_claim_panics_in_debug() {
    let mut s = storage();
    let mut c = claims_of(&mut s);
    c.claim(SurfaceLayer::new(Axis::X, 0), &cells(&[(4, 4)]));
    c.claim(SurfaceLayer::new(Axis::X, 0), &cells(&[(4, 4)]));
}
