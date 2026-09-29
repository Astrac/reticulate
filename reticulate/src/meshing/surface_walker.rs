use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    face::*,
    meshing::{
        Triangle,
        bitfields::{Material, MaterialId},
        surface_scanner::SurfaceScanner,
    },
};

pub struct SurfaceWalker<'a, M, F> {
    materials: Material<'a>,
    surface_scanner: SurfaceScanner<'a, F>,
    to_material: &'a mut M,
}

impl<'a, M, F> SurfaceWalker<'a, M, F>
where
    F: FnMut(Triangle),
{
    pub fn new(
        materials: Material<'a>,
        surface_scanner: SurfaceScanner<'a, F>,
        to_material: &'a mut M,
    ) -> SurfaceWalker<'a, M, F> {
        SurfaceWalker {
            materials,
            surface_scanner,
            to_material,
        }
    }
}

impl<'a, M, F, Data> DenseWalker<Data> for SurfaceWalker<'a, M, F>
where
    F: FnMut(Triangle),
    M: Fn(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}
    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);

        self.surface_scanner
            .scan(FACE_X_N, morton, depth, material, &self.materials);
        self.surface_scanner
            .scan(FACE_Y_N, morton, depth, material, &self.materials);
        self.surface_scanner
            .scan(FACE_Z_N, morton, depth, material, &self.materials);

        self.materials.fill(AXIS_X, morton, depth, material);
        self.materials.fill(AXIS_Y, morton, depth, material);
        self.materials.fill(AXIS_Z, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        let material = 0;

        self.surface_scanner
            .scan(FACE_X_P, morton, depth, 0, &self.materials);
        self.surface_scanner
            .scan(FACE_Y_P, morton, depth, 0, &self.materials);
        self.surface_scanner
            .scan(FACE_Z_P, morton, depth, 0, &self.materials);

        self.materials.fill(AXIS_X, morton, depth, material);
        self.materials.fill(AXIS_Y, morton, depth, material);
        self.materials.fill(AXIS_Z, morton, depth, material);
    }
}
