use crate::{
    meshing::{
        bitfields::{Bitfields, BitfieldsStorage, MaterialId, PLANE_SIDE},
        gathering_walker::GatheringWalker,
    },
    utils::testing::{
        DIRT, Fixture, STONE, VoxelGrid,
        fixtures::parity_of,
        mesh::{Block, dense_walk},
    },
};

/// Runs the gathering pass over `voxels` and checks that the parity it
/// gathered is exactly what a brute-force comparison of neighbouring
/// voxels gives.
fn assert_gathers_brute_force_parity(
    voxels: &[((usize, usize, usize), MaterialId)],
    blocks: &[Block],
) {
    let mut reference = Fixture::new(voxels);
    let reference_parity = parity_of(&mut reference.parity);

    let mut storage = BitfieldsStorage::default();
    let bitfields = Bitfields::from_storage(&mut storage);
    let mut to_material = |m: &MaterialId| *m;
    let mut walker = GatheringWalker::new(bitfields, &mut to_material);
    dense_walk(&mut walker, &VoxelGrid::new(voxels), blocks);

    let gathered = Bitfields::from_storage(&mut storage).parity;
    for axis in 0..3 {
        for layer in 0..PLANE_SIDE {
            assert_eq!(
                gathered.masks[axis].get_layer(layer),
                reference_parity.masks[axis].get_layer(layer),
                "axis {axis}, layer {layer}"
            );
        }
    }
}

#[test]
fn empty_chunk_has_no_transitions() {
    assert_gathers_brute_force_parity(&[], &[]);
}

#[test]
fn single_voxel_has_a_transition_on_each_side_of_each_axis() {
    assert_gathers_brute_force_parity(&[((2, 3, 4), STONE)], &[]);
}

#[test]
fn voxel_at_the_chunk_origin() {
    assert_gathers_brute_force_parity(&[((0, 0, 0), STONE)], &[]);
}

#[test]
fn same_material_neighbours_have_no_transition_between_them() {
    assert_gathers_brute_force_parity(&[((2, 3, 4), STONE), ((3, 3, 4), STONE)], &[]);
}

#[test]
fn different_materials_have_a_transition_between_them() {
    assert_gathers_brute_force_parity(&[((2, 3, 4), STONE), ((3, 3, 4), DIRT)], &[]);
}

#[test]
fn large_nodes_are_gathered_as_a_single_transition() {
    let mut voxels = Vec::new();
    for x in 4..8 {
        for y in 4..8 {
            for z in 4..8 {
                voxels.push(((x, y, z), STONE));
            }
        }
    }
    assert_gathers_brute_force_parity(&voxels, &[((4, 4, 4), 3)]);
}

#[test]
fn large_empty_nodes_next_to_solids() {
    assert_gathers_brute_force_parity(&[((1, 1, 1), STONE)], &[((2, 0, 0), 4)]);
}

#[test]
fn large_nodes_of_different_materials_touch() {
    let mut voxels = Vec::new();
    for x in 0..8 {
        for y in 0..8 {
            for z in 0..8 {
                let material = if x < 4 { STONE } else { DIRT };
                voxels.push(((x, y, z), material));
            }
        }
    }
    assert_gathers_brute_force_parity(&voxels, &[((0, 0, 0), 3)]);
}
