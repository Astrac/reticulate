use reticulate::{CompactOctree, Coords3D, Fixed8Octree, Walk, utils::PrintWalker};

pub fn test_scene(octree: &mut Fixed8Octree<()>) {
    add_2x2(octree);
    // add_nested_cubes(octree);

    let compressed = CompactOctree::from_fixed8(&octree);
    let mut print_walker = PrintWalker::default();
    octree.walk(&mut print_walker);
    println!("{compressed:?}");
    compressed.walk(&mut print_walker);
    // let mut print_walker = DensePrintWalker::default();
    // octree.walk_dense(&mut print_walker);
}

pub fn add_nested_cubes(octree: &mut Fixed8Octree<()>) {
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                if (y == 1 || y == 2) && (x == 1 || x == 2) && (z == 3 || z == 4) {
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
