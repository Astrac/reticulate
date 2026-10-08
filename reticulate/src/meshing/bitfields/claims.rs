use crate::{Coords2D, PLANE_SIDE, meshing::surface_layer::SurfaceLayer};

/// Per-axis claim storage: plane `axis_coord` is PLANE_SIDE words, where
/// word u is row u and bit v is column v. This is the same layout as
/// StripWeaver::filled and Parity::gather_footprint.
#[repr(transparent)]
pub struct ClaimsMask<'a>(&'a mut [[u32; PLANE_SIDE]; PLANE_SIDE]);

impl<'a> ClaimsMask<'a> {
    #[inline(always)]
    pub fn from_u64_slice(data: &'a mut [u64]) -> ClaimsMask<'a> {
        let layers: &mut [[u32; PLANE_SIDE]] = bytemuck::cast_slice_mut(data);

        ClaimsMask(
            layers
                .try_into()
                .expect("Unexpected size for claims layers"),
        )
    }

    #[inline(always)]
    fn plane(&self, index: usize) -> &[u32; PLANE_SIDE] {
        &self.0[index & (PLANE_SIDE - 1)]
    }

    #[inline(always)]
    fn plane_mut(&mut self, index: usize) -> &mut [u32; PLANE_SIDE] {
        &mut self.0[index & (PLANE_SIDE - 1)]
    }
}

/// Tracks which face cells have already been emitted.
pub struct Claims<'a> {
    pub masks: [ClaimsMask<'a>; 3],
}

impl<'a> Claims<'a> {
    /// Adds `shape` (row u, bits over v) to the claimed cells of this plane.
    #[inline(always)]
    pub fn claim(&mut self, layer: SurfaceLayer, shape: &[u32; PLANE_SIDE]) {
        let plane = self.masks[layer.axis.as_usize()].plane_mut(layer.index as usize);
        for (word, &row) in plane.iter_mut().zip(shape.iter()) {
            debug_assert!(
                *word & row == 0,
                "cells claimed twice: {:032b}",
                *word & row
            );
            *word |= row;
        }
    }

    /// Claimed cells of row u, bits over v. A single word read, for step 2's guard.
    #[inline(always)]
    pub fn claimed_row(&self, layer: SurfaceLayer, u: usize) -> u32 {
        self.masks[layer.axis.as_usize()].plane(layer.index as usize)[u]
    }

    #[inline(always)]
    pub fn is_unclaimed(&self, layer: SurfaceLayer, uv: Coords2D) -> bool {
        (self.claimed_row(layer, uv.u as usize) >> uv.v) & 1 == 0
    }
}
