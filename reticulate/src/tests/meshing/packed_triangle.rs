use crate::{
    Axis, Coords3D, Face, MaterialId,
    meshing::{packed_triangle::PackedTriangle, surface_layer::SurfaceLayer},
};

type Uv = [(u8, u8); 3];

const MAX_PLANE: u8 = PackedTriangle::PLANE_MASK as u8; // 31: plane 32 belongs to the neighbouring chunk
const MAX_UV: u8 = PackedTriangle::UV_MASK as u8 - 31; // 32: in-plane vertices sit on cell corners

const STORED_NORMAL_SIGN: i32 = 1;

const CANONICAL_UV: Uv = [(0, 0), (0, 1), (1, 0)];

fn faces() -> [Face; 6] {
    [
        Face::Positive(Axis::X),
        Face::Negative(Axis::X),
        Face::Positive(Axis::Y),
        Face::Negative(Axis::Y),
        Face::Positive(Axis::Z),
        Face::Negative(Axis::Z),
    ]
}

fn xyz(c: Coords3D) -> (u8, u8, u8) {
    (c.x, c.y, c.z)
}

fn along(c: Coords3D, axis: Axis) -> u8 {
    match axis.as_usize() {
        0 => c.x,
        1 => c.y,
        _ => c.z,
    }
}

/// Everything the packing has to preserve.
#[derive(Clone, Copy, Debug)]
struct Fields {
    face: Face,
    plane: u8,
    uv: Uv,
    material: MaterialId,
}

impl Fields {
    fn zero(face: Face) -> Self {
        Fields {
            face,
            plane: 0,
            uv: [(0, 0); 3],
            material: 0,
        }
    }
    fn full(face: Face) -> Self {
        Fields {
            face,
            plane: MAX_PLANE,
            uv: [(MAX_UV, MAX_UV); 3],
            material: MaterialId::MAX,
        }
    }
    fn pack(&self) -> PackedTriangle {
        PackedTriangle::pack_from(self.face, self.plane, self.uv, self.material)
    }
}

/// Packs, unpacks and compares every field. Vertices are compared against the
/// crate's own projection, so the test doesn't assume a particular axis permutation.
#[track_caller]
fn assert_roundtrip(f: Fields) {
    let t = f.pack();
    let layer = SurfaceLayer::new(f.face.axis(), f.plane);
    let expected = f.uv.map(|uv| xyz(Coords3D::from_2d_projection(layer, uv)));

    assert_eq!(
        t.vertices().map(xyz),
        expected,
        "vertices of {f:?}, raw {:#018x}",
        t.0
    );
    assert_eq!(
        t.material(),
        f.material,
        "material of {f:?}, raw {:#018x}",
        t.0
    );
    assert_eq!(t.plane(), f.plane, "plane of {f:?}, raw {:#018x}", t.0);
    assert_eq!(
        t.axis().as_usize(),
        f.face.axis().as_usize(),
        "axis of {f:?}, raw {:#018x}",
        t.0
    );
    assert_eq!(t.face(), f.face, "face of {f:?}, raw {:#018x}", t.0);
}

/// Tiny deterministic PRNG (xorshift64*), no dependencies.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn fields(&mut self) -> Fields {
        let mut uv = [(0u8, 0u8); 3];
        for p in &mut uv {
            *p = (
                self.below(MAX_UV as u64 + 1) as u8,
                self.below(MAX_UV as u64 + 1) as u8,
            );
        }
        Fields {
            face: faces()[self.below(6) as usize],
            plane: self.below(MAX_PLANE as u64 + 1) as u8,
            uv,
            material: self.next() as MaterialId,
        }
    }
}

// ---------------------------------------------------------------------------
// Layout. Needs private access: only compiles as a child of the triangle module.
// ---------------------------------------------------------------------------
mod layout {
    use super::*;

