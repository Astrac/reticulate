use crate::{Axis, Coords3D, Depth, Morton3D, PLANE_SIDE, meshing::surface_layer::SurfaceLayer};

#[test]
fn roundtrip_zero() {
    let c = Coords3D { x: 0, y: 0, z: 0 };
    let m = Morton3D::from_coords_max_depth(c);
    assert_eq!(c, Coords3D::from_morton(m), "Roundtrip failed for zero");
}

#[test]
fn roundtrip_one() {
    let c = Coords3D { x: 1, y: 1, z: 1 };
    let m = Morton3D::from_coords_max_depth(c);
    assert_eq!(c, Coords3D::from_morton(m), "Roundtrip failed for one");
}

#[test]
fn roundtrip_max() {
    let c = Coords3D {
        x: 255,
        y: 255,
        z: 255,
    };
    let m = Morton3D::from_coords_max_depth(c);
    assert_eq!(c, Coords3D::from_morton(m), "Roundtrip failed for max");
}

#[test]
fn roundtrip_mixed() {
    let c = Coords3D {
        x: 10,
        y: 20,
        z: 30,
    };
    let m = Morton3D::from_coords_max_depth(c);
    assert_eq!(
        c,
        Coords3D::from_morton(m),
        "Roundtrip failed for mixed values"
    );
}

#[test]
fn roundtrip_boundary_values() {
    let cases = [
        Coords3D { x: 0, y: 0, z: 1 },
        Coords3D { x: 0, y: 1, z: 0 },
        Coords3D { x: 1, y: 0, z: 0 },
        Coords3D {
            x: 127,
            y: 127,
            z: 127,
        },
        Coords3D { x: 255, y: 0, z: 0 },
    ];

    for c in cases {
        let m = Morton3D::from_coords_max_depth(c);
        assert_eq!(c, Coords3D::from_morton(m), "Roundtrip failed for {c:?}");
    }
}

#[test]
fn roundtrip_every_chunk_coordinate() {
    for x in 0..PLANE_SIDE as u8 {
        for y in (0..PLANE_SIDE as u8).step_by(3) {
            for z in (0..PLANE_SIDE as u8).step_by(5) {
                let c = Coords3D { x, y, z };
                let m = Morton3D::from_coords_max_depth(c);
                assert_eq!(c, Coords3D::from_morton(m));
            }
        }
    }
}

#[test]
fn x_is_the_lowest_bit_of_every_triplet() {
    assert_eq!(
        Morton3D::from_coords_max_depth(Coords3D { x: 1, y: 0, z: 0 }).as_u32(),
        0b001
    );
    assert_eq!(
        Morton3D::from_coords_max_depth(Coords3D { x: 0, y: 1, z: 0 }).as_u32(),
        0b010
    );
    assert_eq!(
        Morton3D::from_coords_max_depth(Coords3D { x: 0, y: 0, z: 1 }).as_u32(),
        0b100
    );
}

#[test]
fn from_morton_scales_shallow_nodes_to_their_first_voxel() {
    // Depth 1, octant x=1, y=0, z=1 covers the voxels starting at (16, 0, 16).
    let node = Morton3D::from_code(0b101, Depth(1));
    assert_eq!(Coords3D::from_morton(node), Coords3D { x: 16, y: 0, z: 16 });

    // The root covers everything, starting at the origin.
    assert_eq!(
        Coords3D::from_morton(Morton3D::from_code(0, Depth(0))),
        Coords3D::default()
    );
}

#[test]
fn from_morton_unscaled_ignores_the_depth() {
    let node = Morton3D::from_code(0b101, Depth(1));
    assert_eq!(
        Coords3D::from_morton_unscaled(node),
        Coords3D { x: 1, y: 0, z: 1 }
    );

    // Triplets from the top: 110, 101, 011. Bit 0 of each gives x = 0b011,
    // bit 1 gives y = 0b101 and bit 2 gives z = 0b110.
    let node = Morton3D::from_code(0b110_101_011, Depth(3));
    assert_eq!(
        Coords3D::from_morton_unscaled(node),
        Coords3D { x: 3, y: 5, z: 6 }
    );
}

/// The projection takes strip coordinates, i.e. `(column, row)`, which is
/// `(v, u)` in the plane conventions: X -> (u: y, v: z), Y -> (u: z, v: x),
/// Z -> (u: x, v: y).
#[test]
fn from_2d_projection_places_column_and_row_per_axis() {
    let (column, row) = (5, 3);
    assert_eq!(
        Coords3D::from_2d_projection(SurfaceLayer::new(Axis::X, 7), (column, row)),
        Coords3D { x: 7, y: 3, z: 5 }
    );
    assert_eq!(
        Coords3D::from_2d_projection(SurfaceLayer::new(Axis::Y, 7), (column, row)),
        Coords3D { x: 5, y: 7, z: 3 }
    );
    assert_eq!(
        Coords3D::from_2d_projection(SurfaceLayer::new(Axis::Z, 7), (column, row)),
        Coords3D { x: 3, y: 5, z: 7 }
    );
}

#[test]
fn from_2d_projection_inverts_projected_axis_coords() {
    for axis in Axis::AXES {
        let point = Coords3D { x: 9, y: 21, z: 14 };
        let (uv, layer) = Morton3D::from_coords_max_depth(point).projected_axis_coords(axis);
        let back = Coords3D::from_2d_projection(SurfaceLayer::new(axis, layer), (uv.v, uv.u));
        assert_eq!(back, point, "{axis:?}");
    }
}
