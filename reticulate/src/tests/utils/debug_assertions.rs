use crate::{
    Coords2D,
    axis::Axis,
    meshing::surface_layer::SurfaceLayer,
    utils::{
        debug_assertions::{debug_assert_within_footprint, find_unclaimed_faces},
        testing::{Fixture, cells, claims_of, parity_of},
    },
};

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "strip claimed non-face cells")]
fn footprint_assertion_catches_non_face_cells() {
    let mut f = Fixture::solid(&[(2, 5, 9)]);
    let parity = parity_of(&mut f.parity);
    debug_assert_within_footprint(
        &parity,
        SurfaceLayer::new(Axis::X, 2),
        &cells(&[(5, 9), (5, 10)]),
    );
}

#[test]
fn footprint_assertion_accepts_cells_inside_the_footprint() {
    let mut f = Fixture::solid(&[(2, 5, 9)]);
    let parity = parity_of(&mut f.parity);
    debug_assert_within_footprint(&parity, SurfaceLayer::new(Axis::X, 2), &cells(&[(5, 9)]));
    debug_assert_within_footprint(&parity, SurfaceLayer::new(Axis::X, 2), &cells(&[]));
}

#[test]
fn unclaimed_faces_of_a_voxel_are_its_six_faces() {
    let mut f = Fixture::solid(&[(2, 5, 9)]);
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);

    let mut missing = find_unclaimed_faces(&parity, &claims);
    missing.sort_by_key(|&(axis, layer, _)| (axis.as_usize(), layer));

    assert_eq!(
        missing,
        vec![
            (Axis::X, 2, Coords2D { u: 5, v: 9 }),
            (Axis::X, 3, Coords2D { u: 5, v: 9 }),
            (Axis::Y, 5, Coords2D { u: 9, v: 2 }),
            (Axis::Y, 6, Coords2D { u: 9, v: 2 }),
            (Axis::Z, 9, Coords2D { u: 2, v: 5 }),
            (Axis::Z, 10, Coords2D { u: 2, v: 5 }),
        ]
    );
}

#[test]
fn claimed_faces_are_no_longer_missing() {
    let mut f = Fixture::solid(&[(2, 5, 9)]);
    claims_of(&mut f.claims).claim(SurfaceLayer::new(Axis::X, 3), &cells(&[(5, 9)]));
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);

    let missing = find_unclaimed_faces(&parity, &claims);

    assert_eq!(missing.len(), 5);
    assert!(!missing.contains(&(Axis::X, 3, Coords2D { u: 5, v: 9 })));
}

#[test]
fn an_empty_chunk_has_no_faces_to_claim() {
    let mut f = Fixture::solid(&[]);
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);

    assert!(find_unclaimed_faces(&parity, &claims).is_empty());
}
