use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    face::*,
    meshing::{
        Triangle,
        bitfields::{Material, MaterialId},
        shaper::Shaper,
    },
};

pub struct ShapingWalker<'a, M, F> {
    materials: Material<'a>,
    shaper: Shaper<'a, F>,
    to_material: M,
}

impl<'a, M, F> ShapingWalker<'a, M, F>
where
    F: FnMut(Triangle),
{
    pub fn new(
        materials: Material<'a>,
        shaper: Shaper<'a, F>,
        to_material: M,
    ) -> ShapingWalker<'a, M, F> {
        ShapingWalker {
            materials,
            shaper,
            to_material,
        }
    }
}

impl<'a, M, F, Data> DenseWalker<Data> for ShapingWalker<'a, M, F>
where
    F: FnMut(Triangle),
    M: Fn(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}
    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);

        self.shaper
            .process(FACE_X_N, morton, depth, material, &self.materials);
        self.shaper
            .process(FACE_Y_N, morton, depth, material, &self.materials);
        self.shaper
            .process(FACE_Z_N, morton, depth, material, &self.materials);

        self.materials.fill(AXIS_X, morton, depth, material);
        self.materials.fill(AXIS_Y, morton, depth, material);
        self.materials.fill(AXIS_Z, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        let material = 0;

        self.shaper
            .process(FACE_X_P, morton, depth, 0, &self.materials);
        self.shaper
            .process(FACE_Y_P, morton, depth, 0, &self.materials);
        self.shaper
            .process(FACE_Z_P, morton, depth, 0, &self.materials);

        self.materials.fill(AXIS_X, morton, depth, material);
        self.materials.fill(AXIS_Y, morton, depth, material);
        self.materials.fill(AXIS_Z, morton, depth, material);
    }
}