    const _: () = assert!(PackedTriangle::FACE_SHIFT == 0);
    const _: () = assert!(
        PackedTriangle::AXIS_SHIFT == PackedTriangle::FACE_SHIFT + PackedTriangle::FACE_BITS
    );
    const _: () = assert!(
        PackedTriangle::MAT_SHIFT == PackedTriangle::AXIS_SHIFT + PackedTriangle::AXIS_BITS
    );
    const _: () =
        assert!(PackedTriangle::PLANE_SHIFT == PackedTriangle::MAT_SHIFT + MaterialId::BITS);
    const _: () = assert!(PackedTriangle::V0U + PackedTriangle::UV_BITS <= 64);
    // The layout is meant to hold 32 as an in-plane value and 31 as a plane.
    const _: () = assert!(PackedTriangle::UV_MASK >= 32);
    const _: () = assert!(PackedTriangle::PLANE_MASK >= 31);

    #[test]
    fn size_is_one_u64() {
        assert_eq!(core::mem::size_of::<PackedTriangle>(), 8);
        assert_eq!(core::mem::align_of::<PackedTriangle>(), 8);
    }

    #[test]
    fn uses_exactly_60_bits() {
        assert_eq!(PackedTriangle::V0U + PackedTriangle::UV_BITS, 60);
        for face in faces() {
            let raw = Fields::full(face).pack().0;
            assert_eq!(
                raw >> 60,
                0,
                "spare top bits must stay zero, raw {raw:#018x}"
            );
        }
    }

    #[test]
    fn all_zero_fields_pack_to_zero_except_face_and_axis() {
        for face in faces() {
            let raw = Fields::zero(face).pack().0;
            let flag_axis_mask =
                (1u64 << (PackedTriangle::FACE_BITS + PackedTriangle::AXIS_BITS)) - 1;
            assert_eq!(raw & !flag_axis_mask, 0, "{face:?}, raw {raw:#018x}");
        }
    }

    /// Each field occupies exactly its own bit range: setting it to all ones
    /// must change those bits and nothing else.
    #[test]
    fn fields_occupy_disjoint_ranges() {
        let ranges: [(&str, u32, u32); 10] = [
            (
                "face",
                PackedTriangle::FACE_SHIFT,
                PackedTriangle::FACE_BITS,
            ),
            (
                "axis",
                PackedTriangle::AXIS_SHIFT,
                PackedTriangle::AXIS_BITS,
            ),
            ("material", PackedTriangle::MAT_SHIFT, MaterialId::BITS),
            (
                "plane",
                PackedTriangle::PLANE_SHIFT,
                PackedTriangle::PLANE_BITS,
            ),
            ("v2v", PackedTriangle::V2V, PackedTriangle::UV_BITS),
            ("v2u", PackedTriangle::V2U, PackedTriangle::UV_BITS),
            ("v1v", PackedTriangle::V1V, PackedTriangle::UV_BITS),
            ("v1u", PackedTriangle::V1U, PackedTriangle::UV_BITS),
            ("v0v", PackedTriangle::V0V, PackedTriangle::UV_BITS),
            ("v0u", PackedTriangle::V0U, PackedTriangle::UV_BITS),
        ];
        let mut seen = 0u64;
        for (name, shift, bits) in ranges {
            let mask = ((1u64 << bits) - 1) << shift;
            assert_eq!(seen & mask, 0, "{name} overlaps an earlier field");
            seen |= mask;
        }
        assert_eq!(
            seen,
            (1u64 << 60) - 1,
            "fields must tile bits 0..60 with no gaps"
        );
    }
}

// ---------------------------------------------------------------------------
// Round trips
// ---------------------------------------------------------------------------
mod roundtrip {
    use super::*;

    #[test]
    fn simple_every_face() {
        for face in faces() {
            assert_roundtrip(Fields {
                face,
                plane: 3,
                uv: [(1, 2), (3, 4), (5, 6)],
                material: 42,
            });
        }
    }

    #[test]
    fn all_zero() {
        for face in faces() {
            assert_roundtrip(Fields::zero(face));
        }
    }

    #[test]
    fn all_max() {
        for face in faces() {
            assert_roundtrip(Fields::full(face));
        }
    }

