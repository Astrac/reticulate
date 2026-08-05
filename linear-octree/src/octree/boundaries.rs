use bitvec::prelude::*;

use crate::Morton;

pub const BOUNDARY_OUTER_NEGATIVE_X: u8 = 0b00001;
pub const BOUNDARY_OUTER_NEGATIVE_Y: u8 = 0b00010;
pub const BOUNDARY_OUTER_NEGATIVE_Z: u8 = 0b00100;
pub const BOUNDARY_OUTER_POSITIVE_X: u8 = 0b01001;
pub const BOUNDARY_OUTER_POSITIVE_Y: u8 = 0b01010;
pub const BOUNDARY_OUTER_POSITIVE_Z: u8 = 0b01100;
pub const BOUNDARY_INNER_X: u8 = 0b10001;
pub const BOUNDARY_INNER_Y: u8 = 0b10010;
pub const BOUNDARY_INNER_Z: u8 = 0b10100;

#[allow(dead_code)]
#[derive(Clone, Default, Debug)]
struct TraversalBoundaries {
    masks: BitArr!(for 9 * (64 * 64), in u64, Lsb0),
    depth: u8,
}

const MASK_SIZE_BITS: usize = 64 * 64;
const MASK_SIZE_BLOCKS: usize = 64;

#[allow(dead_code, unused)]
impl TraversalBoundaries {
    const INNER_BOUNDARIES: [u8; 3] = [BOUNDARY_INNER_X, BOUNDARY_INNER_Y, BOUNDARY_INNER_Z];

    pub fn fill_octant(&mut self, morton: Morton, depth: u8, octant: u8) {
        use_all_boundary_projections(morton, self.depth, depth, |boundary, projection| {
            let projection: usize = projection as usize;
            let depth: usize = depth as usize;

            // Slice the mask of the boundary to the depth*depth submask at morton index
            // given by projection
            let idx = Self::boundary_block_index(boundary) + (projection * depth * depth);
            let slice = &mut self.masks[idx..(idx + MASK_SIZE_BITS)];

            !slice;
        });
    }

    // Outer X -> Outer X
    // Inner X -> Outer X (P if X == 0, N if X == 1)
    pub fn fork_octant(&self, octant: u8) -> TraversalBoundaries {
        let mut forked = TraversalBoundaries::default();
        forked.depth = self.depth + 1;

        for boundary in Self::octant_outer_boundaries(octant) {
            let idx = Self::boundary_bit_index(boundary);
            forked.masks[idx..(idx + MASK_SIZE_BITS)]
                .copy_from_bitslice(&self.masks[idx..(idx + MASK_SIZE_BITS)]);
        }

        for boundary in Self::INNER_BOUNDARIES {
            let inner_idx = Self::boundary_bit_index(boundary);
            let outer_idx =
                Self::boundary_bit_index(Self::outer_boundary_matching_inner(octant, boundary));
            forked.masks[outer_idx..(outer_idx + MASK_SIZE_BITS)]
                .copy_from_bitslice(&self.masks[inner_idx..(inner_idx + MASK_SIZE_BITS)]);
        }

        forked
    }

    pub fn join_octant(&mut self, octant: u8, boundaries: &TraversalBoundaries) {
        for boundary in Self::octant_outer_boundaries(octant) {
            let idx = Self::boundary_bit_index(boundary);
            self.masks[idx..(idx + MASK_SIZE_BITS)]
                .copy_from_bitslice(&boundaries.masks[idx..(idx + MASK_SIZE_BITS)]);
        }

        for boundary in Self::INNER_BOUNDARIES {
            let inner_idx = Self::boundary_bit_index(boundary);
            let outer_idx =
                Self::boundary_bit_index(Self::outer_boundary_matching_inner(octant, boundary));
            self.masks[inner_idx..(inner_idx + MASK_SIZE_BITS)]
                .copy_from_bitslice(&boundaries.masks[outer_idx..(outer_idx + MASK_SIZE_BITS)]);
        }
    }

    #[inline(always)]
    const fn boundary_block_index(boundary: u8) -> usize {
        let face = boundary as usize;
        ((3 * (face >> 3)) + ((face & 0b111) >> 1)) * MASK_SIZE_BLOCKS
    }

