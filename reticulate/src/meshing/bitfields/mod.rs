mod material;
mod occlusion;
mod parity;

pub use material::Material;
pub use occlusion::Occlusion;
pub use parity::{Parity, ParityMask};

use bitvec::prelude::*;

use crate::octree::TREE_DEPTH;

type Word = u64;
type Slice = BitSlice<u64, Lsb0>;
// TODO Move this type, here is wrong
pub type MaterialId = u16;

pub const STORAGE_WORD: usize = size_of::<Word>() * 8;
pub const MATERIAL_BITS: usize = size_of::<MaterialId>() * 8;
pub const PLANE_SIDE: usize = 1 << TREE_DEPTH;
pub const PLANE_AREA: usize = PLANE_SIDE * PLANE_SIDE;
pub const CHUNK_VOLUME: usize = PLANE_SIDE * PLANE_SIDE * PLANE_SIDE;

const OCCLUSION_BITS: usize = CHUNK_VOLUME;
const OCCLUSION_WORDS: usize = OCCLUSION_BITS / STORAGE_WORD;
const MATERIAL_PLANE_BITS: usize = PLANE_AREA * MATERIAL_BITS;
const MATERIAL_PLANE_WORDS: usize = PLANE_AREA * MATERIAL_BITS / STORAGE_WORD;
const PARITY_BITS: usize = CHUNK_VOLUME;
const PARITY_WORDS: usize = PARITY_BITS / STORAGE_WORD;

const BITFIELD_SIZE: usize = (OCCLUSION_BITS * 3) + (MATERIAL_PLANE_BITS * 3) + (PARITY_BITS * 3);

#[derive(Clone, Default)]
pub struct BitfieldsStorage {
    bits: BitArr!(for BITFIELD_SIZE, in Word, Lsb0),
}

pub struct Bitfields<'a> {
    pub occlusion: Occlusion<'a>,
    pub parity: Parity<'a>,
    pub material: Material<'a>,
}

impl<'a> Bitfields<'a> {
    pub fn from_storage(storage: &'a mut BitfieldsStorage) -> Bitfields<'a> {
        let storage_words = storage.bits.as_raw_mut_slice();
        let (x_material_plane, storage_words) = storage_words.split_at_mut(MATERIAL_PLANE_WORDS);
        let (y_material_plane, storage_words) = storage_words.split_at_mut(MATERIAL_PLANE_WORDS);
        let (z_material_plane, storage_words) = storage_words.split_at_mut(MATERIAL_PLANE_WORDS);
        let (x_occlusion, storage_words) = storage_words.split_at_mut(OCCLUSION_WORDS);
        let (y_occlusion, storage_words) = storage_words.split_at_mut(OCCLUSION_WORDS);
        let (z_occlusion, storage_words) = storage_words.split_at_mut(OCCLUSION_WORDS);
        let (x_parity, storage_words) = storage_words.split_at_mut(PARITY_WORDS);
        let (y_parity, z_parity) = storage_words.split_at_mut(PARITY_WORDS);

        Bitfields {
            occlusion: Occlusion {
                masks: [
                    BitSlice::from_slice_mut(x_occlusion),
                    BitSlice::from_slice_mut(y_occlusion),
                    BitSlice::from_slice_mut(z_occlusion),
                ],
            },
            parity: Parity {
                masks: [
                    ParityMask::from_u64_slice(x_parity),
                    ParityMask::from_u64_slice(y_parity),
                    ParityMask::from_u64_slice(z_parity),
                ],
            },
            material: Material {
                cells: [
                    BitSlice::from_slice_mut(x_material_plane),
                    BitSlice::from_slice_mut(y_material_plane),
                    BitSlice::from_slice_mut(z_material_plane),
                ],
            },
        }
    }
}
