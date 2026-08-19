use reticulate::{CompactOctree, Coords3D, DenseWalk, Fixed8Octree, Walk, utils::*};

pub fn test_scene() -> impl DenseWalk<()> {
    let mut octree: Fixed8Octree<()> = Fixed8Octree::new();

    // add_2x2(&mut octree);
    // add_8x8(&mut octree);
    add_nested_cubes(&mut octree);

    let compressed = CompactOctree::from_fixed8(&octree);
    let mut print_walker = PrintWalker::default();
    compressed.walk(&mut print_walker);

    compressed
    // octree
}

pub fn add_8x8(octree: &mut Fixed8Octree<()>) {
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}


pub fn add_nested_cubes(octree: &mut Fixed8Octree<()>) {
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                if (y == 3 || y == 4) && (x == 3 || x == 4) && (z == 3 || z == 4) {
                    continue;
                }

                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}

pub fn add_2x2(octree: &mut Fixed8Octree<()>) {
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                octree.update(Coords3D { x: x, y: y, z: z }, Some(()));
            }
        }
    }
}
