use crate::{
    Coords2D, Depth, Morton2D,
    morton_2d::{decode_morton_portable, encode_morton_portable},
};

#[test]
fn u_takes_the_even_bits_and_v_the_odd_bits() {
    assert_eq!(Morton2D::from_uv_max_depth(1, 0).as_u32(), 0b01);
    assert_eq!(Morton2D::from_uv_max_depth(0, 1).as_u32(), 0b10);
    assert_eq!(Morton2D::from_uv_max_depth(0b11, 0).as_u32(), 0b0101);
    assert_eq!(Morton2D::from_uv_max_depth(0, 0b11).as_u32(), 0b1010);
    assert_eq!(Morton2D::from_uv_max_depth(0b101, 0b011).as_u32(), 0b011011);
}

#[test]
fn roundtrip_through_coords_for_every_chunk_coordinate() {
    for u in 0..=u8::MAX {
        for v in 0..=u8::MAX {
            let coords = Morton2D::from_uv_max_depth(u, v).as_coords();
            assert_eq!(coords, Coords2D { u, v });
        }
    }
}

#[test]
fn codes_of_distinct_coordinates_are_distinct() {
    let mut seen = vec![false; 1 << 16];
    for u in 0..=u8::MAX {
        for v in 0..=u8::MAX {
            let code = Morton2D::from_uv_max_depth(u, v).as_usize();
            assert!(!seen[code], "({u}, {v}) collides");
            seen[code] = true;
        }
    }
}

#[test]
fn constructors_record_the_depth() {
    assert_eq!(Morton2D::from_uv_max_depth(1, 2).depth(), Depth::MAX);
    assert_eq!(
        Morton2D::from_uv_and_depth(1, 2, Depth(3)).depth(),
        Depth(3)
    );
    assert_eq!(Morton2D::from_code_max_depth(7).depth(), Depth::MAX);
    assert_eq!(Morton2D::from_code_and_depth(7, Depth(2)).depth(), Depth(2));
}

#[test]
fn depth_does_not_change_the_code() {
    assert_eq!(
        Morton2D::from_uv_and_depth(5, 9, Depth(2)).as_u32(),
        Morton2D::from_uv_max_depth(5, 9).as_u32()
    );
}

#[test]
fn from_code_keeps_the_code_verbatim() {
    // Even bits (u): 0, 1, 1, 1 from the bottom -> 0b1110. Odd bits (v):
    // 1, 0, 0, 1 from the bottom -> 0b1001.
    let m = Morton2D::from_code_max_depth(0b1101_0110);
    assert_eq!(m.as_u32(), 0b1101_0110);
    assert_eq!(m.as_usize(), 0b1101_0110);
    assert_eq!(
        m.as_coords(),
        Coords2D {
            u: 0b1110,
            v: 0b1001
        }
    );
}

#[test]
fn portable_implementation_roundtrips() {
    for x in 0..=u8::MAX as u32 {
        for y in 0..=u8::MAX as u32 {
            assert_eq!(decode_morton_portable(encode_morton_portable(x, y)), (x, y));
        }
    }
}

#[test]
fn portable_implementation_handles_the_full_16_bits() {
    assert_eq!(encode_morton_portable(0xFFFF, 0), 0x5555_5555);
    assert_eq!(encode_morton_portable(0, 0xFFFF), 0xAAAA_AAAA);
    assert_eq!(decode_morton_portable(0xFFFF_FFFF), (0xFFFF, 0xFFFF));
}

/// The dispatching `encode`/`decode` pick the BMI2 path on x86_64 CPUs that
/// have it and the portable one everywhere else (aarch64 included), so the
/// public results must match the portable reference either way.
#[test]
fn dispatch_matches_the_portable_reference() {
    for u in 0..=u8::MAX {
        for v in 0..=u8::MAX {
            let m = Morton2D::from_uv_max_depth(u, v);
            assert_eq!(m.as_u32(), encode_morton_portable(u as u32, v as u32));
        }
    }
    for code in (0..=u32::MAX).step_by(0x0001_0001 + 2) {
        let (u, v) = decode_morton_portable(code);
        let coords = Morton2D::from_code_max_depth(code).as_coords();
        assert_eq!((coords.u as u32, coords.v as u32), (u & 0xFF, v & 0xFF));
    }
}

#[cfg(target_arch = "x86_64")]
#[test]
fn bmi2_and_portable_implementations_agree() {
    use crate::morton_2d::{decode_morton_bmi2, encode_morton_bmi2};

    if !std::arch::is_x86_feature_detected!("bmi2") {
        eprintln!("skipped: this CPU has no BMI2");
        return;
    }

    for x in (0..=0xFFFF).step_by(257) {
        for y in (0..=0xFFFF).step_by(251) {
            // SAFETY: BMI2 support was checked above.
            let (hw, sw) = unsafe { (encode_morton_bmi2(x, y), encode_morton_portable(x, y)) };
            assert_eq!(hw, sw, "encode({x}, {y})");
        }
    }
    for code in (0..=u32::MAX).step_by(0x0001_0001 + 2) {
        // SAFETY: BMI2 support was checked above.
        let hw = unsafe { decode_morton_bmi2(code) };
        assert_eq!(hw, decode_morton_portable(code), "decode({code:#x})");
    }
}
