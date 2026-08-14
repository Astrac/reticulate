use crate::{axis::*, coords::Coords3D, morton_3d::Morton3D};

#[test]
fn test_roundtrip_zero() {
    let c = Coords3D { x: 0, y: 0, z: 0 };
    let m: Morton3D = c.into();
    let decoded: Coords3D = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for zero");
}

#[test]
fn test_roundtrip_one() {
    let c = Coords3D { x: 1, y: 1, z: 1 };
    let m: Morton3D = c.into();
    let decoded: Coords3D = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for one");
}

#[test]
fn test_roundtrip_max() {
    let c = Coords3D {
        x: 255,
        y: 255,
        z: 255,
    };
    let m: Morton3D = c.into();
    let decoded: Coords3D = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for max");
}

#[test]
fn test_roundtrip_mixed() {
    let c = Coords3D {
        x: 10,
        y: 20,
        z: 30,
    };
    let m: Morton3D = c.into();
    let decoded: Coords3D = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for mixed values");
}

#[test]
fn test_uniqueness() {
    // Different coordinates should produce different Morton codes
    let c1 = Coords3D { x: 1, y: 2, z: 3 };
    let c2 = Coords3D { x: 2, y: 1, z: 3 };
    let c3 = Coords3D { x: 3, y: 2, z: 1 };

    let m1: Morton3D = c1.into();
    let m2: Morton3D = c2.into();
    let m3: Morton3D = c3.into();

    assert_ne!(m1, m2, "Distinct coords c1 and c2 produced same code");
    assert_ne!(m2, m3, "Distinct coords c2 and c3 produced same code");
    assert_ne!(m1, m3, "Distinct coords c1 and c3 produced same code");
}

#[test]
fn test_boundary_values() {
    // Test specific boundary cases
    let cases = vec![
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
        let m: Morton3D = c.into();
        let decoded: Coords3D = m.into();
        assert_eq!(c, decoded, "Roundtrip failed for {:?}", c);
    }
}

#[test]
fn test_project_on_plane_axis_z() {
    let input_code = 0b101_011;
    let wrapper = Morton3D::from(input_code);
    let projected_2d = wrapper.drop_axis(AXIS_Z);
    assert_eq!(projected_2d, 0b01_11);
}

#[test]
fn test_project_on_plane_axis_z_deeper() {
    let input_code = 0b110_101_011;
    let wrapper = Morton3D::from(input_code);
    let projected_2d = wrapper.drop_axis(AXIS_Z);
    assert_eq!(projected_2d, 0b10_01_11);
}

#[test]
fn test_project_on_plane_axis_y() {
    let input_code = 0b010_101;
    let wrapper = Morton3D::from(input_code);
    let projected_2d = wrapper.drop_axis(AXIS_Y);
    assert_eq!(projected_2d, 0b00_11);
}

#[test]
fn test_project_on_plane_axis_y_deeper() {
    let input_code = 0b110_010_101;
    let wrapper = Morton3D::from(input_code);
    let projected_2d = wrapper.drop_axis(AXIS_Y);
    assert_eq!(projected_2d, 0b10_00_11);
}

#[test]
fn test_project_on_plane_axis_x() {
    let input_code = 0b101_110;
    let wrapper = Morton3D::from(input_code);
    let projected_2d = wrapper.drop_axis(AXIS_X);
    assert_eq!(projected_2d, 0b10_11);
}

#[test]
fn test_project_on_plane_axis_x_deeper() {
    let input_code = 0b110_101_110;
    let wrapper = Morton3D::from(input_code);
    let projected_2d = wrapper.drop_axis(AXIS_X);
    assert_eq!(projected_2d, 0b11_10_11);
}
