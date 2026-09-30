use crate::{
    Coords2D, Coords3D, Depth, Morton2D, Morton3D,
    axis::{AXES, Axis},
};

#[test]
fn uniqueness() {
    // Different coordinates should produce different Morton codes
    let c1 = Coords3D { x: 1, y: 2, z: 3 };
    let c2 = Coords3D { x: 2, y: 1, z: 3 };
    let c3 = Coords3D { x: 3, y: 2, z: 1 };

    let m1: Morton3D = Morton3D::from_coords_max_depth(c1);
    let m2: Morton3D = Morton3D::from_coords_max_depth(c2);
    let m3: Morton3D = Morton3D::from_coords_max_depth(c3);

    assert_ne!(m1, m2, "Distinct coords c1 and c2 produced same code");
    assert_ne!(m2, m3, "Distinct coords c2 and c3 produced same code");
    assert_ne!(m1, m3, "Distinct coords c1 and c3 produced same code");
}

#[test]
fn drop_axis_on_z() {
    let input_code = 0b101_011;
    let wrapper = Morton3D::from_code_max_depth(input_code);
    let projected_2d = wrapper.drop_axis(Axis::Z);
    assert_eq!(projected_2d.as_u32(), 0b01_11);
}

#[test]
fn drop_axis_on_z_deeper() {
    let input_code = 0b110_101_011;
    let wrapper = Morton3D::from_code_max_depth(input_code);
    let projected_2d = wrapper.drop_axis(Axis::Z);
    assert_eq!(projected_2d.as_u32(), 0b10_01_11);
}

#[test]
fn drop_axis_on_y() {
    let input_code = 0b010_101;
    let wrapper = Morton3D::from_code_max_depth(input_code);
    let projected_2d = wrapper.drop_axis(Axis::Y);
    assert_eq!(projected_2d.as_u32(), 0b00_11);
}

#[test]
fn drop_axis_on_y_deeper() {
    let input_code = 0b110_010_101;
    let wrapper = Morton3D::from_code_max_depth(input_code);
    let projected_2d = wrapper.drop_axis(Axis::Y);
    assert_eq!(projected_2d.as_u32(), 0b01_00_11);
}

#[test]
fn drop_axis_on_x() {
    let input_code = 0b101_110;
    let wrapper = Morton3D::from_code_max_depth(input_code);
    let projected_2d = wrapper.drop_axis(Axis::X);
    assert_eq!(projected_2d.as_u32(), 0b10_11);
}

#[test]
fn drop_axis_on_x_deeper() {
    let input_code = 0b110_101_110;
    let wrapper = Morton3D::from_code_max_depth(input_code);
    let projected_2d = wrapper.drop_axis(Axis::X);
    assert_eq!(projected_2d.as_u32(), 0b11_10_11);
}

#[test]
fn from_xyz_matches_from_coords() {
    let (x, y, z) = (9, 21, 30);
    let coords = Coords3D { x, y, z };
    assert_eq!(
        Morton3D::from_xyz_max_depth(x, y, z),
        Morton3D::from_coords_max_depth(coords)
    );
    assert_eq!(
        Morton3D::from_xyz_and_depth(x, y, z, Depth(3)),
        Morton3D::from_coords_and_depth(coords, Depth(3))
    );
}

#[test]
fn constructors_record_the_depth() {
    assert_eq!(Morton3D::from_xyz_max_depth(1, 2, 3).depth(), Depth::MAX);
    assert_eq!(Morton3D::from_xyz_and_depth(1, 2, 3, Depth(2)).depth(), Depth(2));
    assert_eq!(Morton3D::from_code_max_depth(5).depth(), Depth::MAX);
    assert_eq!(Morton3D::from_code(5, Depth(1)).depth(), Depth(1));
    assert_eq!(Morton3D::from_code(5, Depth(1)).as_u32(), 5);
}

#[test]
fn octant_is_the_lowest_triplet() {
    assert_eq!(Morton3D::from_code_max_depth(0b101_011_110).octant(), 0b110);
    assert_eq!(Morton3D::from_code(0, Depth(0)).octant(), 0);
}

#[test]
fn parent_drops_the_last_triplet_and_one_level() {
    let node = Morton3D::from_code(0b101_011_110, Depth(3));
    assert_eq!(node.parent(), Morton3D::from_code(0b101_011, Depth(2)));
}

#[test]
fn child_octant_appends_a_triplet_and_one_level() {
    let node = Morton3D::from_code(0b101_011, Depth(2));
    assert_eq!(
        node.child_octant(0b110),
        Morton3D::from_code(0b101_011_110, Depth(3))
    );
}

#[test]
fn child_and_parent_are_inverse() {
    let node = Morton3D::from_code(0b011_010, Depth(2));
    for octant in 0..8 {
        let child = node.child_octant(octant);
        assert_eq!(child.octant(), octant);
        assert_eq!(child.parent(), node);
    }
}

#[test]
fn projected_axis_coords_follow_the_plane_conventions() {
    let point = Morton3D::from_xyz_max_depth(9, 21, 14);

    // X plane: u is y, v is z. Y plane: u is z, v is x. Z plane: u is x, v is y.
    assert_eq!(point.projected_axis_coords(Axis::X), (Coords2D { u: 21, v: 14 }, 9));
    assert_eq!(point.projected_axis_coords(Axis::Y), (Coords2D { u: 14, v: 9 }, 21));
    assert_eq!(point.projected_axis_coords(Axis::Z), (Coords2D { u: 9, v: 21 }, 14));
}

#[test]
fn projected_axis_morton_is_the_morton_of_the_projected_coords() {
    let point = Morton3D::from_xyz_max_depth(9, 21, 14);
    for axis in AXES {
        let (coords, layer) = point.projected_axis_coords(axis);
        let (morton, morton_layer) = point.projected_axis_morton(axis);
        assert_eq!(morton, Morton2D::from_uv_max_depth(coords.u, coords.v));
        assert_eq!(morton_layer, layer);
    }
}

#[test]
fn projection_scales_shallow_nodes_to_their_first_voxel() {
    // Depth 1, octant x=1, y=0, z=1: the node starts at voxel (16, 0, 16).
    let node = Morton3D::from_code(0b101, Depth(1));
    let (morton, layer) = node.projected_axis_morton(Axis::Y);
    assert_eq!(layer, 0);
    assert_eq!(morton.depth(), Depth(1));
    assert_eq!(morton.as_coords(), Coords2D { u: 16, v: 16 });
}

#[test]
fn drop_axis_keeps_the_node_depth() {
    let node = Morton3D::from_code(0b101_011, Depth(2));
    for axis in AXES {
        assert_eq!(node.drop_axis(axis).depth(), Depth(2));
    }
}
