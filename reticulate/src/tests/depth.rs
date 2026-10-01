use crate::{
    Depth,
    meshing::bitfields::{PLANE_AREA, PLANE_SIDE},
    octree::TREE_DEPTH,
};

#[test]
fn max_is_the_tree_depth() {
    assert_eq!(Depth::MAX, Depth(TREE_DEPTH as u8));
}

#[test]
fn default_is_the_root() {
    assert_eq!(Depth::default(), Depth(0));
}

#[test]
fn side_halves_with_every_level() {
    let sides: Vec<_> = (0..=TREE_DEPTH).map(|d| Depth::from(d).as_side()).collect();
    assert_eq!(sides, [32, 16, 8, 4, 2, 1]);
    assert_eq!(Depth(0).as_side(), PLANE_SIDE);
}

#[test]
fn area_quarters_with_every_level() {
    let areas: Vec<_> = (0..=TREE_DEPTH).map(|d| Depth::from(d).as_area()).collect();
    assert_eq!(areas, [1024, 256, 64, 16, 4, 1]);
    assert_eq!(Depth(0).as_area(), PLANE_AREA);
}

#[test]
fn area_is_side_squared() {
    for d in 0..=TREE_DEPTH {
        let depth = Depth::from(d);
        assert_eq!(depth.as_area(), depth.as_side() * depth.as_side());
    }
}

#[test]
fn arithmetic_with_plain_integers() {
    assert_eq!(Depth(2) + 3, Depth(5));
    assert_eq!(Depth(5) - 3, Depth(2));
    assert_eq!(5u8 - Depth(2), 3);
}

#[test]
fn conversions_round_trip() {
    assert_eq!(Depth::from(3u8), Depth(3));
    assert_eq!(Depth::from(3usize), Depth(3));
    assert_eq!(u8::from(Depth(3)), 3);
    assert_eq!(Depth(3).as_usize(), 3);
}

#[test]
fn depths_are_ordered_by_level() {
    assert!(Depth(1) < Depth(2));
    assert!(Depth(0) < Depth::MAX);
}
