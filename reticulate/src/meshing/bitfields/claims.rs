use crate::{Coords2D, axis::*, meshing::bitfields::PLANE_SIDE};

/// Per-axis claim storage: plane `axis_coord` is PLANE_SIDE words, where
/// word u is row u and bit v is column v. This is the same layout as
/// StripWeaver::filled and Parity::gather_footprint.
#[repr(transparent)]
pub struct ClaimsMask<'a>(&'a mut [u32]);

impl<'a> ClaimsMask<'a> {
    #[inline(always)]
    pub fn from_u64_slice(data: &'a mut [u64]) -> ClaimsMask<'a> {
        ClaimsMask(bytemuck::cast_slice_mut(data))
    }

    #[inline(always)]
    fn plane(&self, axis_coord: usize) -> &[u32] {
        let i = axis_coord * PLANE_SIDE;
        &self.0[i..i + PLANE_SIDE]
    }

    #[inline(always)]
    fn plane_mut(&mut self, axis_coord: usize) -> &mut [u32] {
        let i = axis_coord * PLANE_SIDE;
        &mut self.0[i..i + PLANE_SIDE]
    }
}

/// Tracks which face cells have already been emitted.
pub struct Claims<'a> {
    pub masks: [ClaimsMask<'a>; 3],
}

impl<'a> Claims<'a> {
    /// Adds `shape` (row u, bits over v) to the claimed cells of this plane.
    pub fn mark_done(&mut self, axis: Axis, axis_coord: usize, shape: &[u32; PLANE_SIDE]) {
        let plane = self.masks[axis.as_usize()].plane_mut(axis_coord);
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
    pub fn claimed_row(&self, axis: Axis, axis_coord: usize, u: usize) -> u32 {
        self.masks[axis.as_usize()].plane(axis_coord)[u]
    }

    #[inline(always)]
    pub fn is_pending(&self, axis: Axis, axis_coord: usize, uv: Coords2D) -> bool {
        (self.claimed_row(axis, axis_coord, uv.u as usize) >> uv.v) & 1 == 0
    }
}
