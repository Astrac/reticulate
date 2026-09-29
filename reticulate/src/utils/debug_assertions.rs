use crate::{
    axis::Axis,
    meshing::{
        bitfields::{PLANE_SIDE, Parity},
        surface_layer::SurfaceLayer,
    },
};

pub fn debug_assert_within_footprint(
    parity: &Parity,
    axis: Axis,
    layer: usize,
    filled: &[u32; PLANE_SIDE],
) {
    for (u, &claimed) in filled.iter().enumerate() {
        let footprint = gather_footprint(parity, axis, layer, u);
        debug_assert!(
            claimed & !footprint == 0,
            "strip claimed non-face cells on {:?}@{}, row {u}: {:032b}",
            axis,
            layer,
            claimed & !footprint
        );
    }
}

// TODO - Use this or remove it
pub fn find_unclaimed_faces(
    parity: &crate::meshing::bitfields::Parity,
    occlusion: &crate::meshing::bitfields::Claims,
) -> Vec<(crate::axis::Axis, usize, crate::Coords2D)> {
    let mut missing = Vec::new();
    for axis in [
        crate::axis::AXIS_X,
        crate::axis::AXIS_Y,
        crate::axis::AXIS_Z,
    ] {
        for axis_coord in 0..crate::meshing::bitfields::PLANE_SIDE {
            for u in 0..crate::meshing::bitfields::PLANE_SIDE {
                let mut bits = gather_footprint(&parity, axis, axis_coord, u);
                while bits != 0 {
                    let v = bits.trailing_zeros();
                    bits &= bits - 1;
                    let coords = crate::Coords2D { u: u as u32, v };
                    if occlusion.is_unclaimed(SurfaceLayer::new(axis, axis_coord), coords) {
                        missing.push((axis, axis_coord, coords));
                    }
                }
            }
        }
    }
    missing
}

fn gather_footprint(parity: &Parity, axis: Axis, axis_coord: usize, u: usize) -> u32 {
    let words = parity.masks[axis.as_usize()].get_layer(u);
    let mut row = 0u32;
    for v in 0..PLANE_SIDE {
        row |= ((words[v] >> axis_coord) & 1) << v;
    }
    row
}
