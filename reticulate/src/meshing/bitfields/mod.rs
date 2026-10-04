mod claims;
mod material;
mod parity;

pub use claims::{Claims, ClaimsMask};
pub use material::Material;
pub use parity::{Parity, ParityMask};

use bitvec::prelude::*;

use crate::{CHUNK_VOLUME, MaterialId, PLANE_AREA};

type Word = u64;
type Slice = BitSlice<u64, Lsb0>;

pub const CLAIMS_BITS: usize = CHUNK_VOLUME;
pub const CLAIMS_WORDS: usize = CLAIMS_BITS / STORAGE_WORD;
pub const MATERIAL_BITS: usize = MaterialId::BITS as usize;
pub const MATERIAL_PLANE_BITS: usize = PLANE_AREA * MATERIAL_BITS;
pub const MATERIAL_WORDS: usize = PLANE_AREA * MATERIAL_BITS / STORAGE_WORD;
pub const PARITY_BITS: usize = CHUNK_VOLUME;
pub const PARITY_WORDS: usize = PARITY_BITS / STORAGE_WORD;
pub const STORAGE_WORD: usize = Word::BITS as usize;

const BITFIELD_SIZE: usize = (CLAIMS_BITS * 3) + (MATERIAL_PLANE_BITS * 3) + (PARITY_BITS * 3);

#[derive(Clone, Debug, Default)]
pub struct BitfieldsStorage {
    bits: BitArr!(for BITFIELD_SIZE, in Word, Lsb0),
}

pub struct Bitfields<'a> {
    pub claims: Claims<'a>,
    pub parity: Parity<'a>,
    pub material: Material<'a>,
}

impl<'a> Bitfields<'a> {
    pub fn from_storage(storage: &'a mut BitfieldsStorage) -> Bitfields<'a> {
        let storage_words = storage.bits.as_raw_mut_slice();
        let (x_parity, storage_words) = storage_words.split_at_mut(PARITY_WORDS);
        let (y_parity, storage_words) = storage_words.split_at_mut(PARITY_WORDS);
        let (z_parity, storage_words) = storage_words.split_at_mut(PARITY_WORDS);
        let (x_material_plane, storage_words) = storage_words.split_at_mut(MATERIAL_WORDS);
        let (y_material_plane, storage_words) = storage_words.split_at_mut(MATERIAL_WORDS);
        let (z_material_plane, storage_words) = storage_words.split_at_mut(MATERIAL_WORDS);
        let (x_claims, storage_words) = storage_words.split_at_mut(CLAIMS_WORDS);
        let (y_claims, z_claims) = storage_words.split_at_mut(CLAIMS_WORDS);

        Bitfields {
            claims: Claims {
                masks: [
                    ClaimsMask::from_u64_slice(x_claims),
                    ClaimsMask::from_u64_slice(y_claims),
                    ClaimsMask::from_u64_slice(z_claims),
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
