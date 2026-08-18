use reticulate::{Coords3D, Fixed8Octree};

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