    #[inline(always)]
    const fn boundary_bit_index(boundary: u8) -> usize {
        let face = boundary as usize;
        ((3 * (face >> 3)) + ((face & 0b111) >> 1)) * MASK_SIZE_BITS
    }

    #[inline(always)]
    const fn outer_boundary_matching_inner(octant: u8, boundary: u8) -> u8 {
        ((((boundary & octant) == 0) as u8) << 3) | (boundary & 0b011111)
    }

    #[inline(always)]
    const fn octant_outer_boundaries(octant: u8) -> [u8; 3] {
        // Isolate the coordinate bits
        let x_bit = octant & 1;
        let y_bit = (octant >> 1) & 1;
        let z_bit = (octant >> 2) & 1;

        // If the coordinate of the octant is positive, the outer axis is positive
        // The outer boundary has format 0b0PZYX, where ZYX is a bitmask of the axes
        // and P says whether boundary is positive or negative.
        [
            (x_bit << 3) | 0b00001,
            (y_bit << 3) | 0b00010,
            (z_bit << 3) | 0b00100,
        ]
    }
}

pub fn get_2d_outer_projections(morton: u32, depth: u8) -> [u16; 3] {
    let total_bits = depth * 3;
    let active_path = morton >> (18 - total_bits);

    // Clear path
    let depth_bitmask = (1u32 << total_bits) - 1;
    let path = active_path & depth_bitmask;

    // Z-Plane (zyxzyx -> 0yx0yx)
    let xy_x = path & 0x92492493;
    let xy_y = path & 0x24924924;

    // Z-Plane (0yx0yx -> yxyx)
    let mut xy = (xy_x | (xy_x >> 1)) & 0x000c30c3;
    xy = (xy | (xy >> 2)) & 0x00f00f00;
    xy = (xy | (xy >> 4)) & 0x03f0003f;
    let xy_bits = (xy | (xy_y >> 1)) as u16;

    // Y-Plane (zyxzyx -> z0xz0x)
    let xz_x = path & 0x92492493;
    let xz_z = path & 0x49249249;

    // Y-Plane (zyxzyx -> zxzx)
    let mut xz = (xz_x | (xz_x >> 1)) & 0x000c30c3;
    xz = (xz | (xz >> 2)) & 0x00f00f00;
    xz = (xz | (xz >> 4)) & 0x03f0003f;
    let xz_bits = (xz | (xz_z >> 2)) as u16;

    // X-Plane (zyxzyx -> zy0zy0)
    let yz_y = path & 0x24924924;
    let yz_z = path & 0x49249249;

    // X-Plane (zyxzyx -> zyzy)
    let mut yz = (yz_y | (yz_y >> 1)) & 0x000c30c3;
    yz = (yz | (yz >> 2)) & 0x00f00f00;
    yz = (yz | (yz >> 4)) & 0x03f0003f;
    let yz_bits = ((yz >> 1) | (yz_z >> 2)) as u16;

    // Padding 0s followed by depth * 2 1s (0000...11)
    let depth_bounding = ((1u16 << (depth * 2)) - 1) as u16;

    [
        xy_bits & depth_bounding,
        xz_bits & depth_bounding,
        yz_bits & depth_bounding,
    ]
}

const BOUNDARIES_OUTER: [u8; 6] = [
    BOUNDARY_OUTER_NEGATIVE_X,
    BOUNDARY_OUTER_NEGATIVE_Y,
    BOUNDARY_OUTER_NEGATIVE_Z,
    BOUNDARY_OUTER_POSITIVE_X,
    BOUNDARY_OUTER_POSITIVE_Y,
    BOUNDARY_OUTER_POSITIVE_Z,
];

