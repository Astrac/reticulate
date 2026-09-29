use crate::{
    DenseWalk,
    meshing::{
        Triangle,
        bitfields::{Bitfields, BitfieldsStorage, MaterialId},
        gathering_walker::GatheringWalker,
        surface_walker::SurfaceWalker,
    },
};

#[derive(Debug, Default)]
pub struct Triangulator {
    storage: BitfieldsStorage,
}

impl Triangulator {
    pub fn new() -> Triangulator {
        Triangulator {
            storage: BitfieldsStorage::default(),
        }
    }

    pub fn triangulate<Data, F, M>(
        &mut self,
        tree: impl DenseWalk<Data>,
        mut to_material: M,
        mut on_triangle: F,
    ) where
        Data: Copy + PartialEq,
        M: Fn(&Data) -> MaterialId,
        F: FnMut(Triangle),
    {
        let bitfields = Bitfields::from_storage(&mut self.storage);
        let mut gathering_walker = GatheringWalker::new(bitfields, &mut to_material);
        tree.walk_dense(&mut gathering_walker);

        let (mut materials, surface_scanner) =
            Bitfields::from_storage(&mut self.storage).as_surface_scanner(&mut on_triangle);

        materials.clear();

        let mut surface_walker = SurfaceWalker::new(materials, surface_scanner, &mut to_material);
        tree.walk_dense(&mut surface_walker);
    }
}
