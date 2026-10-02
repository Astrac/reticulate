use crate::{
    Axis, PLANE_SIDE,
    meshing::{bitfields::Parity, surface_layer::SurfaceLayer},
};

pub fn debug_assert_within_footprint(
    parity: &Parity,
    layer: SurfaceLayer,
    filled: &[u32; PLANE_SIDE],
) {
    for (u, &claimed) in filled.iter().enumerate() {
        let footprint = gather_footprint(parity, layer, u);
        debug_assert!(
            claimed & !footprint == 0,
            "strip claimed non-face cells on {:?}@{}, row {u}: {:032b}",
            layer.axis,
            layer.index,
            claimed & !footprint
        );
    }
}

// TODO - Use this or remove it
#[allow(unused)]
pub fn find_unclaimed_faces(
    parity: &crate::meshing::bitfields::Parity,
    occlusion: &crate::meshing::bitfields::Claims,
) -> Vec<(crate::Axis, usize, crate::Coords2D)> {
    let mut missing = Vec::new();
    for axis in Axis::AXES {
        for axis_coord in 0..PLANE_SIDE {
            for u in 0..PLANE_SIDE {
                let mut bits =
                    gather_footprint(parity, SurfaceLayer::new(axis, axis_coord as u8), u);
                while bits != 0 {
                    let v = bits.trailing_zeros();
                    bits &= bits - 1;
                    let coords = crate::Coords2D {
                        u: u as u8,
                        v: v as u8,
                    };
                    if occlusion.is_unclaimed(SurfaceLayer::new(axis, axis_coord as u8), coords) {
                        missing.push((axis, axis_coord, coords));
                    }
                }
            }
        }
    }
    missing
}

fn gather_footprint(parity: &Parity, layer: SurfaceLayer, u: usize) -> u32 {
    let words = parity.masks[layer.axis.as_usize()].get_layer(u);
    let mut row = 0u32;
    for (v, word) in words.iter().enumerate() {
        row |= ((word >> layer.index) & 1) << v;
    }
    row
}
