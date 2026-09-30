use crate::{
    Coords2D,
    axis::Axis,
    meshing::{bitfields::PLANE_SIDE, surface_layer::SurfaceLayer},
    utils::testing::{parity_of, storage},
};

#[test]
fn get_layer_returns_correct_slice() {
    let mut s = storage();
    {
        let mut p = parity_of(&mut s);
        p.set_transition(SurfaceLayer::new(Axis::X, 5), Coords2D { u: 3, v: 7 }, 1);
    }
    let p = parity_of(&mut s);
    let layer = p.masks[Axis::X.as_usize()].get_layer(3);
    assert_eq!(layer.len(), PLANE_SIDE);
    assert_eq!(layer[7], 1u32 << 5);
    for (i, word) in layer.iter().enumerate() {
        if i != 7 {
            assert_eq!(*word, 0, "unexpected bit set at slice index {i}");
        }
    }
}

#[test]
fn gather_edges_matches_get_layer_directly() {
    let mut s = storage();
    {
        let mut p = parity_of(&mut s);
        p.set_transition(SurfaceLayer::new(Axis::Y, 2), Coords2D { u: 10, v: 20 }, 1);
    }
    let p = parity_of(&mut s);
    let via_gather = p.gather_edges(SurfaceLayer::new(Axis::Y, 10));
    let via_direct = p.masks[Axis::Y.as_usize()].get_layer(10);
    assert_eq!(via_gather, via_direct);
    assert_eq!(via_gather[20], 1u32 << 2);
}

#[test]
fn set_transition_area_size_one_sets_single_bit() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Z, 9), Coords2D { u: 4, v: 4 }, 1);

    let layer = p.masks[Axis::Z.as_usize()].get_layer(4);
    assert_eq!(layer[4], 1u32 << 9);
    for (i, word) in layer.iter().enumerate() {
        if i != 4 {
            assert_eq!(*word, 0);
        }
    }
}

#[test]
fn set_transition_area_size_four_writes_2x2_block() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::X, 3), Coords2D { u: 5, v: 5 }, 4);

    for u in 5..7 {
        let layer = p.masks[Axis::X.as_usize()].get_layer(u);
        for v in 5..7 {
            assert_eq!(layer[v], 1u32 << 3, "missing bit at u={u}, v={v}");
        }
        assert_eq!(layer[4], 0);
        assert_eq!(layer[7], 0);
    }
    assert_eq!(p.masks[Axis::X.as_usize()].get_layer(4)[5], 0);
    assert_eq!(p.masks[Axis::X.as_usize()].get_layer(7)[5], 0);
}

#[test]
fn set_transition_area_size_sixteen_writes_4x4_block() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Y, 1), Coords2D { u: 0, v: 0 }, 16);

    for u in 0..4 {
        let layer = p.masks[Axis::Y.as_usize()].get_layer(u);
        for v in 0..4 {
            assert_eq!(layer[v], 1u32 << 1);
        }
        assert_eq!(layer[4], 0, "block must not overrun past side=4");
    }
}

#[test]
fn set_transition_area_size_one_side_clamped_to_at_least_one() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Z, 0), Coords2D { u: 15, v: 15 }, 1);
    assert_eq!(p.masks[Axis::Z.as_usize()].get_layer(15)[15], 1u32 << 0);
}

#[test]
fn set_transition_ors_rather_than_overwrites() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::X, 2), Coords2D { u: 1, v: 1 }, 1);
    p.set_transition(SurfaceLayer::new(Axis::X, 5), Coords2D { u: 1, v: 1 }, 1);
    let word = p.masks[Axis::X.as_usize()].get_layer(1)[1];
    assert_eq!(
        word,
        (1u32 << 2) | (1u32 << 5),
        "second write must not clobber the first"
    );
}

#[test]
fn set_transition_only_writes_the_targeted_axis() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::X, 4), Coords2D { u: 8, v: 8 }, 1);

    assert_eq!(p.masks[Axis::X.as_usize()].get_layer(8)[8], 1u32 << 4);
    assert_eq!(
        p.masks[Axis::Y.as_usize()].get_layer(8)[8],
        0,
        "Y mask must be untouched"
    );
    assert_eq!(
        p.masks[Axis::Z.as_usize()].get_layer(8)[8],
        0,
        "Z mask must be untouched"
    );
}

#[test]
fn gather_guard_reconstructs_bit_across_multiple_layers() {
    let mut s = storage();
    {
        let mut p = parity_of(&mut s);
        // Write bit `u`=6 at (v=3), for several axis_coord "layer" values.
        p.set_transition(SurfaceLayer::new(Axis::X, 6), Coords2D { u: 2, v: 3 }, 1);
        p.set_transition(SurfaceLayer::new(Axis::X, 6), Coords2D { u: 5, v: 3 }, 1);
        p.set_transition(SurfaceLayer::new(Axis::X, 6), Coords2D { u: 9, v: 3 }, 1);
    }
    let p = parity_of(&mut s);
    let guard = p.gather_guard(Axis::X, /*v=*/ 3, /*u=*/ 6, 0, PLANE_SIDE);
    let expected = (1u32 << 2) | (1u32 << 5) | (1u32 << 9);
    assert_eq!(guard, expected);
}

