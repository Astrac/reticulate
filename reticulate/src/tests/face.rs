use crate::{Axis, Face};

#[test]
fn axis_ignores_the_direction() {
    for axis in Axis::AXES {
        assert_eq!(Face::Positive(axis).axis(), axis);
        assert_eq!(Face::Negative(axis).axis(), axis);
    }
}

#[test]
fn as_usize_numbers_positive_faces_0_to_2_and_negative_faces_3_to_5() {
    for axis in Axis::AXES {
        assert_eq!(Face::Positive(axis).as_usize(), axis.as_usize());
        assert_eq!(Face::Negative(axis).as_usize(), axis.as_usize() + 3);
    }
}

#[test]
fn as_usize_is_unique_per_face() {
    let mut seen = [false; 6];
    for axis in Axis::AXES {
        for face in [Face::Positive(axis), Face::Negative(axis)] {
            assert!(
                !seen[face.as_usize()],
                "{face:?} collides with another face"
            );
            seen[face.as_usize()] = true;
        }
    }
    assert!(seen.iter().all(|&s| s));
}
