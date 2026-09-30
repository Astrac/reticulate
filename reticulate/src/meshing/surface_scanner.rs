#[cfg(debug_assertions)]
use crate::utils::debug_assertions::debug_assert_within_footprint;
use crate::{
    Morton3D,
    face::Face,
    meshing::{
        Triangle,
        bitfields::{Claims, Material, MaterialId, Parity},
        surface_layer::SurfaceLayer,
        triangulation::{discoverer::Discoverer, weaver::Weaver},
    },
};

pub struct SurfaceScanner<'a, F> {
    claims: Claims<'a>,
    parity: Parity<'a>,
    on_triangle: F,
}

impl<'a, F> SurfaceScanner<'a, F>
where
    F: FnMut(Triangle),
{
    #[inline(always)]
    pub const fn new(
        claims: Claims<'a>,
        parity: Parity<'a>,
        on_triangle: F,
    ) -> SurfaceScanner<'a, F> {
        SurfaceScanner {
            claims,
            parity,
            on_triangle,
        }
    }

    pub fn scan(
        &mut self,
        face: Face,
        morton: Morton3D,
        current_material: MaterialId,
        materials: &Material<'a>,
    ) {
        let axis = face.axis();
        let (morton, axis_coord) = morton.projected_axis_morton(axis);
        let layer = SurfaceLayer::new(axis, axis_coord);
        let discoverer = Discoverer::new(&self.parity, layer);
        let scanner = Weaver::new(discoverer);

        materials.iterate_materials(axis, morton, |material, coords, _| {
            if material == current_material
                || (material != 0 && current_material != 0)
                || !self.claims.is_unclaimed(layer, coords)
            {
                return;
            }
            let face_material = material.max(current_material);

            let filled = scanner.from_seed(&self.claims, layer, coords, |t| {
                (self.on_triangle)(Triangle::from_strip_triangle(
                    face,
                    layer.layer as u8,
                    t,
                    face_material,
                ))
            });

            #[cfg(debug_assertions)]
            debug_assert_within_footprint(&self.parity, layer, &filled);

            self.claims.claim(layer, &filled);
        });
    }
}
