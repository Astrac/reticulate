use crate::{
    DenseWalk,
    meshing::{
        Triangle,
        bitfields::{Bitfields, BitfieldsStorage, MaterialId},
        parity_walker::ParityWalker,
        shape_walker::ShapeWalker,
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
        M: Copy + Fn(&Data) -> MaterialId,
        F: FnMut(Triangle),
    {
        let bitfields = Bitfields::from_storage(&mut self.storage);
        let mut parity_walker = ParityWalker::new(bitfields, to_material);
        tree.walk_dense(&mut parity_walker);

        let mut bitfields = Bitfields::from_storage(&mut self.storage);
        bitfields.material.clear_material_planes();

        let mut shape_walker = ShapeWalker::new(bitfields, to_material, &mut self.on_triangle);
        tree.walk_dense(&mut shape_walker);
    }
}
