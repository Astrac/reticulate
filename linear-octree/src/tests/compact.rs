use crate::{CompactOctree, CompactOctreeNode, Fixed8Octree, Fixed8OctreeNode};

// Helper to quickly build a flat vector representing an empty leaf slot
fn empty_node() -> Fixed8OctreeNode<i32> {
    Fixed8OctreeNode::Leaf { data: None }
}

// Helper to quickly build a flat vector representing a populated leaf slot
fn leaf_node(val: i32) -> Fixed8OctreeNode<i32> {
    Fixed8OctreeNode::Leaf { data: Some(val) }
}

#[test]
fn test_empty_tree() {
    let tree: Fixed8Octree<i32> = Fixed8Octree { nodes: Vec::new() };
    let compact = CompactOctree::from_fixed8(tree);
    assert!(compact.nodes.is_empty());
}

#[test]
fn test_single_root_leaf() {
    let tree = Fixed8Octree {
        nodes: vec![leaf_node(42)],
    };
    let compact = CompactOctree::from_fixed8(tree);

    assert_eq!(compact.nodes.len(), 1);
    match &compact.nodes[0] {
        CompactOctreeNode::Leaf(val) => assert_eq!(*val, 42),
        _ => panic!("Expected a Leaf node"),
    }
}

#[test]
fn test_root_empty_leaf_pruned() {
    let tree = Fixed8Octree {
        nodes: vec![empty_node()],
    };
    let compact = CompactOctree::from_fixed8(tree);
    assert!(compact.nodes.is_empty());
}

#[test]
fn test_branch_compression_and_mask() {
    // Create a branch where only child 0 and child 3 have data
    // Indices: Root is 0, children start at 1
    let mut nodes = vec![Fixed8OctreeNode::Branch { children_index: 1 }];

    // Push 8 children
    nodes.push(leaf_node(10)); // Child 0 (bit 0)
    nodes.push(empty_node()); // Child 1
    nodes.push(empty_node()); // Child 2
    nodes.push(leaf_node(40)); // Child 3 (bit 3)
    for _ in 4..8 {
        nodes.push(empty_node());
    }

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(tree);

    // Total layout size should be 3: 1 Branch + 2 active Leaves
    assert_eq!(compact.nodes.len(), 3);

    // Verify bitmask: (1 << 0) | (1 << 3) = 1 | 8 = 9
    match &compact.nodes[0] {
        CompactOctreeNode::Branch(mask) => assert_eq!(*mask, 9),
        _ => panic!("Expected a Branch node"),
    }

    // Verify sequential child layout
    match &compact.nodes[1] {
        CompactOctreeNode::Leaf(val) => assert_eq!(*val, 10),
        _ => panic!("Expected first child leaf"),
    }
    match &compact.nodes[2] {
        CompactOctreeNode::Leaf(val) => assert_eq!(*val, 40),
        _ => panic!("Expected second child leaf"),
    }
}

#[test]
fn test_coalescing_identical_leaves() {
    // Create a branch where all 8 children have the exact same data value
    let mut nodes = vec![Fixed8OctreeNode::Branch { children_index: 1 }];
    for _ in 0..8 {
        nodes.push(leaf_node(7));
    }

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(tree);

    // The branch and all 8 leaves should collapse down into a single Leaf node
    assert_eq!(compact.nodes.len(), 1);
    match &compact.nodes[0] {
        CompactOctreeNode::Leaf(val) => assert_eq!(*val, 7),
        _ => panic!("Expected all nodes to collapse into a single unified leaf"),
    }
}

#[test]
fn test_no_coalescing_on_mixed_values() {
    // Create a branch where all 8 slots are populated, but one value differs
    let mut nodes = vec![Fixed8OctreeNode::Branch { children_index: 1 }];
    for _ in 0..7 {
        nodes.push(leaf_node(7));
    }
    nodes.push(leaf_node(99)); // The odd one out

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(tree);

    // Should NOT coalesce. Total size = 1 Branch + 8 Leaves = 9 nodes
    assert_eq!(compact.nodes.len(), 9);
    match &compact.nodes[0] {
        CompactOctreeNode::Branch(mask) => assert_eq!(*mask, 0xFF),
        _ => panic!("Expected uncollapsed Branch node"),
    }
}

#[test]
fn test_empty_branch_pruned() {
    // Create a branch where all 8 children are None leaves
    let mut nodes = vec![Fixed8OctreeNode::Branch { children_index: 1 }];
    for _ in 0..8 {
        nodes.push(empty_node());
    }

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(tree);

    // The entire sub-hierarchy is empty, so it should prune down to zero nodes
    assert!(compact.nodes.is_empty());
}
