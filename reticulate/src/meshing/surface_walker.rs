use crate::{
    DenseWalker, Depth, Morton3D,
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
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}
    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);

        self.surface_scanner.scan(
            Face::Negative(Axis::X),
            morton,
            depth,
            material,
            &self.materials,
        );
        self.surface_scanner.scan(
            Face::Negative(Axis::Y),
            morton,
            depth,
            material,
            &self.materials,
        );
        self.surface_scanner.scan(
            Face::Negative(Axis::Z),
            morton,
            depth,
            material,
            &self.materials,
        );

        self.materials.fill(Axis::X, morton, depth, material);
        self.materials.fill(Axis::Y, morton, depth, material);
        self.materials.fill(Axis::Z, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        let material = 0;

        self.surface_scanner
            .scan(Face::Positive(Axis::X), morton, depth, 0, &self.materials);
        self.surface_scanner
            .scan(Face::Positive(Axis::Y), morton, depth, 0, &self.materials);
        self.surface_scanner
            .scan(Face::Positive(Axis::Z), morton, depth, 0, &self.materials);

        self.materials.fill(Axis::X, morton, depth, material);
        self.materials.fill(Axis::Y, morton, depth, material);
        self.materials.fill(Axis::Z, morton, depth, material);
    }
}
