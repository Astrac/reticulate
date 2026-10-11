use std::collections::HashMap;

use reticulate::MaterialId;

use crate::ChunkDemo;

pub fn chunk_multi_material_non_minimal_triangles_example(
    floor: MaterialId,
    wall: MaterialId,
) -> ChunkDemo {
    let mut voxels = HashMap::<(u8, u8, u8), _>::new();
    voxels.insert((2, 1, 1), floor);
    voxels.insert((2, 1, 2), floor);
    voxels.insert((3, 1, 2), floor);
    voxels.insert((3, 1, 1), wall);
    voxels.insert((3, 2, 1), wall);
    voxels.insert((3, 2, 2), wall);

    ChunkDemo::new(
        "non_minimal_triangles_by_buried_material_seam",
        move |coords| *voxels.get(&(coords.x, coords.y, coords.z)).unwrap_or(&0),
    )
}
