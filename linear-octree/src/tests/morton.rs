use crate::{Coords, Morton};

#[test]
fn test_roundtrip_zero() {
    let c = Coords { x: 0, y: 0, z: 0 };
    let m: Morton = c.into();
    let decoded: Coords = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for zero");
}

#[test]
fn test_roundtrip_one() {
    let c = Coords { x: 1, y: 1, z: 1 };
    let m: Morton = c.into();
    let decoded: Coords = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for one");
}

#[test]
fn test_roundtrip_max() {
    let c = Coords {
        x: 255,
        y: 255,
        z: 255,
    };
    let m: Morton = c.into();
    let decoded: Coords = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for max");
}

#[test]
fn test_roundtrip_mixed() {
    let c = Coords {
        x: 10,
        y: 20,
        z: 30,
    };
    let m: Morton = c.into();
    let decoded: Coords = m.into();
    assert_eq!(c, decoded, "Roundtrip failed for mixed values");
}

#[test]
fn test_uniqueness() {
    // Different coordinates should produce different Morton codes
    let c1 = Coords { x: 1, y: 2, z: 3 };
    let c2 = Coords { x: 2, y: 1, z: 3 };
    let c3 = Coords { x: 3, y: 2, z: 1 };

    let m1: Morton = c1.into();
    let m2: Morton = c2.into();
    let m3: Morton = c3.into();

    assert_ne!(m1, m2, "Distinct coords c1 and c2 produced same code");
    assert_ne!(m2, m3, "Distinct coords c2 and c3 produced same code");
    assert_ne!(m1, m3, "Distinct coords c1 and c3 produced same code");
}

#[test]
fn test_boundary_values() {
    // Test specific boundary cases
    let cases = vec![
        Coords { x: 0, y: 0, z: 1 },
        Coords { x: 0, y: 1, z: 0 },
        Coords { x: 1, y: 0, z: 0 },
        Coords {
            x: 127,
            y: 127,
            z: 127,
        },
        Coords { x: 255, y: 0, z: 0 },
    ];

    for c in cases {
        let m: Morton = c.into();
        let decoded: Coords = m.into();
        assert_eq!(c, decoded, "Roundtrip failed for {:?}", c);
    }
}
