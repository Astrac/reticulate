use crate::{
    Coords3D, DenseWalk, Depth, Fixed8Octree, Morton3D, Walk,
    utils::testing::{RecordingWalker, WalkEvent},
};

fn tree_with(points: &[((u8, u8, u8), &str)]) -> Fixed8Octree<String> {
    let mut tree = Fixed8Octree::new();
    for &((x, y, z), data) in points {
        tree.update(Coords3D { x, y, z }, Some(data.to_string()));
    }
    tree
}

#[test]
fn walk_empty_tree_reports_nothing() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert!(walker.events.is_empty());
}

#[test]
fn walk_visits_single_leaf() {
    let tree = tree_with(&[((0, 0, 0), "leaf1")]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert_eq!(
        walker.leaves(),
        vec![(Morton3D::from_code(0, Depth::MAX), "leaf1".to_string())]
    );
}

#[test]
fn walk_visits_multiple_leaves_in_morton_order() {
    let tree = tree_with(&[((1, 1, 1), "b"), ((0, 0, 0), "a"), ((31, 31, 31), "c")]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    let leaves = walker.leaves();
    let data: Vec<_> = leaves.iter().map(|(_, d)| d.as_str()).collect();
    assert_eq!(data, ["a", "b", "c"]);
    let codes: Vec<_> = leaves.iter().map(|(m, _)| m.as_u32()).collect();
    assert!(codes.windows(2).all(|w| w[0] < w[1]), "{codes:?}");
    assert_eq!(codes, [0, 0b111, 0b111_111_111_111_111]);
}

#[test]
fn walk_branches_are_balanced_and_nested() {
    let tree = tree_with(&[((0, 0, 0), "leaf"), ((20, 7, 13), "other")]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert_eq!(walker.branch_starts(), walker.branch_ends());
    assert!(walker.branch_starts() > 0);
    walker.assert_well_nested();
}

#[test]
fn walk_reports_the_root_branch_first_and_last() {
    let tree = tree_with(&[((3, 4, 5), "x")]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    let root = Morton3D::from_code(0, Depth(0));
    assert!(matches!(
        walker.events.first(),
        Some(WalkEvent::BranchStart { morton, .. }) if *morton == root
    ));
    assert_eq!(
        walker.events.last(),
        Some(&WalkEvent::BranchEnd { morton: root })
    );
}

#[test]
fn walk_full_octants() {
    let mut tree = Fixed8Octree::new();
    for x in 0..4 {
        for y in 0..4 {
            for z in 0..4 {
                tree.update(Coords3D { x, y, z }, Some(format!("({x},{y},{z})")));
            }
        }
    }

    let mut walker = RecordingWalker::new();
    tree.walk_dense(&mut walker);

    assert_eq!(walker.branch_starts(), walker.branch_ends());
    assert_eq!(
        walker.leaves().len(),
        64,
        "There should be 64 terminal leaves in a 4x4x4 cube"
    );
    walker.assert_well_nested();
}

#[test]
fn dense_walk_of_an_empty_tree_is_one_empty_root() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    let mut walker = RecordingWalker::new();

    tree.walk_dense(&mut walker);

    assert_eq!(
        walker.events,
        vec![WalkEvent::Empty {
            morton: Morton3D::from_code(0, Depth(0))
        }]
    );
}

#[test]
fn dense_walk_tiles_the_whole_chunk() {
    let tree = tree_with(&[((0, 0, 0), "a"), ((5, 9, 2), "b"), ((31, 31, 31), "c")]);
    let mut walker = RecordingWalker::new();

    tree.walk_dense(&mut walker);

    assert_eq!(walker.covered_volume(), 32 * 32 * 32);
    assert_eq!(walker.leaves().len(), 3);
    assert!(!walker.empties().is_empty());
    walker.assert_well_nested();
}

#[test]
fn dense_and_sparse_walks_agree_on_leaves() {
    let tree = tree_with(&[((0, 0, 0), "a"), ((5, 9, 2), "b"), ((31, 0, 17), "c")]);
    let (mut sparse, mut dense) = (RecordingWalker::new(), RecordingWalker::new());

    tree.walk(&mut sparse);
    tree.walk_dense(&mut dense);

    assert_eq!(sparse.leaves(), dense.leaves());
}

#[test]
fn removed_voxel_shows_up_as_empty_in_dense_walk() {
    let mut tree = tree_with(&[((6, 6, 6), "a"), ((7, 7, 7), "b")]);
    tree.update(Coords3D { x: 6, y: 6, z: 6 }, None);
    let mut walker = RecordingWalker::new();

    tree.walk_dense(&mut walker);

    let gone = Morton3D::from_xyz_max_depth(6, 6, 6);
    assert!(walker.empties().contains(&gone));
    assert_eq!(walker.leaves().len(), 1);
    assert_eq!(walker.covered_volume(), 32 * 32 * 32);
}
