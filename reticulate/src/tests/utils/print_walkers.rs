use crate::{
    CompactOctree, Coords3D, DenseWalk, Fixed8Octree, Walk,
    utils::{DensePrintWalker, PrintWalker},
};

fn sample() -> Fixed8Octree<u32> {
    let mut tree = Fixed8Octree::new();
    tree.update(Coords3D { x: 0, y: 0, z: 0 }, Some(1));
    tree.update(Coords3D { x: 17, y: 4, z: 30 }, Some(2));
    tree
}

// The print walkers only write to stdout; these tests make sure their
// indentation bookkeeping survives a whole walk (it underflows if a
// branch end is ever reported without its start).

#[test]
fn print_walker_handles_a_fixed8_walk() {
    sample().walk(&mut PrintWalker::default());
}

#[test]
fn print_walker_handles_a_compact_walk() {
    CompactOctree::from_fixed8(&sample()).walk(&mut PrintWalker::default());
}

#[test]
fn dense_print_walker_handles_a_fixed8_walk() {
    sample().walk_dense(&mut DensePrintWalker::default());
}

#[test]
fn dense_print_walker_handles_a_compact_walk() {
    CompactOctree::from_fixed8(&sample()).walk_dense(&mut DensePrintWalker::default());
}
