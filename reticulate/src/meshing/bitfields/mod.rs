mod claims;
mod material;
mod parity;

pub use claims::{Claims, ClaimsMask};
pub use material::Material;
pub use parity::{Parity, ParityMask};

use crate::{CHUNK_VOLUME, MaterialId, PLANE_AREA};

type Word = u64;

pub const CLAIMS_BITS: usize = CHUNK_VOLUME;
pub const CLAIMS_WORDS: usize = CLAIMS_BITS / STORAGE_WORD;
pub const PARITY_BITS: usize = CHUNK_VOLUME;
pub const PARITY_WORDS: usize = PARITY_BITS / STORAGE_WORD;
pub const STORAGE_WORD: usize = Word::BITS as usize;

#[derive(Clone, Debug)]
#[repr(C, align(64))]
pub struct BitfieldsStorage {
    material: [[MaterialId; PLANE_AREA]; 3],
    parity: [[Word; PARITY_WORDS]; 3],
    claims: [[Word; CLAIMS_WORDS]; 3],
}

impl Default for BitfieldsStorage {
    fn default() -> Self {
        Self {
            parity: [[0; _]; 3],
            material: [[0; _]; 3],
            claims: [[0; _]; 3],
        }
    }
}

pub struct Bitfields<'a> {
    pub claims: Claims<'a>,
    pub parity: Parity<'a>,
    pub material: Material<'a>,
}

impl<'a> Bitfields<'a> {
    pub fn from_storage(storage: &'a mut BitfieldsStorage) -> Bitfields<'a> {
        Bitfields {
            claims: Claims {
                masks: storage
                    .claims
                    .each_mut()
                    .map(|d| ClaimsMask::from_u64_slice(d)),
            },
            parity: Parity {
                masks: storage
                    .parity
                    .each_mut()
                    .map(|d| ParityMask::from_u64_slice(d)),
            },
            material: Material {
                planes: storage.material.each_mut(),
            },
        }
    }
}
