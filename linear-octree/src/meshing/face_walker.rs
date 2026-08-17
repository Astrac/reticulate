use bitvec::prelude::*;

use crate::{
    Coords3D, Morton3D, Walker,
    axis::*,
    coords::Coords2D,
    face::{FACE_X_N, FACE_X_P, Face},
    meshing::{
        BITFIELD_FACE_FULL_SIZE,
        bitfield::{Bitfield, face_size},
        routing::Routing,
    },
    morton_2d::Morton2D,
};

#[allow(dead_code)]
pub struct FaceWalker<F, Data> {
    on_face: F,
    bitfield: Bitfield,
    coalescing_stack: [CoalesceState<Data>; 7],
    routing_stack: [Routing; 7],
    scratchpad: BitArr!(for BITFIELD_FACE_FULL_SIZE, in u64, Lsb0),
}

#[derive(Clone, Copy, PartialEq)]
enum CoalesceState<Data> {
    Empty,
    Homogeneous(Data),
    Heterogeneous,
}

impl<F, Data> FaceWalker<F, Data>
where
    Data: Copy + PartialEq,
{
    pub fn new(f: F) -> FaceWalker<F, Data> {
        FaceWalker {
            on_face: f,
            bitfield: Bitfield::default(),
            coalescing_stack: [CoalesceState::Empty; 7],
            routing_stack: [Routing::default(); 7],
            scratchpad: Default::default(),
        }
    }
}

