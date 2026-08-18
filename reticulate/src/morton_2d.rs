use crate::{utils::bitmasks::*, coords::Coords2D};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Morton2D(u32);

impl Morton2D {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub(crate) fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl From<Morton2D> for Coords2D {
    fn from(value: Morton2D) -> Self {
        let (u, v) = decode_morton(value.0);
        Coords2D { u, v }
    }
}

impl From<Coords2D> for Morton2D {
    fn from(value: Coords2D) -> Self {
        Morton2D(encode_morton(value.u, value.v))
    }
}

impl From<u32> for Morton2D {
    fn from(value: u32) -> Self {
        Morton2D(value)
    }
}

#[inline(always)]
fn encode_morton(x: u32, y: u32) -> u32 {
    #[cfg(target_feature = "bmi2")]
    {
        return unsafe { encode_morton_32_bmi2(x, y) };
    }

    #[cfg(all(target_arch = "x86_64", not(target_feature = "bmi2")))]
    {
        if is_x86_feature_detected!("bmi2") {
            return unsafe { encode_morton_32_bmi2(x, y) };
        }
    }

    encode_morton_portable(x, y)
}

#[inline(always)]
fn decode_morton(code: u32) -> (u32, u32) {
    #[cfg(target_feature = "bmi2")]
    {
        return unsafe { decode_morton_bmi2(code) };
    }

    #[cfg(all(target_arch = "x86_64", not(target_feature = "bmi2")))]
    {
        if is_x86_feature_detected!("bmi2") {
            return unsafe { decode_morton_bmi2(code) };
        }
    }

    decode_morton_portable(code)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline(always)]
unsafe fn encode_morton_bmi2(x: u32, y: u32) -> u32 {
    use std::arch::x86_64::_pdep_u32;
    _pdep_u32(x, 0x5555_5555) | _pdep_u32(y, 0xAAAA_AAAA)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline(always)]
unsafe fn decode_morton_bmi2(code: u32) -> (u32, u32) {
    use std::arch::x86_64::_pext_u32;
    (_pext_u32(code, 0x5555_5555), _pext_u32(code, 0xAAAA_AAAA))
}

#[inline(always)]
const fn encode_morton_portable(x: u32, y: u32) -> u32 {
    interleave_zeroes(x) | (interleave_zeroes(y) << 1)
}

#[inline(always)]
const fn decode_morton_portable(code: u32) -> (u32, u32) {
    (
        take_one_every_2_bits(code),
        take_one_every_2_bits(code >> 1),
    )
}
