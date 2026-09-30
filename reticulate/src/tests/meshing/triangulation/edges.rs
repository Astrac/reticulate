use crate::{
    Coords2D,
    axis::Axis,
    meshing::{bitfields::Parity, surface_layer::SurfaceLayer, triangulation::edges::Edges},
    utils::testing::{parity_of, storage},
};

/// Sets a parity transition at voxel (x, y, z) on the given axis, using the
/// (plane, row, column) convention of each axis: X -> (u: y, v: z),
/// Y -> (u: z, v: x), Z -> (u: x, v: y).
fn transition(parity: &mut Parity, axis: Axis, (x, y, z): (u8, u8, u8)) {
    let (layer, u, v) = match axis {
        Axis::X => (x, y, z),
        Axis::Y => (y, z, x),
        Axis::Z => (z, x, y),
    };
    parity.set_transition(SurfaceLayer::new(axis, layer), Coords2D { u, v }, 1);
}

// The walls of a plane are the transitions along the *third* axis (the one
// that is neither the plane's axis nor its columns' axis): rows of a Y plane
// are z, its columns are x, and its walls are the X transitions.

#[test]
fn y_plane_walls_come_from_x_transitions() {
    let mut s = storage();
    let mut parity = parity_of(&mut s);
    transition(&mut parity, Axis::X, (5, 7, 9));

    let edges = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 7));
    assert_eq!(edges.walls(9), 1 << 5);
    assert_eq!(edges.walls(8), 0);
}

#[test]
fn x_plane_walls_come_from_z_transitions() {
    let mut s = storage();
    let mut parity = parity_of(&mut s);
    transition(&mut parity, Axis::Z, (5, 7, 9));

    let edges = Edges::from_parity(&parity, SurfaceLayer::new(Axis::X, 5));
    assert_eq!(edges.walls(7), 1 << 9);
    assert_eq!(edges.walls(6), 0);
}

#[test]
fn z_plane_walls_come_from_y_transitions() {
    let mut s = storage();
    let mut parity = parity_of(&mut s);
    transition(&mut parity, Axis::Y, (5, 7, 9));

    let edges = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Z, 9));
    assert_eq!(edges.walls(5), 1 << 7);
    assert_eq!(edges.walls(4), 0);
}

#[test]
fn walls_include_the_previous_layer() {
    let mut s = storage();
    let mut parity = parity_of(&mut s);
    transition(&mut parity, Axis::X, (5, 7, 9));

    // The wall between plane y = 7 and y = 8 is seen by both planes.
    let below = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 6));
    let same = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 7));
    let above = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 8));
    let beyond = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 9));
    assert_eq!(below.walls(9), 0);
    assert_eq!(same.walls(9), 1 << 5);
    assert_eq!(above.walls(9), 1 << 5);
    assert_eq!(beyond.walls(9), 0);
}

#[test]
fn walls_merge_current_and_previous_layers() {
    let mut s = storage();
    let mut parity = parity_of(&mut s);
    transition(&mut parity, Axis::X, (5, 7, 9));
    transition(&mut parity, Axis::X, (12, 8, 9));

    let edges = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 8));
    assert_eq!(edges.walls(9), (1 << 5) | (1 << 12));
}

#[test]
fn first_layer_has_no_previous_layer() {
    let mut s = storage();
    let mut parity = parity_of(&mut s);
    transition(&mut parity, Axis::X, (5, 0, 9));

    let edges = Edges::from_parity(&parity, SurfaceLayer::new(Axis::Y, 0));
    assert_eq!(edges.walls(9), 1 << 5);
}