#[test]
fn gather_guard_ignores_bits_outside_the_extracted_position() {
    let mut s = storage();
    {
        let mut p = parity_of(&mut s);
        // Same (u=layer, v) coordinates as the previous test, but the
        // set bit is `u`=7, not 6 — must NOT show up when we ask for 6.
        p.set_transition(SurfaceLayer::new(Axis::X, 7), Coords2D { u: 2, v: 3 }, 1);
    }
    let p = parity_of(&mut s);
    let guard = p.gather_guard(Axis::X, 3, 6, 0, PLANE_SIDE);
    assert_eq!(guard, 0);
}

#[test]
fn gather_guard_respects_slice_index_v() {
    let mut s = storage();
    {
        let mut p = parity_of(&mut s);
        // Bit 6 set at v=4, but we'll query v=3 — must not appear.
        p.set_transition(SurfaceLayer::new(Axis::X, 6), Coords2D { u: 2, v: 4 }, 1);
    }
    let p = parity_of(&mut s);
    let guard = p.gather_guard(Axis::X, 3, 6, 0, PLANE_SIDE);
    assert_eq!(guard, 0);
}

#[test]
fn gather_guard_range_excludes_layers_outside_start_end() {
    let mut s = storage();
    {
        let mut p = parity_of(&mut s);
        p.set_transition(SurfaceLayer::new(Axis::Z, 1), Coords2D { u: 2, v: 0 }, 1); // inside [0,5)
        p.set_transition(SurfaceLayer::new(Axis::Z, 1), Coords2D { u: 10, v: 0 }, 1); // outside [0,5)
    }
    let p = parity_of(&mut s);
    let guard = p.gather_guard(Axis::Z, 0, 1, 0, 5);
    assert_eq!(guard, 1u32 << 2, "layer=10 must be excluded by the range");
}

#[test]
fn gather_guard_empty_range_returns_zero() {
    let mut s = storage();
    let p = parity_of(&mut s);
    assert_eq!(p.gather_guard(Axis::Y, 0, 0, 5, 5), 0);
}

#[test]
fn mask_x_convention_layer_is_y_index_is_z_bit_is_x() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    let (y_coord, z_coord, x_coord) = (11, 13, 17);
    p.set_transition(
        SurfaceLayer::new(Axis::X, x_coord),
        Coords2D {
            u: y_coord,
            v: z_coord,
        },
        1,
    );
    assert_eq!(
        p.masks[Axis::X.as_usize()].get_layer(y_coord as usize)[z_coord as usize],
        1u32 << x_coord
    );
}

#[test]
fn mask_y_convention_layer_is_z_index_is_x_bit_is_y() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    let (z_coord, x_coord, y_coord) = (4, 8, 12);
    p.set_transition(
        SurfaceLayer::new(Axis::Y, y_coord),
        Coords2D {
            u: z_coord,
            v: x_coord,
        },
        1,
    );
    assert_eq!(
        p.masks[Axis::Y.as_usize()].get_layer(z_coord as usize)[x_coord as usize],
        1u32 << y_coord
    );
}

#[test]
fn mask_z_convention_layer_is_x_index_is_y_bit_is_z() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    let (x_coord, y_coord, z_coord) = (6, 14, 2);
    p.set_transition(
        SurfaceLayer::new(Axis::Z, z_coord),
        Coords2D {
            u: x_coord,
            v: y_coord,
        },
        1,
    );
    assert_eq!(
        p.masks[Axis::Z.as_usize()].get_layer(x_coord as usize)[y_coord as usize],
        1u32 << z_coord
    );
}

/// Z transitions at (x, y, z) are stored in plane x, row y, bit z. A Y plane's
/// columns are x and its row boundaries are z, so the guard of Y plane y at
/// row boundary z has bit x set wherever such a transition exists.
#[test]
fn boundary_guard_reports_columns_crossed_by_a_transition() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Z, 9), Coords2D { u: 5, v: 7 }, 1);

    let plane = SurfaceLayer::new(Axis::Y, 7);
    assert_eq!(p.boundary_guard(plane, 9, !0), 1 << 5);
    assert_eq!(p.boundary_guard(plane, 10, !0), 0, "other boundary");
    assert_eq!(p.boundary_guard(SurfaceLayer::new(Axis::Y, 6), 9, !0), 0);
}

#[test]
fn boundary_guard_also_looks_at_the_previous_plane() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Z, 9), Coords2D { u: 5, v: 7 }, 1);

    assert_eq!(
        p.boundary_guard(SurfaceLayer::new(Axis::Y, 8), 9, !0),
        1 << 5
    );
}

#[test]
fn boundary_guard_is_limited_to_the_mask() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Z, 9), Coords2D { u: 5, v: 7 }, 1);
    p.set_transition(SurfaceLayer::new(Axis::Z, 9), Coords2D { u: 12, v: 7 }, 1);

    let plane = SurfaceLayer::new(Axis::Y, 7);
    assert_eq!(p.boundary_guard(plane, 9, 1 << 5), 1 << 5);
    assert_eq!(p.boundary_guard(plane, 9, 1 << 12), 1 << 12);
    assert_eq!(p.boundary_guard(plane, 9, (1 << 5) | (1 << 12)), (1 << 5) | (1 << 12));
    assert_eq!(p.boundary_guard(plane, 9, 1 << 6), 0);
}

#[test]
fn boundary_guard_on_the_first_plane_has_no_previous_plane() {
    let mut s = storage();
    let mut p = parity_of(&mut s);
    p.set_transition(SurfaceLayer::new(Axis::Z, 9), Coords2D { u: 5, v: 0 }, 1);

    assert_eq!(
        p.boundary_guard(SurfaceLayer::new(Axis::Y, 0), 9, !0),
        1 << 5
    );
}
