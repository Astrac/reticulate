use crate::{
    Axis, CHUNK_VOLUME, Coords2D, PLANE_AREA, PLANE_SIDE, TREE_DEPTH,
    meshing::{
        bitfields::{Bitfields, BitfieldsStorage},
        surface_layer::SurfaceLayer,
    },
};

#[test]
fn chunk_dimensions_derive_from_the_tree_depth() {
    assert_eq!(PLANE_SIDE, 1 << TREE_DEPTH);
    assert_eq!(PLANE_AREA, PLANE_SIDE * PLANE_SIDE);
    assert_eq!(CHUNK_VOLUME, PLANE_AREA * PLANE_SIDE);
}

#[test]
fn fresh_storage_is_entirely_empty() {
    let mut storage = BitfieldsStorage::default();
    let bitfields = Bitfields::from_storage(&mut storage);

    for axis in Axis::AXES {
        assert!(
            bitfields.material.planes[axis.as_usize()]
                .iter()
                .all(|x| *x == 0)
        );

        for layer in 0..PLANE_SIDE {
            assert!(
                bitfields.parity.masks[axis.as_usize()]
                    .get_layer(layer)
                    .iter()
                    .all(|&w| w == 0)
            );
            for u in 0..PLANE_SIDE {
                assert_eq!(
                    bitfields
                        .claims
                        .claimed_row(SurfaceLayer::new(axis, layer as u8), u),
                    0
                );
            }
        }
    }
}

#[test]
fn views_do_not_overlap() {
    let mut storage = BitfieldsStorage::default();
    let mut bitfields = Bitfields::from_storage(&mut storage);

    // Fill the last plane of every claims mask and the last layer of every
    // parity mask, which are the regions next to their neighbours in memory.
    for axis in Axis::AXES {
        let last = SurfaceLayer::new(axis, PLANE_SIDE as u8 - 1);
        bitfields.claims.claim(last, &[u32::MAX; PLANE_SIDE]);
        for u in 0..PLANE_SIDE as u8 {
            for v in 0..PLANE_SIDE as u8 {
                bitfields.parity.set_transition(last, Coords2D { u, v }, 1);
            }
        }
    }
    for axis in Axis::AXES {
        bitfields.material.planes[axis.as_usize()].fill(u16::MAX);
    }

    // Each view must read back exactly what was written to it: the claims of
    // every plane but the last are untouched, and so is the parity below the
    // last layer bit.
    for axis in Axis::AXES {
        for layer in 0..PLANE_SIDE as u8 - 1 {
            assert_eq!(
                bitfields
                    .claims
                    .claimed_row(SurfaceLayer::new(axis, layer), 0),
                0,
                "{axis:?} claims plane {layer} was touched"
            );
        }
        for layer in 0..PLANE_SIDE {
            for word in bitfields.parity.masks[axis.as_usize()].get_layer(layer) {
                assert_eq!(*word, 1u32 << (PLANE_SIDE - 1), "{axis:?} parity {layer}");
            }
        }
    }
}

#[test]
fn storage_is_reused_across_views() {
    let mut storage = BitfieldsStorage::default();
    {
        let mut bitfields = Bitfields::from_storage(&mut storage);
        bitfields
            .parity
            .set_transition(SurfaceLayer::new(Axis::Z, 4), Coords2D { u: 1, v: 2 }, 1);
    }

    let bitfields = Bitfields::from_storage(&mut storage);
    assert_eq!(
        bitfields.parity.masks[Axis::Z.as_usize()].get_layer(1)[2],
        1 << 4
    );
}
