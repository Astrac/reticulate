mod boundaries {
    use bitvec::prelude::*;

    use crate::{Morton, octree::faces::Face};

    pub const OUTER_NEGATIVE_X: u8 = 0b00001;
    pub const OUTER_NEGATIVE_Y: u8 = 0b00010;
    pub const OUTER_NEGATIVE_Z: u8 = 0b00100;
    pub const OUTER_POSITIVE_X: u8 = 0b01001;
    pub const OUTER_POSITIVE_Y: u8 = 0b01010;
    pub const OUTER_POSITIVE_Z: u8 = 0b01100;
    pub const INNER_X: u8 = 0b10001;
    pub const INNER_Y: u8 = 0b10010;
    pub const INNER_Z: u8 = 0b10100;

    // TODO: Move to graveyard
    #[allow(dead_code)]
    #[derive(Clone, Default, Debug)]
    pub struct Boundaries {
        masks: BitArr!(for 9 * (64 * 64), in u64, Lsb0),
        depth: u8,
    }

    const MASK_SIZE_BITS: usize = 64 * 64;
    const MASK_SIZE_BLOCKS: usize = 64;

    #[allow(dead_code, unused)]
    impl Boundaries {
        const INNER_BOUNDARIES: [u8; 3] = [INNER_X, INNER_Y, INNER_Z];

        // This is not needed: when handling on_leaf the boundaries are the context of the parent
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

        pub fn minimum_depth(&self, octant: u8) -> u8 {
            todo!()
        }

        // Octant 0 000 X=0001 Y=0010 Z=0100
        // Octant 1 001 X=1001 Y=0010 Z=0100
        // Octant 2 010 X=0001 Y=1010 Z=0100
        // Octant 3 011 X=1001 Y=1010 Z=0100
        // Octant 4 100 X=0001 Y=0010 Z=1100
        // Octant 5 101 X=1001 Y=0010 Z=1100
        // Octant 6 110 X=0001 Y=1010 Z=1100
        // Octant 7 111 X=1001 Y=1010 Z=1100
        // Each octant bit decides which face to flip
        // TODO: Optimize Vec, etc...
        pub fn inner_faces(&self, octant: u8) -> Vec<Face> {
            let mut result = Vec::new();
            for boundary in Self::INNER_BOUNDARIES {
                let x_idx = Self::boundary_block_index(boundary);
                let mask_size = (self.depth as usize).pow(2);

                // Can we track if children already had complex geometry instead of bitmasking?
                if (self.masks[x_idx..(x_idx + mask_size)].all()) {
                    let f = (boundary & 0b111) | ((octant & boundary).min(1) << 3);
                    result.push(Face::from(f))
                }
            }

            result
        }

        pub fn outer_faces(&self, octant: u8) -> Vec<Face> {
            todo!()
        }

        // Outer X -> Outer X
        // Inner X -> Outer X (P if X == 0, N if X == 1)
        // TODO: Do we even need to worry about slicking the bit arrays when forking/joining?
        pub fn fork_octant(&self, octant: u8) -> Boundaries {
            let mut forked = Boundaries::default();
            forked.depth = self.depth + 1;

            for boundary in Self::octant_outer_boundaries(octant) {
                let idx = Self::boundary_index(boundary);
                forked.masks[idx..(idx + MASK_SIZE_BITS)]
                    .copy_from_bitslice(&self.masks[idx..(idx + MASK_SIZE_BITS)]);
            }

            for boundary in Self::INNER_BOUNDARIES {
                let inner_idx = Self::boundary_index(boundary);
                let outer_idx =
                    Self::boundary_index(Self::outer_boundary_matching_inner(octant, boundary));
                forked.masks[outer_idx..(outer_idx + MASK_SIZE_BITS)]
                    .copy_from_bitslice(&self.masks[inner_idx..(inner_idx + MASK_SIZE_BITS)]);
            }

            forked
        }

        // TODO: Do we even need to worry about slicking the bit arrays when forking/joining?
        pub fn join_octant(&mut self, octant: u8, boundaries: &Boundaries) {
            for boundary in Self::octant_outer_boundaries(octant) {
                let idx = Self::boundary_index(boundary);
                self.masks[idx..(idx + MASK_SIZE_BITS)]
                    .copy_from_bitslice(&boundaries.masks[idx..(idx + MASK_SIZE_BITS)]);
            }

            for boundary in Self::INNER_BOUNDARIES {
                let inner_idx = Self::boundary_index(boundary);
                let outer_idx =
                    Self::boundary_index(Self::outer_boundary_matching_inner(octant, boundary));
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
        const fn boundary_index(boundary: u8) -> usize {
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

    #[inline(always)]
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
        OUTER_NEGATIVE_X,
        OUTER_NEGATIVE_Y,
        OUTER_NEGATIVE_Z,
        OUTER_POSITIVE_X,
        OUTER_POSITIVE_Y,
        OUTER_POSITIVE_Z,
    ];

    #[inline(always)]
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

        let [z_projection, y_projection, x_projection] =
            get_2d_outer_projections(leaf_morton, depth);

        if outer_boundary_mask != 0 {
            if (outer_boundary_mask & (1 << 0)) != 0 {
                f(OUTER_NEGATIVE_X, x_projection);
            }
            if (outer_boundary_mask & (1 << 1)) != 0 {
                f(OUTER_POSITIVE_X, x_projection);
            }
            if (outer_boundary_mask & (1 << 2)) != 0 {
                f(OUTER_NEGATIVE_Y, y_projection);
            }
            if (outer_boundary_mask & (1 << 3)) != 0 {
                f(OUTER_POSITIVE_Y, y_projection);
            }
            if (outer_boundary_mask & (1 << 4)) != 0 {
                f(OUTER_NEGATIVE_Z, z_projection);
            }
            if (outer_boundary_mask & (1 << 5)) != 0 {
                f(OUTER_POSITIVE_Z, z_projection);
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
            f(INNER_X, x_projection);
        }
        if touches_inner_y {
            f(INNER_Y, y_projection);
        }
        if touches_inner_z {
            f(INNER_Z, z_projection);
        }
    }
}

#[allow(dead_code)]
mod constants_and_luts {
    pub const LEFT_DOWN_BACK: u8 = 0; // 000
    pub const RIGHT_DOWN_BACK: u8 = 1; // 001
    pub const LEFT_UP_BACK: u8 = 2; // 010
    pub const RIGHT_UP_BACK: u8 = 3; // 011
    pub const LEFT_DOWN_FORWARD: u8 = 4; // 100
    pub const RIGHT_DOWN_FORWARD: u8 = 5; // 101
    pub const LEFT_UP_FORWARD: u8 = 6; // 110
    pub const RIGHT_UP_FORWARD: u8 = 7; // 111

    pub const VOXEL_BDL: u8 = 0;
    pub const VOXEL_BDR: u8 = 1;
    pub const VOXEL_BUL: u8 = 2;
    pub const VOXEL_BUR: u8 = 3;
    pub const VOXEL_FDL: u8 = 4;
    pub const VOXEL_FDR: u8 = 5;
    pub const VOXEL_FUL: u8 = 6;
    pub const VOXEL_FUR: u8 = 7;

    pub const FACE_LEFT: u8 = 0b000001; // -X
    pub const FACE_RIGHT: u8 = 0b000010; // +X
    pub const FACE_BOTTOM: u8 = 0b000100; // -Y
    pub const FACE_TOP: u8 = 0b001000; // +Y
    pub const FACE_BACK: u8 = 0b010000; // -Z
    pub const FACE_FRONT: u8 = 0b100000; // +Z

    const fn octant_face(voxel: u8, voxel_face: u8) -> u64 {
        (voxel_face as u64) << (6 * voxel)
    }

    pub const OCTANT_INTERNAL_FACES: u64 = octant_face(VOXEL_BDL, FACE_BACK);

    pub const OCTANT_EXTERNAL_FACES: u64 = 0;

    // Offsets of faces that facing outwards.
    pub const UP_OUT_IDX: u8 = 0;
    pub const DOWN_OUT_IDX: u8 = 4;
    pub const FORWARD_OUT_IDX: u8 = 8;
    pub const BACK_OUT_IDX: u8 = 12;
    pub const LEFT_OUT_IDX: u8 = 16;
    pub const RIGHT_OUT_IDX: u8 = 20;

    // Offsets of faces that facing inwards.
    pub const LEFT_RIGHT_IDX: u8 = 24;
    pub const RIGHT_LEFT_IDX: u8 = 28;
    pub const DOWN_UP_IDX: u8 = 32;
    pub const UP_DOWN_IDX: u8 = 36;
    pub const BACK_FOREWARD_IDX: u8 = 40;
    pub const FORWARD_BACK_INDEX: u8 = 44;

    pub const SOLIDITY_OCTANT_FACES_LUT: [u64; 256] = {
        let mut lut = [0; 256];
        let mut mask = 0;

        while mask < 256 {
            lut[mask] = compute_visible_faces(mask as u8);
            mask += 1;
        }

        lut
    };

    pub const SOLIDITY_VOXEL_FACES_LUT: [u64; 256] = {
        let mut lut = [0u64; 256];
        let mut solidity_mask = 0;

        while solidity_mask < 256 {
            let octant_mask = SOLIDITY_OCTANT_FACES_LUT[solidity_mask];

            let mut voxels = [0u8; 8];

            // --- 1. EXTRACT UP FACES ---
            // Outer Up (Bits 0..3) maps to Morton blocks [2, 3, 6, 7]
            // Internal Y-axis Down-to-Up (Bits 32..35) maps to Morton blocks [0, 1, 4, 5]
            let up_outer = (octant_mask & 0x0F) as u8;
            let up_inter = ((octant_mask >> 32) & 0x0F) as u8;
            voxels[0] |= ((up_inter >> 0) & 1) << 0; // Block 0 (LDB)
            voxels[1] |= ((up_inter >> 1) & 1) << 0; // Block 1 (RDB)
            voxels[2] |= ((up_outer >> 0) & 1) << 0; // Block 2 (LUB)
            voxels[3] |= ((up_outer >> 1) & 1) << 0; // Block 3 (RUB)
            voxels[4] |= ((up_inter >> 2) & 1) << 0; // Block 4 (LDF)
            voxels[5] |= ((up_inter >> 3) & 1) << 0; // Block 5 (RDF)
            voxels[6] |= ((up_outer >> 2) & 1) << 0; // Block 6 (LUF)
            voxels[7] |= ((up_outer >> 3) & 1) << 0; // Block 7 (RUF)

            // --- 2. EXTRACT DOWN FACES ---
            // Outer Down (Bits 4..7) maps to Morton blocks [4, 5, 0, 1]
            // Internal Y-axis Up-to-Down (Bits 36..39) maps to Morton blocks [6, 7, 2, 3]
            let down_outer = ((octant_mask >> 4) & 0x0F) as u8;
            let down_inter = ((octant_mask >> 36) & 0x0F) as u8;
            voxels[0] |= ((down_outer >> 2) & 1) << 1;
            voxels[1] |= ((down_outer >> 3) & 1) << 1;
            voxels[2] |= ((down_inter >> 2) & 1) << 1;
            voxels[3] |= ((down_inter >> 3) & 1) << 1;
            voxels[4] |= ((down_outer >> 0) & 1) << 1;
            voxels[5] |= ((down_outer >> 1) & 1) << 1;
            voxels[6] |= ((down_inter >> 0) & 1) << 1;
            voxels[7] |= ((down_inter >> 1) & 1) << 1;

            // --- 3. EXTRACT FRONT FACES ---
            // Outer Front (Bits 8..11) maps to Morton blocks [6, 7, 4, 5]
            // Internal Z-axis Front-to-Back (Bits 44..47) maps to Morton blocks [2, 3, 0, 1]
            let front_outer = ((octant_mask >> 8) & 0x0F) as u8;
            let front_inter = ((octant_mask >> 44) & 0x0F) as u8;
            voxels[0] |= ((front_inter >> 2) & 1) << 2;
            voxels[1] |= ((front_inter >> 3) & 1) << 2;
            voxels[2] |= ((front_inter >> 0) & 1) << 2;
            voxels[3] |= ((front_inter >> 1) & 1) << 2;
            voxels[4] |= ((front_outer >> 2) & 1) << 2;
            voxels[5] |= ((front_outer >> 3) & 1) << 2;
            voxels[6] |= ((front_outer >> 0) & 1) << 2;
            voxels[7] |= ((front_outer >> 1) & 1) << 2;

            // --- 4. EXTRACT BACK FACES ---
            // Outer Back (Bits 12..15) maps to Morton blocks [2, 3, 0, 1]
            // Internal Z-axis Back-to-Front (Bits 40..43) maps to Morton blocks [6, 7, 4, 5]
            let back_outer = ((octant_mask >> 12) & 0x0F) as u8;
            let back_inter = ((octant_mask >> 40) & 0x0F) as u8;
            voxels[0] |= ((back_outer >> 2) & 1) << 3;
            voxels[1] |= ((back_outer >> 3) & 1) << 3;
            voxels[2] |= ((back_outer >> 0) & 1) << 3;
            voxels[3] |= ((back_outer >> 1) & 1) << 3;
            voxels[4] |= ((back_inter >> 2) & 1) << 3;
            voxels[5] |= ((back_inter >> 3) & 1) << 3;
            voxels[6] |= ((back_inter >> 0) & 1) << 3;
            voxels[7] |= ((back_inter >> 1) & 1) << 3;

            // --- 5. EXTRACT LEFT FACES ---
            // Outer Left (Bits 16..19) maps to Morton blocks [2, 6, 0, 4]
            // Internal X-axis Right-to-Left (Bits 28..31) maps to Morton blocks [3, 7, 1, 5]
            let left_outer = ((octant_mask >> 16) & 0x0F) as u8;
            let left_inter = ((octant_mask >> 28) & 0x0F) as u8;
            voxels[0] |= ((left_outer >> 2) & 1) << 4;
            voxels[1] |= ((left_inter >> 2) & 1) << 4;
            voxels[2] |= ((left_outer >> 0) & 1) << 4;
            voxels[3] |= ((left_inter >> 0) & 1) << 4;
            voxels[4] |= ((left_outer >> 3) & 1) << 4;
            voxels[5] |= ((left_inter >> 3) & 1) << 4;
            voxels[6] |= ((left_outer >> 1) & 1) << 4;
            voxels[7] |= ((left_inter >> 1) & 1) << 4;

            // --- 6. EXTRACT RIGHT FACES ---
            // Outer Right (Bits 20..23) maps to Morton blocks [3, 7, 1, 5]
            // Internal X-axis Left-to-Right (Bits 24..27) maps to Morton blocks [2, 6, 0, 4]
            let right_outer = ((octant_mask >> 20) & 0x0F) as u8;
            let right_inter = ((octant_mask >> 24) & 0x0F) as u8;
            voxels[0] |= ((right_inter >> 2) & 1) << 5;
            voxels[1] |= ((right_outer >> 2) & 1) << 5;
            voxels[2] |= ((right_inter >> 0) & 1) << 5;
            voxels[3] |= ((right_outer >> 0) & 1) << 5;
            voxels[4] |= ((right_inter >> 3) & 1) << 5;
            voxels[5] |= ((right_outer >> 3) & 1) << 5;
            voxels[6] |= ((right_inter >> 1) & 1) << 5;
            voxels[7] |= (right_outer >> 1) & 5;

            lut[solidity_mask] = u64::from_le_bytes(voxels);
            solidity_mask += 1;
        }

        lut
    };

    const fn compute_visible_faces(solidity_mask: u8) -> u64 {
        let mut mask: u64 = 0;

        // 0: LDB (000)
        if is_solid(solidity_mask, LEFT_DOWN_BACK) {
            if !is_solid(solidity_mask, LEFT_UP_BACK) {
                set_bit(&mut mask, DOWN_UP_IDX, 0);
            } else {
                set_bit(&mut mask, DOWN_OUT_IDX, 2);
            }
            if !is_solid(solidity_mask, LEFT_DOWN_FORWARD) {
                set_bit(&mut mask, BACK_FOREWARD_IDX, 0);
            } else {
                set_bit(&mut mask, BACK_OUT_IDX, 2);
            }
            if !is_solid(solidity_mask, RIGHT_DOWN_BACK) {
                set_bit(&mut mask, LEFT_RIGHT_IDX, 0);
            } else {
                set_bit(&mut mask, LEFT_OUT_IDX, 2);
            }
        }
        // 1: RDB (001)
        if is_solid(solidity_mask, RIGHT_DOWN_BACK) {
            if !is_solid(solidity_mask, RIGHT_UP_BACK) {
                set_bit(&mut mask, DOWN_UP_IDX, 1);
            } else {
                set_bit(&mut mask, DOWN_OUT_IDX, 3);
            }
            if !is_solid(solidity_mask, RIGHT_DOWN_FORWARD) {
                set_bit(&mut mask, BACK_FOREWARD_IDX, 1);
            } else {
                set_bit(&mut mask, BACK_OUT_IDX, 3);
            }
            if !is_solid(solidity_mask, LEFT_DOWN_BACK) {
                set_bit(&mut mask, RIGHT_LEFT_IDX, 0);
            } else {
                set_bit(&mut mask, RIGHT_OUT_IDX, 2);
            }
        }
        // 2: LUB (010)
        if is_solid(solidity_mask, LEFT_UP_BACK) {
            if !is_solid(solidity_mask, LEFT_DOWN_BACK) {
                set_bit(&mut mask, UP_DOWN_IDX, 0);
            } else {
                set_bit(&mut mask, UP_OUT_IDX, 0);
            }
            if !is_solid(solidity_mask, LEFT_UP_FORWARD) {
                set_bit(&mut mask, BACK_FOREWARD_IDX, 2);
            } else {
                set_bit(&mut mask, BACK_OUT_IDX, 0);
            }
            if !is_solid(solidity_mask, RIGHT_UP_BACK) {
                set_bit(&mut mask, LEFT_RIGHT_IDX, 1);
            } else {
                set_bit(&mut mask, LEFT_OUT_IDX, 0);
            }
        }
        // 3: RUB (011)
        if is_solid(solidity_mask, RIGHT_UP_BACK) {
            if !is_solid(solidity_mask, RIGHT_DOWN_BACK) {
                set_bit(&mut mask, UP_DOWN_IDX, 1);
            } else {
                set_bit(&mut mask, UP_OUT_IDX, 1);
            }
            if !is_solid(solidity_mask, RIGHT_UP_FORWARD) {
                set_bit(&mut mask, BACK_FOREWARD_IDX, 3);
            } else {
                set_bit(&mut mask, BACK_OUT_IDX, 1);
            }
            if !is_solid(solidity_mask, LEFT_UP_BACK) {
                set_bit(&mut mask, RIGHT_LEFT_IDX, 1);
            } else {
                set_bit(&mut mask, RIGHT_OUT_IDX, 0);
            }
        }
        // 4: LDF (100)
        if is_solid(solidity_mask, LEFT_DOWN_FORWARD) {
            if !is_solid(solidity_mask, LEFT_UP_FORWARD) {
                set_bit(&mut mask, DOWN_UP_IDX, 2);
            } else {
                set_bit(&mut mask, DOWN_OUT_IDX, 0);
            }
            if !is_solid(solidity_mask, LEFT_DOWN_BACK) {
                set_bit(&mut mask, FORWARD_BACK_INDEX, 0);
            } else {
                set_bit(&mut mask, FORWARD_OUT_IDX, 2);
            }
            if !is_solid(solidity_mask, RIGHT_DOWN_FORWARD) {
                set_bit(&mut mask, LEFT_RIGHT_IDX, 2);
            } else {
                set_bit(&mut mask, LEFT_OUT_IDX, 3);
            }
        }
        // 5: RDF (101)
        if is_solid(solidity_mask, RIGHT_DOWN_FORWARD) {
            if !is_solid(solidity_mask, RIGHT_UP_FORWARD) {
                set_bit(&mut mask, DOWN_UP_IDX, 3);
            } else {
                set_bit(&mut mask, DOWN_OUT_IDX, 1);
            }
            if !is_solid(solidity_mask, RIGHT_DOWN_BACK) {
                set_bit(&mut mask, FORWARD_BACK_INDEX, 1);
            } else {
                set_bit(&mut mask, FORWARD_OUT_IDX, 3);
            }
            if !is_solid(solidity_mask, LEFT_DOWN_FORWARD) {
                set_bit(&mut mask, RIGHT_LEFT_IDX, 2);
            } else {
                set_bit(&mut mask, RIGHT_OUT_IDX, 3);
            }
        }
        // 6: LUF (110)
        if is_solid(solidity_mask, LEFT_UP_FORWARD) {
            if !is_solid(solidity_mask, LEFT_DOWN_FORWARD) {
                set_bit(&mut mask, UP_DOWN_IDX, 2);
            } else {
                set_bit(&mut mask, UP_OUT_IDX, 2);
            }
            if !is_solid(solidity_mask, LEFT_UP_BACK) {
                set_bit(&mut mask, FORWARD_BACK_INDEX, 2);
            } else {
                set_bit(&mut mask, FORWARD_OUT_IDX, 0);
            }
            if !is_solid(solidity_mask, RIGHT_UP_FORWARD) {
                set_bit(&mut mask, LEFT_RIGHT_IDX, 3);
            } else {
                set_bit(&mut mask, LEFT_OUT_IDX, 1);
            }
        }
        // 7: RUF (111)
        if is_solid(solidity_mask, RIGHT_UP_FORWARD) {
            if !is_solid(solidity_mask, RIGHT_DOWN_FORWARD) {
                set_bit(&mut mask, UP_DOWN_IDX, 3);
            } else {
                set_bit(&mut mask, UP_OUT_IDX, 3);
            }
            if !is_solid(solidity_mask, RIGHT_UP_BACK) {
                set_bit(&mut mask, FORWARD_BACK_INDEX, 3);
            } else {
                set_bit(&mut mask, FORWARD_OUT_IDX, 1);
            }
            if !is_solid(solidity_mask, LEFT_UP_FORWARD) {
                set_bit(&mut mask, RIGHT_LEFT_IDX, 3);
            } else {
                set_bit(&mut mask, RIGHT_OUT_IDX, 1);
            }
        }

        mask
    }

    const fn is_solid(mask: u8, bit: u8) -> bool {
        (mask & (1 << bit)) != 0
    }

    const fn set_bit(mask: &mut u64, offset: u8, quadrant: u8) {
        *mask |= 1 << (offset + quadrant);
    }
}

#[allow(dead_code)]
mod boundaries_try_1 {
    #[allow(dead_code)]
    #[derive(Clone, Copy, Debug, Default)]
    pub struct Boundaries {
        inner: [u64; 6],
        outer: [u64; 6],
    }

    #[allow(dead_code, unused)]
    impl Boundaries {
        fn create_child_boundaries(&mut self, octant: u8) -> Boundaries {
            let mut boundaries = Boundaries::default();
            let octant = octant as usize;
            let octant_neg = !octant;

            let (from_inner_x, from_inner_y, from_inner_z) =
                (0 ^ octant_neg, 2 ^ octant_neg, 4 ^ octant_neg);

            let (from_outer_x, from_outer_y, from_outer_z) = (0 ^ octant, 2 ^ octant, 4 ^ octant);

            boundaries.outer[from_inner_x ^ 1] = self.inner[from_inner_x];
            boundaries.outer[from_inner_y ^ 1] = self.inner[from_inner_y];
            boundaries.outer[from_inner_z ^ 1] = self.inner[from_inner_z];
            boundaries.outer[from_outer_x ^ 1] = self.outer[from_outer_x];
            boundaries.outer[from_outer_y ^ 1] = self.outer[from_outer_y];
            boundaries.outer[from_outer_z ^ 1] = self.outer[from_outer_z];

            boundaries
        }

        fn apply_child_boundaries(&mut self, octant: u8, boundaries: Boundaries) {
            let octant = octant as usize;
            let octant_neg = !octant;

            let (from_inner_x, from_inner_y, from_inner_z) =
                (1 ^ octant_neg, 3 ^ octant_neg, 5 ^ octant_neg);

            let (from_outer_x, from_outer_y, from_outer_z) = (1 ^ octant, 3 ^ octant, 5 ^ octant);

            self.inner[from_inner_x ^ 1] ^= boundaries.outer[from_inner_x];
            self.inner[from_inner_y ^ 1] ^= boundaries.outer[from_inner_y];
            self.inner[from_inner_z ^ 1] ^= boundaries.outer[from_inner_z];
            self.outer[from_outer_x ^ 1] ^= boundaries.outer[from_outer_x];
            self.outer[from_outer_y ^ 1] ^= boundaries.outer[from_outer_y];
            self.outer[from_outer_z ^ 1] ^= boundaries.outer[from_outer_z];
        }

        // Conceptually is the same as creating a child, settings
        // all its external masks to 1 and applying it to the parent
        //
        fn fill_octant(&mut self, octant: u8) {
            let mut boundaries = self.create_child_boundaries(octant);
            boundaries.inner.fill(u64::MAX);
            boundaries.outer.fill(u64::MAX);
            self.apply_child_boundaries(octant, boundaries);
        }

        #[inline(always)]
        fn iterate_faces(&self) -> impl Iterator<Item = &u64> {
            self.inner.iter().chain(self.outer.iter())
        }

        // Enumerates (index, (outer, inner))
        // Index: TODO explain relationship to morton
        // Outer: outer face mask
        // Inner: outer face mask
        #[inline(always)]
        fn enumerate_face_pairs(&self) -> impl Iterator<Item = (usize, (&u64, &u64))> {
            self.inner.iter().zip(self.outer.iter()).enumerate()
        }

        // Can I downsample once at the end of all the octree building??
        // If I only have 64bits facemasks (they cover each the full face
        // of the chunk, which is 64 small squares)
        fn maximum_downsampling(&self, limit: u8) -> (u8, Boundaries) {
            let limit = limit.min(6);

            let mut acc1 = 0u64;
            let mut acc2 = 0u64;
            let mut acc3 = 0u64;
            let mut acc4 = 0u64;
            let mut acc5 = 0u64;
            let mut acc6 = 0u64;

            // 1. Accumulate mismatch states across all 12 masks concurrently
            for &mask in self.iterate_faces() {
                acc1 |= (mask ^ (mask >> 1)) & 0x5555_5555_5555_5555_u64;
                acc2 |= (mask ^ (mask >> 2)) & 0x3333_3333_3333_3333_u64;
                acc3 |= (mask ^ (mask >> 4)) & 0x0F0F_0F0F_0F0F_0F0F_u64;
                acc4 |= (mask ^ (mask >> 8)) & 0x00FF_00FF_00FF_00FF_u64;
                acc5 |= (mask ^ (mask >> 16)) & 0x0000_FFFF_0000_FFFF_u64;
                acc6 |= (mask ^ (mask >> 32)) & 0x0000_0000_FFFF_FFFF_u64;
            }

            // 2. Reduce the 64-bit error masks down to single binary bit-flags (0 or 1)
            let e1 = (acc1 != 0) as u32;
            let e2 = (acc2 != 0) as u32;
            let e3 = (acc3 != 0) as u32;
            let e4 = (acc4 != 0) as u32;
            let e5 = (acc5 != 0) as u32;
            let e6 = (acc6 != 0) as u32;

            // 3. Pack the global failure bits into a single tracking byte
            let packed = e1 | (e2 << 1) | (e3 << 2) | (e4 << 3) | (e5 << 4) | (e6 << 5);

            // 5. Calculate final depth directly on your 0, 1, 2, 3, 4, 5, 6 scale
            let maximum_possible_downsampling = if packed == 0 {
                6
            } else {
                packed.trailing_zeros()
            };

            let downsampling = maximum_possible_downsampling.min(limit as u32);

            // 6. Perform clean logarithmic downsampling based on the solved final_depth
            let mut downsampled = Boundaries::default();
            match downsampling {
                0 => {
                    downsampled.inner = self.inner;
                    downsampled.outer = self.outer;
                }
                1 => {
                    for (index, (inner, outer)) in self.enumerate_face_pairs() {
                        downsampled.inner[index] = (inner & 0x5555_5555_5555_5555)
                            | ((inner & 0xAAAA_AAAA_AAAA_AAAA) >> 1);

                        downsampled.outer[index] = (outer & 0x5555_5555_5555_5555)
                            | ((outer & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                    }
                }
                2 => {
                    for (index, (inner, outer)) in self.enumerate_face_pairs() {
                        let inner2 = (inner & 0x5555_5555_5555_5555)
                            | ((inner & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        downsampled.inner[index] = (inner2 & 0x3333_3333_3333_3333)
                            | ((inner2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);

                        let outer2 = (outer & 0x5555_5555_5555_5555)
                            | ((outer & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        downsampled.outer[index] = (outer2 & 0x3333_3333_3333_3333)
                            | ((outer2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                    }
                }
                3 => {
                    for (index, (inner, outer)) in self.enumerate_face_pairs() {
                        let inner2 = (inner & 0x5555_5555_5555_5555)
                            | ((inner & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let inner4 = (inner2 & 0x3333_3333_3333_3333)
                            | ((inner2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        downsampled.inner[index] = (inner4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((inner4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);

                        let outer2 = (outer & 0x5555_5555_5555_5555)
                            | ((outer & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let outer4 = (outer2 & 0x3333_3333_3333_3333)
                            | ((outer2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        downsampled.outer[index] = (outer4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((outer4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                    }
                }
                4 => {
                    for (index, (inner, outer)) in self.enumerate_face_pairs() {
                        let inner2 = (inner & 0x5555_5555_5555_5555)
                            | ((inner & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let inner4 = (inner2 & 0x3333_3333_3333_3333)
                            | ((inner2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        let inner8 = (inner4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((inner4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                        downsampled.inner[index] = (inner8 & 0x00FF_00FF_00FF_00FF)
                            | ((inner8 & 0xFF00_FF00_FF00_FF00) >> 8);

                        let outer2 = (outer & 0x5555_5555_5555_5555)
                            | ((outer & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let outer4 = (outer2 & 0x3333_3333_3333_3333)
                            | ((outer2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        let outer8 = (outer4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((outer4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                        downsampled.outer[index] = (outer8 & 0x00FF_00FF_00FF_00FF)
                            | ((outer8 & 0xFF00_FF00_FF00_FF00) >> 8);
                    }
                }
                5 => {
                    for (index, (inner, outer)) in self.enumerate_face_pairs() {
                        let inner2 = (inner & 0x5555_5555_5555_5555)
                            | ((inner & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let inner4 = (inner2 & 0x3333_3333_3333_3333)
                            | ((inner2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        let inner8 = (inner4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((inner4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                        let inner16 = (inner8 & 0x00FF_00FF_00FF_00FF)
                            | ((inner8 & 0xFF00_FF00_FF00_FF00) >> 8);
                        downsampled.inner[index] = (inner16 & 0x0000_FFFF_0000_FFFF)
                            | ((inner16 & 0xFFFF_0000_FFFF_0000) >> 16);

                        let outer2 = (outer & 0x5555_5555_5555_5555)
                            | ((outer & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let outer4 = (outer2 & 0x3333_3333_3333_3333)
                            | ((outer2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        let outer8 = (outer4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((outer4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                        let outer16 = (outer8 & 0x00FF_00FF_00FF_00FF)
                            | ((outer8 & 0xFF00_FF00_FF00_FF00) >> 8);
                        downsampled.outer[index] = (outer16 & 0x0000_FFFF_0000_FFFF)
                            | ((outer16 & 0xFFFF_0000_FFFF_0000) >> 16);
                    }
                }
                6 => {
                    for (index, (inner, outer)) in self.enumerate_face_pairs() {
                        let inner2 = (inner & 0x5555_5555_5555_5555)
                            | ((inner & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let inner4 = (inner2 & 0x3333_3333_3333_3333)
                            | ((inner2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        let inner8 = (inner4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((inner4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                        let inner16 = (inner8 & 0x00FF_00FF_00FF_00FF)
                            | ((inner8 & 0xFF00_FF00_FF00_FF00) >> 8);
                        let inner32 = (inner16 & 0x0000_FFFF_0000_FFFF)
                            | ((inner16 & 0xFFFF_0000_FFFF_0000) >> 16);
                        downsampled.inner[index] = (inner32 & 0x0000_0000_FFFF_FFFF)
                            | ((inner32 & 0xFFFF_FFFF_0000_0000) >> 32);

                        let outer2 = (outer & 0x5555_5555_5555_5555)
                            | ((outer & 0xAAAA_AAAA_AAAA_AAAA) >> 1);
                        let outer4 = (outer2 & 0x3333_3333_3333_3333)
                            | ((outer2 & 0xCCCC_CCCC_CCCC_CCCC) >> 2);
                        let outer8 = (outer4 & 0x0F0F_0F0F_0F0F_0F0F)
                            | ((outer4 & 0xF0F0_F0F0_F0F0_F0F0) >> 4);
                        let outer16 = (outer8 & 0x00FF_00FF_00FF_00FF)
                            | ((outer8 & 0xFF00_FF00_FF00_FF00) >> 8);
                        let outer32 = (outer16 & 0x0000_FFFF_0000_FFFF)
                            | ((outer16 & 0xFFFF_0000_FFFF_0000) >> 16);
                        downsampled.outer[index] = (outer32 & 0x0000_0000_FFFF_FFFF)
                            | ((outer32 & 0xFFFF_FFFF_0000_0000) >> 32);
                    }
                }
                _ => unreachable!(),
            }

            (downsampling as u8, downsampled)
        }
    }

    #[allow(dead_code)]
    fn double_bits_u32(mask: u32) -> u64 {
        let mut m = mask as u64;

        m = (m | (m << 8)) & 0x00FF00FF00FF00FF_u64;
        m = (m | (m << 4)) & 0x0F0F0F0F0F0F0F0F_u64;
        m = (m | (m << 2)) & 0x3333333333333333_u64;
        m = (m | (m << 1)) & 0x5555555555555555_u64;

        m | (m << 1)
    }
}
