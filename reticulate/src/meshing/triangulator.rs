use crate::{
    DenseWalk,
    meshing::{
        Triangle,
        bitfields::{Bitfields, BitfieldsStorage, MaterialId},
        gathering_walker::GatheringWalker,
        shaping_walker::ShapingWalker,
    },
};

pub struct Triangulator<F> {
    storage: BitfieldsStorage,
    on_triangle: F,
}

impl<F: FnMut(Triangle)> Triangulator<F> {
    pub fn new(on_triangle: F) -> Triangulator<F> {
        Triangulator {
            storage: BitfieldsStorage::default(),
            on_triangle,
        }
    }

    pub fn triangulate<Data, M>(&mut self, tree: impl DenseWalk<Data>, to_material: M)
    where
        Data: Copy + PartialEq,
        M: Copy + Fn(&Data) -> MaterialId, // TODO - Remove Copy here
        F: FnMut(Triangle),
    {
        let bitfields = Bitfields::from_storage(&mut self.storage);
        let mut parity_walker = GatheringWalker::new(bitfields, to_material);
        tree.walk_dense(&mut parity_walker);

        let (mut materials, shaper) =
            Bitfields::from_storage(&mut self.storage).as_shaper(&mut self.on_triangle);

        materials.clear();

        let mut shape_walker = ShapingWalker::new(materials, shaper, to_material);
        tree.walk_dense(&mut shape_walker);
    }
}
