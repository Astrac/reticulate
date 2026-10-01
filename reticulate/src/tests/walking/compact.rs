use crate::{
    CompactOctree, Coords3D, DenseWalk, Depth, Fixed8Octree, Morton3D, Walk,
    utils::testing::{RecordingWalker, SeededRandom, WalkEvent},
};

const CHUNK_VOXELS: u32 = 32 * 32 * 32;

fn fixed8_with(points: &[((u8, u8, u8), u32)]) -> Fixed8Octree<u32> {
    let mut tree = Fixed8Octree::new();
    for &((x, y, z), data) in points {
        tree.update(Coords3D { x, y, z }, Some(data));
    }
    tree
}

fn compact_with(points: &[((u8, u8, u8), u32)]) -> CompactOctree<u32> {
    CompactOctree::from_fixed8(&fixed8_with(points))
}

#[test]
fn walk_empty_tree_reports_nothing() {
    let tree: CompactOctree<u32> = CompactOctree::default();
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert!(walker.events.is_empty());
}

#[test]
fn walk_visits_single_leaf() {
    let tree = compact_with(&[((0, 0, 0), 7)]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert_eq!(
        walker.leaves(),
        vec![(Morton3D::from_code(0, Depth::MAX), 7)]
    );
    assert_eq!(walker.branch_starts(), walker.branch_ends());
    walker.assert_well_nested();
}

#[test]
fn walk_reports_the_real_children_mask() {
    // Two voxels in different octants of the root.
    let tree = compact_with(&[((0, 0, 0), 1), ((16, 0, 0), 2)]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert!(matches!(
        walker.events.first(),
        Some(WalkEvent::BranchStart { morton, mask: 0b11 })
            if *morton == Morton3D::from_code(0, Depth(0))
    ));
}

#[test]
fn uniform_octant_collapses_into_one_leaf() {
    let mut points = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                points.push(((x, y, z), 5));
            }
        }
    }
    let tree = compact_with(&points);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert_eq!(
        walker.leaves(),
        vec![(Morton3D::from_code(0, Depth(4)), 5)],
        "eight equal siblings must be reported as their parent"
    );
}

#[test]
fn mixed_octant_does_not_collapse() {
    let mut points = Vec::new();
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                points.push(((x, y, z), 5));
            }
        }
    }
    points[3].1 = 6;
    let tree = compact_with(&points);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert_eq!(walker.leaves().len(), 8);
}

#[test]
fn partial_octant_does_not_collapse() {
    let tree = compact_with(&[((0, 0, 0), 5), ((1, 0, 0), 5)]);
    let mut walker = RecordingWalker::new();

    tree.walk(&mut walker);

    assert_eq!(walker.leaves().len(), 2);
}

#[test]
fn walk_covers_the_same_voxels_as_the_fixed8_walk() {
    let points = [
        ((0, 0, 0), 1),
        ((1, 0, 0), 1),
        ((5, 9, 2), 2),
        ((31, 31, 31), 3),
        ((16, 16, 16), 3),
    ];
    let fixed8 = fixed8_with(&points);
    let compact = CompactOctree::from_fixed8(&fixed8);
    let (mut expected, mut actual) = (RecordingWalker::new(), RecordingWalker::new());

    fixed8.walk(&mut expected);
    compact.walk(&mut actual);

    assert_eq!(actual.leaf_voxels(), expected.leaf_voxels());
    actual.assert_well_nested();
}

#[test]
fn dense_walk_of_a_single_leaf_reports_seven_empty_siblings_per_level() {
    let tree = compact_with(&[((0, 0, 0), 7)]);
    let mut walker = RecordingWalker::new();

    tree.walk_dense(&mut walker);

    assert_eq!(walker.leaves().len(), 1);
    assert_eq!(walker.empties().len(), 7 * 5);
    assert_eq!(walker.covered_volume(), CHUNK_VOXELS);
    walker.assert_well_nested();
}

/// Voxel (24, 16, 0) lies in octant 0b011 of the root (x and y in the
/// upper half), so its siblings are empty on both sides of it.
#[test]
fn dense_walk_reports_empties_before_and_after_the_leaf_path() {
    let tree = compact_with(&[((24, 16, 0), 9)]);
    let mut walker = RecordingWalker::new();

    tree.walk_dense(&mut walker);

    let root_children: Vec<u8> = walker
        .empties()
        .iter()
        .filter(|e| e.depth() == Depth(1))
        .map(|e| e.octant())
        .collect();
    assert_eq!(root_children, [0, 1, 2, 4, 5, 6, 7]);
    assert_eq!(walker.covered_volume(), CHUNK_VOXELS);
}

#[test]
fn dense_walk_tiles_the_chunk_for_random_trees() {
    let mut rng = SeededRandom::new(0xA572AC);
    for round in 0..25 {
        let mut points = Vec::new();
        for _ in 0..(1 + rng.next(60)) {
            let span = if round % 2 == 0 { 32 } else { 4 };
            points.push((
                (
                    rng.next(span) as u8,
                    rng.next(span) as u8,
                    rng.next(span) as u8,
                ),
                1 + rng.next(3) as u32,
            ));
        }

        let fixed8 = fixed8_with(&points);
        let compact = CompactOctree::from_fixed8(&fixed8);

        let (mut from_fixed8, mut from_compact) = (RecordingWalker::new(), RecordingWalker::new());
        fixed8.walk_dense(&mut from_fixed8);
        compact.walk_dense(&mut from_compact);

        assert_eq!(from_compact.covered_volume(), CHUNK_VOXELS, "round {round}");
        assert_eq!(from_fixed8.covered_volume(), CHUNK_VOXELS, "round {round}");
        assert_eq!(
            from_compact.leaf_voxels(),
            from_fixed8.leaf_voxels(),
            "round {round}"
        );
        from_compact.assert_well_nested();
        from_fixed8.assert_well_nested();
    }
}
