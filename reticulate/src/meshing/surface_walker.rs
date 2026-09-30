use crate::{
    DenseWalker, Morton3D,
    axis::Axis,
    face::Face,
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
    fn on_branch_start(&mut self, _morton: Morton3D, _children_mask: u8) {}
    fn on_branch_end(&mut self, _morton: Morton3D) {}

    fn on_leaf(&mut self, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);

        self.surface_scanner
            .scan(Face::Negative(Axis::X), morton, material, &self.materials);
        self.surface_scanner
            .scan(Face::Negative(Axis::Y), morton, material, &self.materials);
        self.surface_scanner
            .scan(Face::Negative(Axis::Z), morton, material, &self.materials);

        self.materials.fill(Axis::X, morton, material);
        self.materials.fill(Axis::Y, morton, material);
        self.materials.fill(Axis::Z, morton, material);
    }

    fn on_empty(&mut self, morton: Morton3D) {
        let material = 0;

        self.surface_scanner
            .scan(Face::Positive(Axis::X), morton, 0, &self.materials);
        self.surface_scanner
            .scan(Face::Positive(Axis::Y), morton, 0, &self.materials);
        self.surface_scanner
            .scan(Face::Positive(Axis::Z), morton, 0, &self.materials);

        self.materials.fill(Axis::X, morton, material);
        self.materials.fill(Axis::Y, morton, material);
        self.materials.fill(Axis::Z, morton, material);
    }
}
