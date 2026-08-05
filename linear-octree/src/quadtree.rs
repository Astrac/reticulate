// use std::ops::{BitXor, BitXorAssign};

// // This tracks a face at a maximum resolution of 4096
// #[derive(Clone, Debug)]
// pub enum FaceMask {
//     // When we need the full for part of the face 4096
//     Branched(u64, SmallVec<[u64; 3]>),
//     // When we are OK with 64
//     Simple(u64),
// }
//
// impl Default for FaceMask {
//     fn default() -> Self {
//         FaceMask::Simple(u64)
//     }
// }
//
// impl BitXor for FaceMask {
//     type Output = FaceMask;
//
//     fn bitxor(self, rhs: Self) -> Self::Output {
//         todo!()
//     }
// }
//
// impl BitXorAssign for FaceMask {
//     fn bitxor_assign(&mut self, rhs: Self) {
//         todo!()
//     }
// }
