use crate::{
    CompactOctree, Fixed8Octree,
    octree::{CompactOctreeNode, Fixed8OctreeNode},
};

fn empty_node() -> Fixed8OctreeNode<i32> {
    Fixed8OctreeNode::Leaf { data: None }
}

fn leaf_node(val: i32) -> Fixed8OctreeNode<i32> {
    Fixed8OctreeNode::Leaf { data: Some(val) }
}

#[test]
fn test_empty_tree() {
    let tree: Fixed8Octree<i32> = Fixed8Octree { nodes: Vec::new() };
    let compact = CompactOctree::from_fixed8(&tree);
    assert!(compact.nodes.is_empty());
}

#[test]
fn test_single_root_leaf() {
    let tree = Fixed8Octree {
        nodes: vec![leaf_node(42)],
    };
    let compact = CompactOctree::from_fixed8(&tree);

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
    let compact = CompactOctree::from_fixed8(&tree);
    assert!(compact.nodes.is_empty());
}

#[test]
fn test_branch_compression_and_mask() {
    let mut nodes = vec![Fixed8OctreeNode::Branch {
        children_index: 1,
        children_mask: 0xff,
    }];

    nodes.push(leaf_node(10));
    nodes.push(empty_node());
    nodes.push(empty_node());
    nodes.push(leaf_node(40));
    for _ in 4..8 {
        nodes.push(empty_node());
    }

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(&tree);

    assert_eq!(compact.nodes.len(), 3);

    println!("{compact:?}");

    match &compact.nodes[0] {
        CompactOctreeNode::Branch(mask) => assert_eq!(*mask, 9),
        _ => panic!("Expected a Branch node"),
    }

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
    let mut nodes = vec![Fixed8OctreeNode::Branch {
        children_index: 1,
        children_mask: 0xff,
    }];
    for _ in 0..8 {
        nodes.push(leaf_node(7));
    }

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(&tree);

    assert_eq!(compact.nodes.len(), 1);
    match &compact.nodes[0] {
        CompactOctreeNode::Leaf(val) => assert_eq!(*val, 7),
        _ => panic!("Expected all nodes to collapse into a single unified leaf"),
    }
}

#[test]
fn test_no_coalescing_on_mixed_values() {
    let mut nodes = vec![Fixed8OctreeNode::Branch {
        children_index: 1,
        children_mask: 0xff,
    }];
    for _ in 0..7 {
        nodes.push(leaf_node(7));
    }
    nodes.push(leaf_node(99));

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(&tree);

    assert_eq!(compact.nodes.len(), 9);
    match &compact.nodes[0] {
        CompactOctreeNode::Branch(mask) => assert_eq!(*mask, 0xFF),
        _ => panic!("Expected uncollapsed Branch node"),
    }
}

#[test]
fn test_empty_branch_pruned() {
    let mut nodes = vec![Fixed8OctreeNode::Branch {
        children_index: 1,
        children_mask: 0xff,
    }];
    for _ in 0..8 {
        nodes.push(empty_node());
    }

    let tree = Fixed8Octree { nodes };
    let compact = CompactOctree::from_fixed8(&tree);

    println!("{compact:?}");
    assert!(compact.nodes.is_empty());
}
