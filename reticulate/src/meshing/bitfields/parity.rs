use crate::{Coords2D, axis::*, meshing::bitfields::PLANE_SIDE};

#[repr(transparent)]
pub struct ParityMask<'a>(&'a mut [u32]);

impl<'a> ParityMask<'a> {
    #[inline(always)]
    pub fn from_u64_slice(data: &'a mut [u64]) -> ParityMask<'a> {
        ParityMask(bytemuck::cast_slice_mut(data))
    }

    #[inline(always)]
    pub fn get_layer(&self, layer: usize) -> &[u32] {
        let index = layer * PLANE_SIDE;
        &self.0[index..index + PLANE_SIDE]
    }
}

/// Represents the material-parity violations along 3 axes. The mask is a sparse
/// collection of 1s only at the spots where there are material transitions. The
/// face extension algorithm scans the rows and uses follow the edges of the material
/// transition to reconstruct the surface. Indexing convention: the 1 represents
/// the first bit in sequences of uniform blocks.
#[repr(transparent)]
pub struct Parity<'a> {
    pub masks: [ParityMask<'a>; 3],
}

impl<'a> Parity<'a> {
    #[inline(always)]
    pub fn set_transition(
        &mut self,
        axis: Axis,
        axis_coord: usize,
        area_coords: Coords2D,
        area_size: usize,
    ) {
        let target = self.masks[axis.as_usize()].0.as_mut();
        let side = area_size.ilog2() as usize;
        let axis_coord = axis_coord as u32;
        let bit = 1u32 << axis_coord;
        let u = area_coords.u as usize;
        let v = area_coords.v as usize;

        for u in u..u + side {
            for v in v..v + side {
                target[u * PLANE_SIDE + v] |= bit;
            }
        }
    }

    #[inline(always)]
    pub fn get_layer(&self, axis: Axis, layer: usize) -> &[u32] {
        self.masks[axis.as_usize()].get_layer(layer)
    }
}