    /// Regression for the 5-bit layout: in-plane coordinate 32 must survive and
    /// must not corrupt its neighbours.
    #[test]
    fn boundary_coordinate_32() {
        for face in faces() {
            for plane in [0, MAX_PLANE] {
                for uv in [
                    [(0, 0), (32, 0), (32, 32)],
                    [(0, 32), (0, 0), (32, 0)],
                    [(32, 32), (32, 32), (32, 32)],
                    [(32, 0), (0, 32), (32, 0)],
                ] {
                    assert_roundtrip(Fields {
                        face,
                        plane,
                        uv,
                        material: 7,
                    });
                    assert_roundtrip(Fields {
                        face,
                        plane,
                        uv,
                        material: MaterialId::MAX,
                    });
                }
            }
        }
    }

    #[test]
    fn every_plane_value() {
        for face in faces() {
            for plane in 0..=MAX_PLANE {
                assert_roundtrip(Fields {
                    plane,
                    ..Fields::zero(face)
                });
                assert_roundtrip(Fields {
                    plane,
                    ..Fields::full(face)
                });
            }
        }
    }

    #[test]
    fn every_uv_value_in_every_slot() {
        for face in faces() {
            for value in 0..=MAX_UV {
                for slot in 0..6 {
                    for base in [Fields::zero(face), Fields::full(face)] {
                        let mut f = base;
                        let p = &mut f.uv[slot / 2];
                        if slot % 2 == 0 {
                            p.0 = value;
                        } else {
                            p.1 = value;
                        }
                        assert_roundtrip(f);
                    }
                }
            }
        }
    }

    #[test]
    fn every_single_material_bit_and_edge_values() {
        for face in faces() {
            for bit in 0..MaterialId::BITS {
                let material = (1 as MaterialId) << bit;
                assert_roundtrip(Fields {
                    material,
                    ..Fields::zero(face)
                });
                assert_roundtrip(Fields {
                    material,
                    ..Fields::full(face)
                });
                // and its complement
                assert_roundtrip(Fields {
                    material: !material,
                    ..Fields::zero(face)
                });
                assert_roundtrip(Fields {
                    material: !material,
                    ..Fields::full(face)
                });
            }
        }
    }

    #[test]
    fn random() {
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
        for _ in 0..100_000 {
            assert_roundtrip(rng.fields());
        }
    }
}

// ---------------------------------------------------------------------------
// Field isolation: changing one field must not change the others
// ---------------------------------------------------------------------------
mod isolation {
    use super::*;

    #[test]
    fn axis_ignores_higher_fields() {
        for face in faces() {
            let t = Fields::full(face).pack();
            assert_eq!(
                t.axis().as_usize(),
                face.axis().as_usize(),
                "{face:?}, raw {:#018x}",
                t.0
            );
        }
    }

    #[test]
    fn plane_and_axis_do_not_bleed() {
        for face in faces() {
            for plane in [0, 1, 15, 16, MAX_PLANE] {
                let t = Fields {
                    plane,
                    ..Fields::full(face)
                }
                .pack();
                assert_eq!(t.plane(), plane, "{face:?}");
                assert_eq!(t.axis().as_usize(), face.axis().as_usize(), "{face:?}");
                assert_eq!(t.material(), MaterialId::MAX, "{face:?}");
            }
        }
    }

    #[test]
    fn material_ignores_neighbours() {
        for face in faces() {
            for material in [0, 1, 0x5555, 0xAAAA, 0x8000, MaterialId::MAX] {
                let a = Fields {
                    material,
                    ..Fields::zero(face)
                }
                .pack();
                let b = Fields {
                    material,
                    ..Fields::full(face)
                }
                .pack();
                assert_eq!(a.material(), material, "{face:?}");
                assert_eq!(b.material(), material, "{face:?}");
            }
        }
    }

    #[test]
    fn geometry_does_not_touch_material() {
        let mut rng = Rng(0xDEAD_BEEF_CAFE_F00D);
        for _ in 0..10_000 {
            let f = rng.fields();
            for material in [0, MaterialId::MAX] {
                assert_eq!(Fields { material, ..f }.pack().material(), material);
            }
        }
    }

