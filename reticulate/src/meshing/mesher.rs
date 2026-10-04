use crate::{
    DenseWalk, MaterialId,
    meshing::{
        Triangle,
        bitfields::{Bitfields, BitfieldsStorage},
        gathering_walker::GatheringWalker,
        surface_scanner::SurfaceScanner,
        surface_walker::SurfaceWalker,
    },
};

#[derive(Debug, Default)]
pub struct Mesher {
    storage: BitfieldsStorage,
}

impl Mesher {
    pub fn new() -> Mesher {
        Mesher {
            storage: BitfieldsStorage::default(),
        }
    }

    pub fn make_triangles<Data, F, M>(
        &mut self,
        tree: &impl DenseWalk<Data>,
        mut to_material: M,
        mut on_triangle: F,
    ) where
        Data: PartialEq,
        M: FnMut(&Data) -> MaterialId,
        F: FnMut(Triangle),
    {
        let Bitfields {
            mut claims,
            mut parity,
            mut material,
        } = Bitfields::from_storage(&mut self.storage);

        let mut gathering_walker =
            GatheringWalker::new(&mut material, &mut parity, &mut to_material);
        tree.walk_dense(&mut gathering_walker);

        material.clear();

        let surface_scanner = SurfaceScanner::new(&mut claims, &parity, &mut on_triangle);
        let mut surface_walker =
            SurfaceWalker::new(&mut material, surface_scanner, &mut to_material);

        tree.walk_dense(&mut surface_walker);
    }
}