pub fn use_all_boundary_projections(
    leaf_morton: Morton,
    current_depth: u8,
    depth: u8,
    mut f: impl FnMut(u8, u16),
) {
    let leaf_morton = leaf_morton.as_u32();

    if current_depth == 0 {
        if depth == 0 {
            for boundary in BOUNDARIES_OUTER {
                f(boundary, 0);
            }
        }
        return;
    }

    let total_bits_ancestor = current_depth * 3;
    let active_path_ancestor = leaf_morton >> (18 - total_bits_ancestor);
    let ancestor_bitmask = (1u32 << total_bits_ancestor) - 1;

    let x_mask_anc = 0x92492493u32 & ancestor_bitmask;
    let y_mask_anc = 0x24924924u32 & ancestor_bitmask;
    let z_mask_anc = 0x49249249u32 & ancestor_bitmask;

    let outer_boundary_mask = ((((active_path_ancestor & x_mask_anc) == 0) as u8) << 0)
        | ((((active_path_ancestor & x_mask_anc) == x_mask_anc) as u8) << 1)
        | ((((active_path_ancestor & y_mask_anc) == 0) as u8) << 2)
        | ((((active_path_ancestor & y_mask_anc) == y_mask_anc) as u8) << 3)
        | ((((active_path_ancestor & z_mask_anc) == 0) as u8) << 4)
        | ((((active_path_ancestor & z_mask_anc) == z_mask_anc) as u8) << 5);

    let [z_projection, y_projection, x_projection] = get_2d_outer_projections(leaf_morton, depth);

    if outer_boundary_mask != 0 {
        if (outer_boundary_mask & (1 << 0)) != 0 {
            f(BOUNDARY_OUTER_NEGATIVE_X, x_projection);
        }
        if (outer_boundary_mask & (1 << 1)) != 0 {
            f(BOUNDARY_OUTER_POSITIVE_X, x_projection);
        }
        if (outer_boundary_mask & (1 << 2)) != 0 {
            f(BOUNDARY_OUTER_NEGATIVE_Y, y_projection);
        }
        if (outer_boundary_mask & (1 << 3)) != 0 {
            f(BOUNDARY_OUTER_POSITIVE_Y, y_projection);
        }
        if (outer_boundary_mask & (1 << 4)) != 0 {
            f(BOUNDARY_OUTER_NEGATIVE_Z, z_projection);
        }
        if (outer_boundary_mask & (1 << 5)) != 0 {
            f(BOUNDARY_OUTER_POSITIVE_Z, z_projection);
        }
    }

    if depth <= current_depth {
        return;
    }

    let total_bits_descendant = depth * 3;
    let segment_shift = 18 - total_bits_descendant;
    let segment_path = leaf_morton >> segment_shift;

    let segment_levels = depth - current_depth;
    let segment_bitmask = (1u32 << (segment_levels * 3)) - 1;
    let path_segment = segment_path & segment_bitmask;

    let x_mask_seg = 0x92492493u32 & segment_bitmask;
    let y_mask_seg = 0x24924924u32 & segment_bitmask;
    let z_mask_seg = 0x49249249u32 & segment_bitmask;

    let first_bit_shift = (segment_levels - 1) * 3;
    let first_bit_x = (path_segment >> first_bit_shift) & 1;
    let first_bit_y = (path_segment >> (first_bit_shift + 1)) & 1;
    let first_bit_z = (path_segment >> (first_bit_shift + 2)) & 1;

    let remaining_x_bits = path_segment & (x_mask_seg >> 3);
    let touches_inner_x = (first_bit_x == 0 && remaining_x_bits == (x_mask_seg >> 3))
        || (first_bit_x == 1 && remaining_x_bits == 0);

    let remaining_y_bits = path_segment & (y_mask_seg >> 3);
    let touches_inner_y = (first_bit_y == 0 && remaining_y_bits == (y_mask_seg >> 3))
        || (first_bit_y == 1 && remaining_y_bits == 0);

    let remaining_z_bits = path_segment & (z_mask_seg >> 3);
    let touches_inner_z = (first_bit_z == 0 && remaining_z_bits == (z_mask_seg >> 3))
        || (first_bit_z == 1 && remaining_z_bits == 0);

    if touches_inner_x {
        f(BOUNDARY_INNER_X, x_projection);
    }
    if touches_inner_y {
        f(BOUNDARY_INNER_Y, y_projection);
    }
    if touches_inner_z {
        f(BOUNDARY_INNER_Z, z_projection);
    }
}