impl<Data: PartialEq + Copy, F> Walker<Data> for FaceWalker<F, Data>
where
    F: FnMut(Morton3D, u8, Face),
{
    fn on_branch_start(&mut self, depth: u8, morton: Morton3D, _children_mask: u8) {
        let octant = morton.as_u32() & 0b111;
        self.routing_stack[depth as usize] = self.bitfield.routing;
        self.bitfield.routing = self.bitfield.routing.focus_octant(octant as u8, depth);

        self.bitfield
            .face_slice_mut(AXIS_X, |r| r.inn_pos_x[(depth as usize % 3)], morton, depth)
            .fill(false);
        self.bitfield
            .face_slice_mut(AXIS_X, |r| r.inn_neg_x[(depth as usize % 3)], morton, depth)
            .fill(false);
    }

    fn on_branch_end(&mut self, depth: u8, morton: Morton3D) {
        let ring_idx = depth as usize % 3;

        // 1. Fetch our local child-tier slices through our current active routing handles
        let inn_pos_x = self
            .bitfield
            .face_slice(AXIS_X, |r| r.inn_pos_x[ring_idx], morton, depth);
        let inn_neg_x = self
            .bitfield
            .face_slice(AXIS_X, |r| r.inn_neg_x[ring_idx], morton, depth);

        let branch_size = inn_pos_x.len();
        let parent_routing = self.routing_stack[depth as usize];

        // 2. Local Sibling Parity Pass: XOR the positive and negative lanes directly
        (&mut self.scratchpad[0..branch_size]).copy_from_bitslice(inn_pos_x);
        self.scratchpad ^= inn_neg_x;
        let culled_bits = &mut self.scratchpad[0..branch_size];

        // Early-out gate: if everything cancelled out, clean local work window and pop stack
        if culled_bits.not_any() {
            self.bitfield
                .face_slice_mut(AXIS_X, |r| r.inn_pos_x[ring_idx], morton, depth)
                .fill(false);
            self.bitfield
                .face_slice_mut(AXIS_X, |r| r.inn_neg_x[ring_idx], morton, depth)
                .fill(false);
            self.bitfield.routing = parent_routing;
            return;
        }

        let max_face_size = branch_size / 4;
        let mut current = 0;

        // Stride initializes safely bounded by full resolution units (minimum of 1)
        let mut stride = max_face_size.max(1);

        // 3. Perfect Stride Emission Loop
        while current < branch_size {
            let block_end = branch_size.min(current + stride);
            let chunk = &culled_bits[current..block_end];

            // Single bit strides are automatically uniform, otherwise check for complete block coherence
            let is_uniform_macro_chunk = stride == 1 || chunk.all() || chunk.not_any();

            if is_uniform_macro_chunk {
                if chunk.all() {
                    // Safe unsigned shift math: guaranteed never to underflow or panic
                    let stride_depth_offset = if max_face_size > 0 {
                        (max_face_size.trailing_zeros() - stride.trailing_zeros()) / 2
                    } else {
                        0
                    };

                    let face_depth = (depth + 1) + (stride_depth_offset as u8);
                    let block_end = current + stride;

                    if culled_bits[current..block_end].any() {
                        if inn_pos_x[current] {
                            let face_morton = compute_face_voxel_morton(
                                current, stride, AXIS_X, false, morton, depth,
                            );
                            (self.on_face)(face_morton, face_depth, FACE_X_N);
                        } else {
                            let face_morton = compute_face_voxel_morton(
                                current, stride, AXIS_X, true, morton, depth,
                            );
                            (self.on_face)(face_morton, face_depth, FACE_X_P);
                        }
                    }
                }

                // Move cursor forward by the full macro-step width and reset stride
                current = block_end;
                stride = max_face_size.max(1);
            } else {
                // Mixed macro chunk: subdivide search window by 4
                stride /= 4;
            }
        }

        if depth > 0 {
            let parent_depth = depth - 1;
            let parent_ring_idx = parent_depth as usize % 3;

            // 1. Determine whether this child branch sits on the positive or negative side of the parent line
            let morton_raw = morton.as_u32();
            let is_child_pos = (morton_raw & 1) != 0; // Axis(1) is AXIS_X, check bit 0

            // 2. Extract the raw integer base address handle directly from the stack-anchored parent routing frame.
            // This avoids dynamic closures entirely, making it 100% type-safe!
            let parent_channel_base = if is_child_pos {
                parent_routing.inn_pos_x[parent_ring_idx]
            } else {
                parent_routing.inn_neg_x[parent_ring_idx]
            };

            // 3. Compute the absolute manual memory offset matching your face_slice logic
            let parent_face_size = face_size(parent_depth);
            let absolute_2d_key = morton.drop_axis(AXIS_X) as usize;
            let parent_target_idx = parent_channel_base + (parent_face_size * absolute_2d_key);

            // 4. Merge the surviving bits upward directly through the raw self.storage array
            let parent_target_slice =
                &mut self.bitfield.storage[parent_target_idx..parent_target_idx + branch_size];
            for (mut dest_bit, mask_bit) in parent_target_slice.iter_mut().zip(culled_bits.iter()) {
                if *mask_bit {
                    *dest_bit = true;
                }
            }
        }

        // 4. Safe Local Canvas Cleanse: Wipe out our local footprints from this specific depth ring
        self.bitfield
            .face_slice_mut(AXIS_X, |r| r.inn_pos_x[ring_idx], morton, depth)
            .fill(false);
        self.bitfield
            .face_slice_mut(AXIS_X, |r| r.inn_neg_x[ring_idx], morton, depth)
            .fill(false);

        // 5. Pop stack context safely at the exit gate
        self.bitfield.routing = parent_routing;
    }

    // fn on_branch_end(&mut self, depth: u8, morton: Morton3D) {
    //     self.bitfield.routing = self.routing_stack[depth as usize];
    //
    //     let inn_pos_x =
    //         self.bitfield
    //             .face_slice(AXIS_X, |r| r.inn_pos_x[(depth as usize % 3)], morton, depth);
    //     let inn_neg_x =
    //         self.bitfield
    //             .face_slice(AXIS_X, |r| r.inn_neg_x[(depth as usize % 3)], morton, depth);
    //
    //     if depth <= 5 {
    //         let face_size = face_size(depth);
    //         let absolute_2d_key = morton.drop_axis(AXIS_X) as usize;
    //         let offset = face_size * absolute_2d_key;
    //         println!("--- CULLING FRAME depth={} morton={:?} ---", depth, morton);
    //         println!(
    //             "  ROUTING HANDLES: inn_pos={}, inn_neg={}, out_pos={}, out_neg={}",
    //             self.bitfield.routing.inn_pos_x[(depth as usize) % 3],
    //             self.bitfield.routing.inn_neg_x[(depth as usize) % 3],
    //             self.bitfield.routing.out_pos_x,
    //             self.bitfield.routing.out_neg_x,
    //         );
    //         println!(
    //             "  ABSOLUTE INDICES: inn_pos_idx={}, inn_neg_idx={}",
    //             self.bitfield.routing.inn_pos_x[(depth as usize) % 3] + offset,
    //             self.bitfield.routing.inn_neg_x[(depth as usize) % 3] + offset
    //         );
    //         // println!(
    //         //     "  REGISTER DATA: POS_BITS={:b}, NEG_BITS={:b}",
    //         //     inn_pos_x, inn_neg_x
    //         // );
    //     }
    //
    //     let branch_size = inn_pos_x.len();
    //     (&mut self.scratchpad[0..branch_size]).copy_from_bitslice(inn_pos_x);
    //     self.scratchpad ^= inn_neg_x;
    //     let culled_bits = &mut self.scratchpad[0..branch_size];
    //
    //     if culled_bits.not_any() {
    //         self.bitfield
    //             .face_slice_mut(AXIS_X, |r| r.inn_pos_x[(depth as usize) % 3], morton, depth)
    //             .fill(false);
    //         self.bitfield
    //             .face_slice_mut(AXIS_X, |r| r.inn_neg_x[(depth as usize) % 3], morton, depth)
    //             .fill(false);
    //         self.bitfield.routing = self.routing_stack[depth as usize];
    //         return;
    //     }
    //
    //     let local_offset = face_size(depth) * morton.drop_axis(AXIS_X) as usize;
    //     let max_face_size = branch_size / 4;
    //     let mut current = local_offset;
    //     let mut stride = max_face_size;
    //     let mut cleans_pos = Vec::new();
    //     let mut cleans_neg = Vec::new();
    //
    //     while current < branch_size {
    //         let current_block_max = branch_size.min(current + stride);
    //         if !culled_bits[current..current_block_max].any() {
    //             current += stride;
    //             stride = max_face_size; // Reset to try macro-scale at the next clean boundary
    //             continue;
    //         }
    //
    //         let block_end = current + stride;
    //         let is_hom_pos = inn_pos_x[0];
    //         let is_hom_neg = inn_neg_x[0];
    //         // let is_uniform_macro_face =
    //         //     culled_bits[current..block_end].all() || culled_bits[current..block_end].not_any();
    //
    //         if (is_hom_pos && !is_hom_neg) || (is_hom_neg && !is_hom_pos) || (stride == 1) {
    //             let stride_depth_offset =
    //                 (max_face_size.trailing_zeros() - stride.trailing_zeros()) / 2;
    //             let face_depth = (depth + 1) + (stride_depth_offset as u8);
    //
    //             if culled_bits[current..block_end].any() {
    //                 if is_hom_pos {
    //                     let face_morton = compute_face_voxel_morton(
    //                         current, stride, AXIS_X, false, morton, depth,
    //                     );
    //                     (self.on_face)(face_morton, face_depth, FACE_X_N);
    //                     cleans_pos.push(current..block_end);
    //                 } else {
    //                     let face_morton =
    //                         compute_face_voxel_morton(current, stride, AXIS_X, true, morton, depth);
    //                     (self.on_face)(face_morton, face_depth, FACE_X_P);
    //                     cleans_neg.push(current..block_end);
    //                 }
    //             }
    //
    //             culled_bits[current..block_end].fill(false);
    //             current = block_end;
    //             stride = max_face_size.max(1);
    //         } else {
    //             stride /= 4;
    //             if stride == 0 {
    //                 current += 1;
    //                 stride = max_face_size;
    //             }
    //         }
    //     }
    //
    //     // for range in cleans_neg {
    //     //     self.bitfield
    //     //         .face_slice_mut(AXIS_X, |r| r.inn_pos_x, morton, depth)[range.clone()]
    //     //         .fill(false);
    //     //     self.bitfield
    //     //         .face_slice_mut(AXIS_X, |r| r.inn_neg_x, morton, depth)[range]
    //     //         .fill(false);
    //     // }
    //     // for range in cleans_pos {
    //     //     self.bitfield
    //     //         .face_slice_mut(AXIS_X, |r| r.inn_pos_x, morton, depth)[range.clone()]
    //     //         .fill(false);
    //     //     self.bitfield
    //     //         .face_slice_mut(AXIS_X, |r| r.inn_neg_x, morton, depth)[range]
    //     //         .fill(false);
    //     // }
    //
    //     // self.bitfield
    //     //     .propagate_and_clear(AXIS_X, morton, depth, culled_bits, &parent_routing);
    //     // .propagate_boundaries(AXIS_X, morton, depth, culled_bits, &parent_routing);
    //     // self.bitfield
    //     //     .face_slice_mut(AXIS_X, |r| r.inn_pos_x, morton, depth)
    //     //     .fill(false);
    //     // self.bitfield
    //     //     .face_slice_mut(AXIS_X, |r| r.inn_neg_x, morton, depth)
    //     //     .fill(false);
    //
    //     let inn_pos_mut = self.bitfield.face_slice_mut(
    //         AXIS_X,
    //         |r| r.inn_pos_x[(depth as usize % 3)],
    //         morton,
    //         depth,
    //     );
    //     for (mut dest_bit, mask_bit) in inn_pos_mut.iter_mut().zip(culled_bits.iter()) {
    //         // If a bit was culled or emitted (culled_bits is true), we wipe it.
    //         // If it was a background macro-boundary, we preserve its state!
    //         if *mask_bit {
    //             *dest_bit = false;
    //         }
    //     }
    //
    //     let inn_neg_mut = self.bitfield.face_slice_mut(
    //         AXIS_X,
    //         |r| r.inn_neg_x[(depth as usize % 3)],
    //         morton,
    //         depth,
    //     );
    //     for (mut dest_bit, mask_bit) in inn_neg_mut.iter_mut().zip(culled_bits.iter()) {
    //         if *mask_bit {
    //             *dest_bit = false;
    //         }
    //     }
    // }

    fn on_leaf(&mut self, depth: u8, morton: Morton3D, data: &Data) {
        let octant = morton.as_u32() & 0b111;

        if octant & AXIS_X.as_u8() as u32 == 0 {
            self.bitfield
                .face_slice_mut(AXIS_X, |r| r.inn_neg_x[depth as usize % 3], morton, depth)
                .fill(true);
            self.bitfield
                .face_slice_mut(AXIS_X, |r| r.out_neg_x, morton, depth)
                .fill(true);
        } else {
            self.bitfield
                .face_slice_mut(AXIS_X, |r| r.inn_pos_x[depth as usize % 3], morton, depth)
                .fill(true);
            self.bitfield
                .face_slice_mut(AXIS_X, |r| r.out_pos_x, morton, depth)
                .fill(true);
        }

        self.coalescing_stack[depth as usize] = match self.coalescing_stack[depth as usize] {
            CoalesceState::Empty => CoalesceState::Homogeneous(*data),
            CoalesceState::Homogeneous(d) => {
                if d == *data {
                    CoalesceState::Homogeneous(d)
                } else {
                    CoalesceState::Heterogeneous
                }
            }
            CoalesceState::Heterogeneous => CoalesceState::Heterogeneous,
        }
    }
}

