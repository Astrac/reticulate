use crate::{
    Coords2D,
    axis::*,
    meshing::{bitfields::PLANE_SIDE, surface_layer::SurfaceLayer},
};

#[repr(transparent)]
#[derive(Debug)]
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
///
/// This is the indexing structure for each of the masks:
///
/// Mask[X]: Planes indexed by Y, edges by Z, bits by X
/// Mask[Y]: Planes indexed by Z, edges by X, bits by Y
/// Mask[Z]: Planes indexed by X, edges by Y, bits by Z
#[repr(transparent)]
pub struct Parity<'a> {
    pub masks: [ParityMask<'a>; 3],
}

impl<'a> Parity<'a> {
    #[inline(always)]
    pub fn set_transition(&mut self, layer: SurfaceLayer, area_coords: Coords2D, area_size: usize) {
        let target = self.masks[layer.axis.as_usize()].0.as_mut();
        let side = area_size.isqrt();
        let axis_coord = layer.layer as u32;
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
    pub fn gather_edges(&self, layer: SurfaceLayer) -> &[u32] {
        self.masks[layer.axis.as_usize()].get_layer(layer.layer as usize)
    }

    // TODO - Too many usizes here, this is error-prone, find a better way
    #[inline(always)]
    pub fn gather_guard(
        &self,
        axis: Axis,
        v: usize,
        u: usize,
        axis_coord_start: usize,
        axis_coord_end: usize,
    ) -> u32 {
        let mut guard = 0u32;
        for axis_coord in axis_coord_start..axis_coord_end {
            let word = self.masks[axis.as_usize()].get_layer(axis_coord)[v];
            guard |= ((word >> u) & 1) << axis_coord;
        }
        guard
    }

    #[inline(always)]
    pub fn boundary_guard(&self, layer: SurfaceLayer, boundary: usize, mask: u32) -> u32 {
        let columns_axis = layer.axis.next();

        let start = mask.trailing_zeros() as usize;
        let end = (32 - mask.leading_zeros()) as usize;

        let mut g = self.gather_guard(columns_axis, layer.layer as usize, boundary, start, end);
        if layer.layer > 0 {
            g |= self.gather_guard(columns_axis, layer.layer as usize - 1, boundary, start, end);
        }

        g & mask
    }
}
