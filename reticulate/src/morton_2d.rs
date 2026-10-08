use crate::{ChunkCoord, Coords2D, Depth, PLANE_AREA, coords_lut::COORDS_LUT, utils::bitmasks};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Morton2D {
    code: u32,
    depth: Depth,
}

impl Morton2D {
    // TODO - Rename this to make it clear that depth is ignored
    #[inline(always)]
    pub const fn as_u32(&self) -> u32 {
        self.code
    }

    #[inline(always)]
    pub const fn depth(&self) -> Depth {
        self.depth
    }

    #[inline(always)]
    pub const fn as_usize(&self) -> usize {
        self.code as usize
    }

    #[inline(always)]
    pub const fn as_coords(&self) -> Coords2D {
        debug_assert!((self.code as usize) < PLANE_AREA);
        COORDS_LUT[self.code as usize]
    }

    #[inline(always)]
    pub fn from_uv_max_depth(u: ChunkCoord, v: ChunkCoord) -> Morton2D {
        Morton2D {
            code: encode_morton(u as u32, v as u32),
            depth: Depth::MAX,
        }
    }

    #[inline(always)]
    pub fn from_uv_and_depth(u: ChunkCoord, v: ChunkCoord, depth: Depth) -> Morton2D {
        Morton2D {
            code: encode_morton(u as u32, v as u32),
            depth,
        }
    }

    #[inline(always)]
    pub const fn from_code_max_depth(code: u32) -> Morton2D {
        Morton2D::from_code_and_depth(code, Depth::MAX)
    }

    #[inline(always)]
    pub const fn from_code_and_depth(code: u32, depth: Depth) -> Morton2D {
        Morton2D { code, depth }
    }
}

/// Interleaves the bits of `x` (even bits) and `y` (odd bits).
///
/// On x86_64 this uses BMI2 `pdep` when available (statically when compiled
/// with `+bmi2`, otherwise via a cached runtime check). Every other target,
/// including aarch64, uses the portable bit-twiddling implementation.
#[inline(always)]
fn encode_morton(x: u32, y: u32) -> u32 {
    #[cfg(target_arch = "x86_64")]
    if std::arch::is_x86_feature_detected!("bmi2") {
        // SAFETY: BMI2 support has just been verified.
        return unsafe { encode_morton_bmi2(x, y) };
    }

    encode_morton_portable(x, y)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline]
pub(crate) unsafe fn encode_morton_bmi2(x: u32, y: u32) -> u32 {
    use std::arch::x86_64::_pdep_u32;
    _pdep_u32(x, 0x5555_5555) | _pdep_u32(y, 0xAAAA_AAAA)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline]
pub(crate) unsafe fn decode_morton_bmi2(code: u32) -> (u32, u32) {
    use std::arch::x86_64::_pext_u32;
    (_pext_u32(code, 0x5555_5555), _pext_u32(code, 0xAAAA_AAAA))
}

#[inline(always)]
pub(crate) const fn encode_morton_portable(x: u32, y: u32) -> u32 {
    bitmasks::interleave_zeroes(x) | (bitmasks::interleave_zeroes(y) << 1)
}

#[inline(always)]
pub(crate) const fn decode_morton_portable(code: u32) -> (u32, u32) {
    (
        bitmasks::take_one_every_2_bits(code),
        bitmasks::take_one_every_2_bits(code >> 1),
    )
}