    /// Only `plane`, `uv` and `material` vary; the axis stays fixed.
    #[test]
    fn same_inputs_same_triangle() {
        let mut rng = Rng(1);
        for _ in 0..1_000 {
            let f = rng.fields();
            assert_eq!(f.pack().0, f.pack().0);
        }
    }
}

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------
mod geometry {
    use super::*;

    #[test]
    fn all_vertices_lie_on_the_plane() {
        let mut rng = Rng(77);
        for _ in 0..10_000 {
            let f = rng.fields();
            let axis = f.face.axis();
            for v in f.pack().vertices() {
                assert_eq!(along(v, axis), f.plane, "{f:?}");
            }
        }
    }

    #[test]
    fn in_plane_coordinates_are_preserved() {
        // With distinct u/v per vertex, the other two coordinates of vertex i
        // must be exactly {u_i, v_i} in some order.
        let f = Fields {
            face: Face::Positive(Axis::Y),
            plane: 9,
            uv: [(1, 2), (3, 4), (5, 6)],
            material: 1,
        };
        let axis = f.face.axis();
        for (v, (u, w)) in f.pack().vertices().into_iter().zip(f.uv) {
            let mut others = [v.x, v.y, v.z]
                .into_iter()
                .enumerate()
                .filter(|&(i, _)| i != axis.as_usize())
                .map(|(_, c)| c)
                .collect::<Vec<_>>();
            others.sort();
            let mut want = [u, w];
            want.sort();
            assert_eq!(others, want);
        }
    }