#[inline(always)]
pub fn compute_face_voxel_morton(
    current: usize,
    face_size: usize,
    axis: Axis,
    is_pos: bool, // true if the positive side is solid matter, false if negative
    parent_morton: Morton3D,
    depth: u8,
) -> Morton3D {
    let parent_coords = Coords3D::from(parent_morton);
    let local_coords = Coords2D::from(Morton2D::from(current as u32));

    let side_length_shift = face_size.trailing_zeros() / 2;
    let block_side = 1 << side_length_shift;
    let child_unit_width = 1 << (6 - (depth + 1));
    let seam_offset = child_unit_width;
    let ownership_offset = if is_pos { block_side } else { 0 };
    let axis_offset = seam_offset - ownership_offset;

    let (dx, dy, dz) = match axis {
        AXIS_X => (
            axis_offset,
            local_coords.u * block_side,
            local_coords.v * block_side,
        ),
        AXIS_Y => (
            local_coords.u * block_side,
            axis_offset,
            local_coords.v * block_side,
        ),
        AXIS_Z => (
            local_coords.u * block_side,
            local_coords.v * block_side,
            axis_offset,
        ),
        _ => unreachable!(),
    };

    let resolution_scale_shift = 6 - depth;

    let absolute_parent_x = parent_coords.x << resolution_scale_shift;
    let absolute_parent_y = parent_coords.y << resolution_scale_shift;
    let absolute_parent_z = parent_coords.z << resolution_scale_shift;

    let parent_mask = !((1 << (6 - depth)) - 1);
    let final_x = (absolute_parent_x & parent_mask) + dx as u8;
    let final_y = (absolute_parent_y & parent_mask) + dy as u8;
    let final_z = (absolute_parent_z & parent_mask) + dz as u8;

    println!(
        "current={current}, face_size={face_size}, axis={axis:?}, is_pos={is_pos}, parent_morton={parent_morton:?}, depth={depth}, coords: {}, {}, {}",
        final_x, final_y, final_z
    );

    Morton3D::from(Coords3D {
        x: final_x,
        y: final_y,
        z: final_z,
    })
}
