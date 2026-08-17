mod bitfield;
mod face_walker;
mod routing;

#[allow(dead_code)]
pub mod face_walker_rethink;
#[allow(dead_code)]
mod decision_boundaries;
#[allow(dead_code)]
mod working_boundary;

pub use face_walker::FaceWalker;

const BITFIELD_FACE_FULL_SIZE: usize = 64 * 64;

const BITFIELD_BOUNDARY_SIZE: usize = BITFIELD_FACE_FULL_SIZE;

const BITFIELD_SIZE: usize = BITFIELD_BOUNDARY_SIZE * 12;

const OUT_POS_X_IDX: usize = 0;
const OUT_POS_Y_IDX: usize = BITFIELD_BOUNDARY_SIZE + 1;
const OUT_POS_Z_IDX: usize = OUT_POS_Y_IDX + BITFIELD_BOUNDARY_SIZE;
const OUT_NEG_X_IDX: usize = OUT_POS_Z_IDX + BITFIELD_BOUNDARY_SIZE;
const OUT_NEG_Y_IDX: usize = OUT_NEG_X_IDX + BITFIELD_BOUNDARY_SIZE;
const OUT_NEG_Z_IDX: usize = OUT_NEG_Y_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_X_IDX: usize = OUT_NEG_Z_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_Y_IDX: usize = INN_POS_X_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_Z_IDX: usize = INN_POS_Y_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_X_IDX: usize = INN_POS_Z_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_Y_IDX: usize = INN_NEG_X_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_Z_IDX: usize = INN_NEG_Y_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_X_1_IDX: usize = OUT_NEG_Z_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_Y_1_IDX: usize = INN_POS_X_1_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_Z_1_IDX: usize = INN_POS_Y_1_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_X_1_IDX: usize = INN_POS_Z_1_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_Y_1_IDX: usize = INN_NEG_X_1_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_Z_1_IDX: usize = INN_NEG_Y_1_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_X_2_IDX: usize = INN_NEG_Z_1_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_Y_2_IDX: usize = INN_POS_X_2_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_POS_Z_2_IDX: usize = INN_POS_Y_2_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_X_2_IDX: usize = INN_POS_Z_2_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_Y_2_IDX: usize = INN_NEG_X_2_IDX + BITFIELD_BOUNDARY_SIZE;
const INN_NEG_Z_2_IDX: usize = INN_NEG_Y_2_IDX + BITFIELD_BOUNDARY_SIZE;

const BITFIELD_FACE_SIZE_BY_DEPTH: [usize; 7] = {
    let mut res = [0; 7];
    let mut i = 0;

    while i < 7 {
        res[i] = BITFIELD_FACE_FULL_SIZE >> (i * 2);
        i += 1;
    }

    res
};

// // [  4096 |  5120 |  5376 | 5440 | 5456 | 5460 | 5461 ]
// // [ 64x64 | 32x32 | 16x16 |  8x8 |  4x4 | 2x2  |  1x1 ]
// // [     0 |     1 |     2 |    3 |    4 |   5  |    6 ]
//
// const BITFIELD_DEPTH_OFFSETS: [usize; 5] = {
//     let mut res = [0usize; 5];
//     let mut offset = 0;
//     let mut i = 0;
//
//     while i < 5 {
//         offset = offset + BITFIELD_FACE_SIZE_BY_DEPTH[i];
//         res[i as usize] = offset;
//         i += 1;
//     }
//
//     res
// };
