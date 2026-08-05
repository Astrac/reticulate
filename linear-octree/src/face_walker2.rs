use crate::{BlockFaces, Morton, Walker};

#[allow(dead_code)]
pub struct FaceWalker2 {
    faces: Vec<BlockFaces>,
    pending: Vec<u64>,
    face_mask: u64,
}

#[allow(dead_code)]
impl FaceWalker2 {
    pub fn new() -> FaceWalker2 {
        FaceWalker2 {
            faces: Vec::new(),
            pending: Vec::new(),
            face_mask: 0,
        }
    }

    pub fn faces(&self) -> &[BlockFaces] {
        &self.faces
    }
}

#[allow(dead_code, unused)]
impl<_Data> Walker<_Data> for FaceWalker2 {
    fn on_branch_start(&mut self, depth: u8, octant: u8, morton: Morton, children_mask: u8) {
        // self.face_mask = self.face_mask ^ BLOCK_ALL_FACES_LUT[octant as usize];
        self.pending.push(self.face_mask);
        self.face_mask = 0x0;
    }

    fn on_branch_end(&mut self, depth: u8, octant: u8, morton: Morton) {
        // TODO improve!
        for i in 0..8 {
            let result = compress_mask(self.face_mask, BLOCK_ALL_FACES_LUT[i]);

            if result != 0 {
                self.faces.push(BlockFaces {
                    depth,
                    coords: morton.into(),
                    face_mask: result as u8,
                });
            }
        }

        if let Some(pending_mask) = self.pending.pop() {
            self.face_mask = pending_mask;
        }
    }

    fn on_leaf(&mut self, depth: u8, octant: u8, morton: Morton, data: &_Data) {
        self.face_mask = self.face_mask ^ BLOCK_ALL_FACES_LUT[octant as usize];
    }
}


/// Compile-time Look-Up Table (LUT) for all 8 voxels in the octant.
/// Indexed directly using the voxel's 3-bit Morton index (0..8).
pub const BLOCK_ALL_FACES_LUT: [u64; 8] = {
    let mut lut = [0u64; 8];
    let mut i = 0;
    while i < 8 {
        lut[i] = calculate_block_faces_mask_packed(i);
        i += 1;
    }
    lut
};

const fn calculate_block_faces_mask_packed(morton_index: usize) -> u64 {
    let x = (morton_index >> 0) & 1;
    let y = (morton_index >> 1) & 1;
    let z = (morton_index >> 2) & 1;

    let idx_yz = (z << 1) | y;
    let idx_xz = (z << 1) | x;
    let idx_xy = (y << 1) | x;

    let mut mask = 0u64;

    // X-Aligned Faces: Bits 0 to 11 (Offset 0)
    mask |= 1 << (0 + (x * 4) + idx_yz);
    mask |= 1 << (0 + ((x + 1) * 4) + idx_yz);

    // Y-Aligned Faces: Bits 12 to 23 (Offset 12)
    mask |= 1 << (12 + (y * 4) + idx_xz);
    mask |= 1 << (12 + ((y + 1) * 4) + idx_xz);

    // Z-Aligned Faces: Bits 24 to 35 (Offset 24)
    mask |= 1 << (24 + (z * 4) + idx_xy);
    mask |= 1 << (24 + ((z + 1) * 4) + idx_xy);

    mask
}

fn compress_mask(value: u64, mut block_mask: u64) -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("bmi2") {
            unsafe { return std::arch::x86_64::_pext_u64(value, block_mask) as u8 };
        }
    }

    let mut result = 0;
    let mut shift = 0;
    while block_mask != 0 {
        let lowest_bit = block_mask & block_mask.wrapping_neg();
        if (value & lowest_bit) != 0 {
            result |= 1 << shift;
        }
        shift += 1;
        block_mask &= block_mask - 1;
    }
    result
}
