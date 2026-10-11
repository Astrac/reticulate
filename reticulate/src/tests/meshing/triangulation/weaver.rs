use crate::{
    Axis, Coords2D,
    meshing::{
        surface_layer::SurfaceLayer,
        triangulation::{edges::Edges, filler::Filler, weaver::Weaver},
    },
    utils::testing::{
        DIRT, Fixture, STONE, assertions, cells, claims_of, mesh::scan_plane, parity_of, weave_rows,
    },
};

#[test]
fn column_seeded_at_its_top() {
    let shape: Vec<_> = (0..4).map(|r| (r, 5)).collect();
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (3, 5));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
}

#[test]
fn seed_above_two_runs() {
    let mut shape: Vec<_> = (1..=6).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 2), (r, 5), (r, 6)]);
    }
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (3, 3));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
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
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (3, 1));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
}

#[test]
fn inverted_u_is_covered() {
    let mut shape: Vec<_> = (1..=5).map(|c| (3, c)).collect();
    for r in 0..3 {
        shape.extend([(r, 1), (r, 5)]);
    }
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (0, 1));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
}

#[test]
fn dip_below_start_is_covered() {
    let mut shape = vec![(2, 1), (2, 2), (2, 4), (3, 4)];
    shape.extend((1..=4).map(|c| (1, c)));
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (2, 1));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
}

#[test]
fn pinch_start_below_end_above() {
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 1), (2, 2)];
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (0, 1));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
}

#[test]
fn pinch_end_below_start_above() {
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 2), (2, 3)];
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (0, 1));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
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
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (0, 1));
    assertions::assert_canonical_weave(&filled, &shape, &tris);
}

#[test]
fn weaver_matches_stepper_reference_weave() {
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 1), (2, 2)];
    let mut f = Fixture::plane(&shape);
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);
    let weaver = Weaver::new(&parity);
    let seed = Coords2D { u: 0, v: 1 };
    let layer = SurfaceLayer::new(Axis::Y, 0);
    let edges = Edges::from_parity(&parity, layer);
    let filler = Filler::new(&claims, &parity);

    let region = filler.fill_region(layer, &edges, seed);
    let reference = weave_rows(&region);

    let mut tris = Vec::new();
    weaver.weave_region(layer, &edges, &region, |t| tris.push(t));
    assert_eq!(tris, reference);
}

/// Two materials on the plane under y = 0 (rows z, cols x): a stone column A
/// at x = 2, z = 4..5, and a dirt cell B at x = 3, z = 4, beside A's lower
/// half. The material wall makes them separate regions. B's corner at
/// (3, 5) lies on A's edge from (3, 4) to (3, 6), so A must have a vertex
/// there too.
#[test]
fn adjacent_regions_share_boundary_vertices() {
    let mut f = Fixture::new(&[((2, 0, 4), STONE), ((2, 0, 5), STONE), ((3, 0, 4), DIRT)]);

    let (a_filled, a_tris) = scan_plane(&mut f, Axis::Y, 0, (4, 2));
    assert_eq!(a_filled, cells(&[(4, 2), (5, 2)]), "stone region");
    claims_of(&mut f.claims).claim(SurfaceLayer::new(Axis::Y, 0), &a_filled);

    let (b_filled, b_tris) = scan_plane(&mut f, Axis::Y, 0, (4, 3));
    assert_eq!(b_filled, cells(&[(4, 3)]), "dirt region");

    let all: Vec<_> = a_tris.into_iter().chain(b_tris).collect();
    assertions::assert_covers_exactly(&all, &cells(&[(4, 2), (5, 2), (4, 3)]));
    assertions::assert_no_t_junctions_2d(&all);
}

#[test]
fn empty_region_emits_nothing() {
    let mut f = Fixture::plane(&[(1, 1)]);
    let parity = parity_of(&mut f.parity);
    let weaver = Weaver::new(&parity);
    let layer = SurfaceLayer::new(Axis::Y, 0);
    let edges = Edges::from_parity(&parity, layer);

    let mut tris = Vec::new();
    weaver.weave_region(layer, &edges, &[0; 32], |t| tris.push(t));

    assert!(tris.is_empty());
}

#[test]
fn single_cell_is_a_quad_of_two_triangles() {
    let mut f = Fixture::plane(&[(4, 6)]);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (4, 6));

    assertions::assert_canonical_weave(&filled, &[(4, 6)], &tris);
    assert_eq!(tris.len(), 2);
}

#[test]
fn rectangle_needs_no_interior_vertices() {
    let shape: Vec<_> = (2..6).flat_map(|r| (3..9).map(move |c| (r, c))).collect();
    let mut f = Fixture::plane(&shape);
    let (filled, tris) = scan_plane(&mut f, Axis::Y, 0, (2, 3));

    assertions::assert_canonical_weave(&filled, &shape, &tris);
    assert_eq!(tris.len(), 2, "a rectangle is a single quad");
}

#[test]
fn weaving_is_the_same_on_every_axis() {
    // The same 2D shape extruded along each axis must give the same strip
    // triangles, since only the parity conventions differ.
    let shape = [(0, 1), (0, 2), (0, 3), (1, 1), (1, 3), (2, 1), (2, 2)];
    let reference = {
        let mut f = Fixture::plane(&shape);
        scan_plane(&mut f, Axis::Y, 0, (0, 1)).1
    };

    // X plane: rows y, columns z. Z plane: rows x, columns y.
    let mut on_x = Fixture::solid(&shape.map(|(r, c)| (0, r, c)));
    let mut on_z = Fixture::solid(&shape.map(|(r, c)| (r, c, 0)));
    assert_eq!(scan_plane(&mut on_x, Axis::X, 0, (0, 1)).1, reference);
    assert_eq!(scan_plane(&mut on_z, Axis::Z, 0, (0, 1)).1, reference);
}
