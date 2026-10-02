use crate::{
    Axis, Coords3D, Face,
    meshing::{Triangle, triangulation::Triangle2D},
};

fn cross(a: Coords3D, b: Coords3D, c: Coords3D) -> [i64; 3] {
    let d = |p: Coords3D, q: Coords3D| {
        [
            q.x as i64 - p.x as i64,
            q.y as i64 - p.y as i64,
            q.z as i64 - p.z as i64,
        ]
    };
    let (u, v) = (d(a, b), d(a, c));
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ]
}

fn strip() -> Triangle2D {
    // Counter-clockwise in (column, row).
    Triangle2D {
        vertices: [(2, 3), (5, 3), (2, 7)],
    }
}

#[test]
fn from_strip_triangle_keeps_face_and_material() {
    let t = Triangle::from_strip_triangle(Face::Positive(Axis::Y), 4, strip(), 9);
    assert_eq!(t.face, Face::Positive(Axis::Y));
    assert_eq!(t.material, 9);
}

#[test]
fn from_strip_triangle_places_every_vertex_on_the_layer() {
    for axis in Axis::AXES {
        for face in [Face::Positive(axis), Face::Negative(axis)] {
            let t = Triangle::from_strip_triangle(face, 11, strip(), 1);
            for v in t.vertices {
                let along = match axis {
                    Axis::X => v.x,
                    Axis::Y => v.y,
                    Axis::Z => v.z,
                };
                assert_eq!(along, 11, "{face:?}: {v:?}");
            }
        }
    }
}

#[test]
fn from_strip_triangle_maps_columns_and_rows_like_the_projection() {
    // Strip vertices are (column, row); rows are u and columns are v.
    let t = Triangle::from_strip_triangle(Face::Negative(Axis::X), 1, strip(), 1);
    assert_eq!(
        t.vertices,
        [
            Coords3D { x: 1, y: 3, z: 2 },
            Coords3D { x: 1, y: 3, z: 5 },
            Coords3D { x: 1, y: 7, z: 2 },
        ]
    );

    let t = Triangle::from_strip_triangle(Face::Negative(Axis::Y), 1, strip(), 1);
    assert_eq!(
        t.vertices,
        [
            Coords3D { x: 2, y: 1, z: 3 },
            Coords3D { x: 5, y: 1, z: 3 },
            Coords3D { x: 2, y: 1, z: 7 },
        ]
    );

    let t = Triangle::from_strip_triangle(Face::Negative(Axis::Z), 1, strip(), 1);
    assert_eq!(
        t.vertices,
        [
            Coords3D { x: 3, y: 2, z: 1 },
            Coords3D { x: 3, y: 5, z: 1 },
            Coords3D { x: 7, y: 2, z: 1 },
        ]
    );
}

#[test]
fn outward_vertices_face_away_from_the_solid() {
    for axis in Axis::AXES {
        for (face, sign) in [(Face::Positive(axis), 1), (Face::Negative(axis), -1)] {
            let t = Triangle::from_strip_triangle(face, 6, strip(), 1);
            let [a, b, c] = t.outward_vertices();
            let normal = cross(a, b, c);

            for other in Axis::AXES {
                let component = normal[other.as_usize()];
                if other == axis {
                    assert_eq!(component.signum(), sign, "{face:?}: {normal:?}");
                } else {
                    assert_eq!(component, 0, "{face:?}: {normal:?}");
                }
            }
        }
    }
}

#[test]
fn outward_vertices_is_a_permutation_of_the_vertices() {
    for axis in Axis::AXES {
        for face in [Face::Positive(axis), Face::Negative(axis)] {
            let t = Triangle::from_strip_triangle(face, 6, strip(), 1);
            let mut original = t.vertices.map(|v| (v.x, v.y, v.z));
            let mut outward = t.outward_vertices().map(|v| (v.x, v.y, v.z));
            original.sort();
            outward.sort();
            assert_eq!(original, outward);
        }
    }
}