    #[test]
    fn vertex_order_is_preserved() {
        let f = Fields {
            face: Face::Negative(Axis::Z),
            plane: 2,
            uv: [(10, 11), (20, 21), (30, 32)],
            material: 5,
        };
        let layer = SurfaceLayer::new(f.face.axis(), f.plane);
        let t = f.pack();
        for i in 0..3 {
            assert_eq!(
                xyz(t.vertices()[i]),
                xyz(Coords3D::from_2d_projection(layer, f.uv[i])),
                "vertex {i}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// from_strip_triangle
// ---------------------------------------------------------------------------
mod from_strip {
    use crate::meshing::triangulation::Triangle2D;

    use super::*;

    /// Adapt to however a `Triangle2D` is built in your crate.
    fn tri2d(vertices: Uv) -> Triangle2D {
        Triangle2D { vertices }
    }

    #[test]
    fn equals_pack_from() {
        let mut rng = Rng(5);
        for _ in 0..5_000 {
            let f = rng.fields();
            let a = PackedTriangle::from_strip_triangle(f.face, f.plane, tri2d(f.uv), f.material);
            assert_eq!(a.0, f.pack().0, "{f:?}");
        }
    }

    #[test]
    fn decodes_to_the_input() {
        for face in faces() {
            let f = Fields {
                face,
                plane: 4,
                uv: [(0, 0), (0, 32), (32, 0)],
                material: 9,
            };
            let t = PackedTriangle::from_strip_triangle(f.face, f.plane, tri2d(f.uv), f.material);
            let layer = SurfaceLayer::new(face.axis(), f.plane);
            assert_eq!(
                t.vertices().map(xyz),
                f.uv.map(|uv| xyz(Coords3D::from_2d_projection(layer, uv)))
            );
            assert_eq!(t.material(), 9);
            assert_eq!(t.plane(), 4);
        }
    }
}

// ---------------------------------------------------------------------------
// face(), face_positive(), outward_vertices(). Delete if these methods are gone.
// ---------------------------------------------------------------------------
mod face_api {
    use super::*;

    #[test]
    fn face_roundtrips() {
        for face in faces() {
            let t = Fields::full(face).pack();
            assert_eq!(t.face(), face);
            assert_eq!(t.is_normal_positive(), face.is_positive());
            assert_eq!(t.face().axis().as_usize(), t.axis().as_usize());
        }
    }

    #[test]
    fn face_and_axis_survive_any_other_fields() {
        let mut rng = Rng(123);
        for _ in 0..10_000 {
            let f = rng.fields();
            let t = f.pack();
            assert_eq!(t.face(), f.face, "{f:?}, raw {:#018x}", t.0);
        }
    }
}

mod normals {
    use super::*;

    /// Sign of the normal along `axis`. Asserts the triangle is perpendicular
    /// to that axis and non-degenerate.
    fn normal_sign(axis: Axis, tri: [Coords3D; 3]) -> i32 {
        let p = |c: Coords3D| [c.x as i32, c.y as i32, c.z as i32];
        let (a, b, c) = (p(tri[0]), p(tri[1]), p(tri[2]));
        let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
        let n = [
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ];
        let ax = axis.as_usize();
        for k in (0..3).filter(|&k| k != ax) {
            assert_eq!(n[k], 0, "not perpendicular to axis {ax}: {tri:?}");
        }
        assert_ne!(n[ax], 0, "degenerate triangle {tri:?}");
        n[ax].signum()
    }

    fn canonical(face: Face) -> PackedTriangle {
        Fields {
            face,
            plane: 3,
            uv: CANONICAL_UV,
            material: 1,
        }
        .pack()
    }

    /// Pins the convention the other tests depend on: the stored triangle, as
    /// decoded by `vertices()`, has this normal sign for every axis.
    #[test]
    fn canonical_winding_has_stored_sign() {
        for face in faces() {
            assert_eq!(
                normal_sign(face.axis(), canonical(face).vertices()),
                STORED_NORMAL_SIGN,
                "{face:?}"
            );
        }
    }

    /// The core test: the outward winding must point along +axis for positive
    /// faces and -axis for negative faces.
    #[test]
    fn outward_vertices_point_away_from_the_solid() {
        for face in faces() {
            let expected = if face.is_positive() { 1 } else { -1 };
            assert_eq!(
                normal_sign(face.axis(), canonical(face).outward_vertices()),
                expected,
                "{face:?}"
            );
        }
    }

    #[test]
    fn outward_vertices_is_a_permutation_of_vertices() {
        let mut rng = Rng(9);
        for _ in 0..2_000 {
            let t = rng.fields().pack();
            let mut a = t.vertices().map(xyz);
            let mut b = t.outward_vertices().map(xyz);
            a.sort();
            b.sort();
            assert_eq!(a, b);
        }
    }

    /// Exactly one of the two signs keeps the stored order and the other swaps it.
    #[test]
    fn only_one_sign_flips_the_order() {
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            let same = |face| {
                let t = canonical(face);
                t.outward_vertices().map(xyz) == t.vertices().map(xyz)
            };
            assert_ne!(
                same(Face::Positive(axis)),
                same(Face::Negative(axis)),
                "{axis:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Debug-only input validation (the packer only uses debug_assert!)
// ---------------------------------------------------------------------------
#[cfg(debug_assertions)]
mod invalid_input {
    use super::*;

    #[test]
    #[should_panic]
    fn plane_32_is_rejected() {
        Fields {
            plane: MAX_PLANE + 1,
            ..Fields::zero(Face::Positive(Axis::X))
        }
        .pack();
    }

    #[test]
    #[should_panic]
    fn uv_out_of_range_is_rejected() {
        let mut f = Fields::zero(Face::Positive(Axis::X));
        f.uv[2].1 = PackedTriangle::UV_MASK as u8 + 1; // 64: doesn't fit 6 bits
        f.pack();
    }
}

#[test]
fn diagnose_sign_bit() {
    for face in faces() {
        let t = Fields::zero(face).pack();
        eprintln!(
            "{face:?}: is_positive={} raw_bit0={} face_positive()={} face()={:?}",
            face.is_positive(),
            t.0 & 1,
            t.is_normal_positive(),
            t.face(),
        );
        assert_eq!(
            t.0 & 1 != 0,
            face.is_positive(),
            "packer wrote the wrong bit for {face:?}"
        );
        assert_eq!(t.is_normal_positive(), face.is_positive(), "{face:?}");
        assert_eq!(t.axis().as_usize(), face.axis().as_usize(), "{face:?}");
        assert_eq!(t.face().is_positive(), face.is_positive(), "{face:?}");
        assert_eq!(
            t.face().axis().as_usize(),
            face.axis().as_usize(),
            "{face:?}"
        );
    }
}
