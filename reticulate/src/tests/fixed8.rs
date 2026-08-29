use crate::{Coords3D, DenseWalk, DenseWalker, Depth, Fixed8Octree, Morton3D, Walk, Walker};

struct TestWalker {
    events: Vec<WalkEvent>,
}

#[derive(Debug, Clone, PartialEq)]
enum WalkEvent {
    BranchStart {
        depth: Depth,
        morton: Morton3D,
        mask: u8,
    },
    BranchEnd {
        depth: Depth,
        morton: Morton3D,
    },
    Leaf {
        depth: Depth,
        morton: Morton3D,
        data: String,
    },
    Empty {
        depth: Depth,
        morton: Morton3D,
    },
}

impl Walker<String> for TestWalker {
    fn on_branch_start(&mut self, depth: Depth, morton: Morton3D, children_mask: u8) {
        self.events.push(WalkEvent::BranchStart {
            depth,
            morton,
            mask: children_mask,
        });
    }

    fn on_branch_end(&mut self, depth: Depth, morton: Morton3D) {
        self.events.push(WalkEvent::BranchEnd { depth, morton });
    }

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &String) {
        self.events.push(WalkEvent::Leaf {
            depth,
            morton,
            data: data.clone(),
        });
    }
}

impl DenseWalker<String> for TestWalker {
    fn on_branch_start(&mut self, depth: Depth, morton: Morton3D, children_mask: u8) {
        self.events.push(WalkEvent::BranchStart {
            depth,
            morton,
            mask: children_mask,
        });
    }

    fn on_branch_end(&mut self, depth: Depth, morton: Morton3D) {
        self.events.push(WalkEvent::BranchEnd { depth, morton });
    }

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &String) {
        self.events.push(WalkEvent::Leaf {
            depth,
            morton,
            data: data.clone(),
        });
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        self.events.push(WalkEvent::Empty { depth, morton });
    }
}

#[test]
fn test_new_initializes_with_root() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    assert!(
        !tree.nodes().is_empty(),
        "New tree should have at least a root node"
    );
}

#[test]
fn test_update_inserts_and_retrieves_data() {
    let mut tree = Fixed8Octree::new();
    let coords = Coords3D {
        x: 10,
        y: 20,
        z: 30,
    };
    let data = "test_value".to_string();

    tree.update(coords, Some(data.clone()));

    assert_eq!(tree.get(coords), Some(&data));
}

#[test]
fn test_update_overwrites_existing_data() {
    let mut tree = Fixed8Octree::new();
    let coords = Coords3D { x: 0, y: 0, z: 0 };

    tree.update(coords, Some("old".to_string()));
    tree.update(coords, Some("new".to_string()));

    assert_eq!(tree.get(coords), Some(&"new".to_string()));
}

#[test]
fn test_update_removes_data() {
    let mut tree = Fixed8Octree::new();
    let coords = Coords3D { x: 0, y: 0, z: 0 };

    tree.update(coords, Some("data".to_string()));
    assert!(tree.get(coords).is_some());

    tree.update(coords, None);
    assert_eq!(tree.get(coords), None);
}

#[test]
fn test_get_returns_none_for_missing() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    let coords = Coords3D {
        x: 100,
        y: 100,
        z: 100,
    };

    assert_eq!(tree.get(coords), None);
}

#[test]
fn test_walk_empty_tree() {
    let tree: Fixed8Octree<String> = Fixed8Octree::new();
    let mut walker = TestWalker { events: Vec::new() };

    tree.walk(&mut walker);

    assert!(walker.events.is_empty());
}

#[test]
fn test_walk_visits_single_leaf() {
    let mut tree = Fixed8Octree::new();
    tree.update(Coords3D { x: 0, y: 0, z: 0 }, Some("leaf1".to_string()));

    let mut walker = TestWalker { events: Vec::new() };
    tree.walk(&mut walker);

    // Should contain exactly one leaf event with the correct data
    let leaves: Vec<&WalkEvent> = walker
        .events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Leaf { .. }))
        .collect();
    assert_eq!(leaves.len(), 1);

    if let WalkEvent::Leaf { data, .. } = leaves[0] {
        assert_eq!(data, "leaf1");
    } else {
        panic!("Expected Leaf event");
    }
}

#[test]
fn test_walk_visits_multiple_leaves() {
    let mut tree = Fixed8Octree::new();
    tree.update(Coords3D { x: 0, y: 0, z: 0 }, Some("a".to_string()));
    tree.update(Coords3D { x: 1, y: 1, z: 1 }, Some("b".to_string()));

    let mut walker = TestWalker { events: Vec::new() };
    tree.walk(&mut walker);

    let mut leaf_data: Vec<String> = walker
        .events
        .iter()
        .filter_map(|e| {
            if let WalkEvent::Leaf { data, .. } = e {
                Some(data.clone())
            } else {
                None
            }
        })
        .collect();

    leaf_data.sort();

    assert_eq!(leaf_data, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn test_walk_order_branches() {
    let mut tree = Fixed8Octree::new();
    // Insert data to force branching structure
    tree.update(Coords3D { x: 0, y: 0, z: 0 }, Some("leaf".to_string()));

    let mut walker = TestWalker { events: Vec::new() };
    tree.walk(&mut walker);

    // Verify that branch starts and ends are balanced
    let starts = walker
        .events
        .iter()
        .filter(|e| matches!(e, WalkEvent::BranchStart { .. }))
        .count();
    let ends = walker
        .events
        .iter()
        .filter(|e| matches!(e, WalkEvent::BranchEnd { .. }))
        .count();

    assert_eq!(starts, ends, "Branch starts and ends should be balanced");
}

#[test]
fn test_walk_full_octants() {
    let mut tree = Fixed8Octree::new();
    for x in 0..4 {
        for y in 0..4 {
            for z in 0..4 {
                tree.update(Coords3D { x, y, z }, Some(format!("({x},{y},{z})")));
            }
        }
    }

    let mut walker = TestWalker { events: Vec::new() };
    tree.walk_dense(&mut walker);

    let starts = walker
        .events
        .iter()
        .filter(|e| matches!(e, WalkEvent::BranchStart { .. }))
        .count();
    let ends = walker
        .events
        .iter()
        .filter(|e| matches!(e, WalkEvent::BranchEnd { .. }))
        .count();

    assert_eq!(starts, ends, "Branch starts and ends should be balanced");

    let leaves = walker
        .events
        .iter()
        .filter(|e| matches!(e, WalkEvent::Leaf { .. }))
        .count();

    assert_eq!(
        64, leaves,
        "There should be 64 terminal leaves in a 4x4x4 cubes"
    );
}
