use bitvec::prelude::*;

use crate::{Coords3D, Depth, MeshFace, axis::*};

/// 33 x 33 lines per plane, each divided in 32 segments.
/// Each segment represents a depth-5 edge of the octree.
/// Each segment has values:
///   - 00 Not drawn
///   - 11 Solid
///   - 01 Subdivided
const EDGE_COUNT: usize = 33 * 33;
const EDGE_SIZE: usize = 33;
const EDGE_MASKS_SIZE: usize = EDGE_SIZE * EDGE_COUNT * 2;
const BITFIELD_SIZE: usize = EDGE_MASKS_SIZE * 3;

const U_MAX: u32 = 16 * 2;
const U_MIN: u32 = 10 * 2;
const EDGE_SPAN_MASK: u32 = ((1u32 << (U_MAX >> 1)) - 1) ^ ((1u32 << (U_MIN >> 1)) - 1);

#[inline(always)]
const fn edge_index(axis: Axis, plane: usize, u: usize) -> usize {
    (axis.as_index() * EDGE_MASKS_SIZE) + (((33 * plane) + u) * EDGE_SIZE * 2)
}

#[inline(always)]
const fn subdivision_index(axis: Axis, plane: usize, u: usize) -> usize {
    edge_index(axis, plane, u) + EDGE_SIZE
}

#[inline(always)]
const fn face_uv_rect(face: MeshFace) -> (Axis, usize, usize, Axis, usize, usize) {
    let MeshFace {
        coords: Coords3D { x, y, z },
        face,
        width,
        height,
        ..
    } = face;

    let face_axis = face.axis();

    match face_axis {
        AXIS_X => {
            let u = (y >> 1) as usize;
            let v = (z >> 1) as usize;
            (AXIS_Y, u, u + (width >> 1), AXIS_Z, v, v + (height >> 1))
        }
        AXIS_Y => {
            let u = (x >> 1) as usize;
            let v = (z >> 1) as usize;
            (AXIS_X, u, u + (width >> 1), AXIS_Z, v, v + (height >> 1))
        }
        AXIS_Z => {
            let u = (x >> 1) as usize;
            let v = (y >> 1) as usize;
            (AXIS_X, u, u + (width >> 1), AXIS_Y, v, v + (height >> 1))
        }
        _ => unreachable!(),
    }
}

#[inline(always)]
const fn face_edge_indices(face: MeshFace) -> [usize; 4] {
    let (axis_u, u_min, u_max, axis_v, v_min, v_max) = face_uv_rect(face);

    [
        edge_index(axis_u, u_min, v_min),
        edge_index(axis_u, u_min, v_max),
        edge_index(axis_v, u_min, v_min),
        edge_index(axis_v, u_max, v_min),
    ]
}

#[inline(always)]
const fn face_subdivision_indices(face: MeshFace) -> [usize; 4] {
    let (axis_u, u_min, u_max, axis_v, v_min, v_max) = face_uv_rect(face);

    [
        subdivision_index(axis_u, u_min, v_min),
        subdivision_index(axis_u, u_min, v_max),
        subdivision_index(axis_v, u_min, v_min),
        subdivision_index(axis_v, u_max, v_min),
    ]
}

pub struct EdgeRegistry {
    edges_bitfield: BitArr!(for BITFIELD_SIZE, in u64, Lsb0),
}

impl EdgeRegistry {
    #[inline(always)]
    pub fn edge_mut(&mut self, axis: Axis, u: usize, v: usize) -> &mut BitSlice<u64> {
        let idx = edge_index(axis, u, v);
        &mut self.edges_bitfield[idx..idx + EDGE_SIZE]
    }

    #[inline(always)]
    pub fn subdivision_mut(&mut self, axis: Axis, u: usize, v: usize) -> &mut BitSlice<u64> {
        let idx = subdivision_index(axis, u, v);
        &mut self.edges_bitfield[idx..idx + EDGE_SIZE]
    }

    #[inline(always)]
    pub fn write_face(&mut self, face: MeshFace) {
        match face.depth {
            Depth(6) => {
                // Set subdivision
            }
            Depth(d) => {
                // let edge_span_mask = ((1u32 << (u_max >> 1)) - 1) ^ ((1u32 << (u_min >> 1)) - 1);
                // Set edge
            }
        }
    }
}
