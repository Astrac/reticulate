use crate::{
    Axis, DenseWalker, Face, MaterialId, Morton3D,
    meshing::{Triangle, bitfields::Material, surface_scanner::SurfaceScanner},
};

pub struct SurfaceWalker<'a, 'b, M, F> {
    materials: &'b mut Material<'a>,
    surface_scanner: SurfaceScanner<'a, 'b, F>,
    to_material: &'b mut M,
}

impl<'a, 'b, M, F> SurfaceWalker<'a, 'b, M, F>
where
    F: FnMut(Triangle),
{
    pub fn new(
        materials: &'b mut Material<'a>,
        surface_scanner: SurfaceScanner<'a, 'b, F>,
        to_material: &'b mut M,
    ) -> SurfaceWalker<'a, 'b, M, F> {
        SurfaceWalker {
            materials,
            surface_scanner,
            to_material,
        }
    }

    #[inline(always)]
    fn scan_and_fill(&mut self, axis: Axis, face: Face, morton: Morton3D, material: MaterialId) {
        let (morton, layer) = morton.projected_axis_morton(axis);

        if material == 0 && self.materials.is_vacant(axis, morton) {
            return;
        }

        self.surface_scanner
            .scan_projected(face, morton, layer, material, &*self.materials);
        self.materials.fill_projected(axis, morton, material);
    }
}

impl<'a, 'b, M, F, Data> DenseWalker<Data> for SurfaceWalker<'a, 'b, M, F>
where
    F: FnMut(Triangle),
    M: FnMut(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _morton: Morton3D, _children_mask: u8) {}
    fn on_branch_end(&mut self, _morton: Morton3D) {}

    fn on_leaf(&mut self, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);
        self.scan_and_fill(Axis::X, Face::Negative(Axis::X), morton, material);
        self.scan_and_fill(Axis::Y, Face::Negative(Axis::Y), morton, material);
        self.scan_and_fill(Axis::Z, Face::Negative(Axis::Z), morton, material);
    }

    fn on_empty(&mut self, morton: Morton3D) {
        self.scan_and_fill(Axis::X, Face::Positive(Axis::X), morton, 0);
        self.scan_and_fill(Axis::Y, Face::Positive(Axis::Y), morton, 0);
        self.scan_and_fill(Axis::Z, Face::Positive(Axis::Z), morton, 0);
    }
}
