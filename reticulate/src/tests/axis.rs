use crate::Axis;

#[test]
fn axes_are_listed_in_index_order() {
    for (index, axis) in Axis::AXES.iter().enumerate() {
        assert_eq!(axis.as_usize(), index);
    }
}

#[test]
fn next_cycles_x_y_z() {
    assert_eq!(Axis::X.next(), Axis::Y);
    assert_eq!(Axis::Y.next(), Axis::Z);
    assert_eq!(Axis::Z.next(), Axis::X);
}

#[test]
fn three_nexts_come_back_to_the_start() {
    for axis in Axis::AXES {
        assert_eq!(axis.next().next().next(), axis);
    }
}
