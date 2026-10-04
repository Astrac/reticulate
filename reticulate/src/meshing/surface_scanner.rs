#[cfg(debug_assertions)]
use crate::utils::debug_assertions::debug_assert_within_footprint;
use crate::{
    Face, MaterialId, Morton3D,
    meshing::{
        Triangle,
        bitfields::{Claims, Material, Parity},
        surface_layer::SurfaceLayer,
        triangulation::{edges::Edges, filler::Filler, weaver::Weaver},
    },
};

pub struct SurfaceScanner<'a, 'b, F> {
    claims: &'b mut Claims<'a>,
    parity: &'b Parity<'a>,
    on_triangle: F,
}

impl<'a, 'b, F> SurfaceScanner<'a, 'b, F>
where
    F: FnMut(Triangle),
{
    #[inline(always)]
    pub const fn new(
        claims: &'b mut Claims<'a>,
        parity: &'b Parity<'a>,
        on_triangle: F,
    ) -> SurfaceScanner<'a, 'b, F> {
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
        let edges = Edges::from_parity(&self.parity, layer);
        let scanner = Weaver::new(&self.parity);

        materials.iterate_materials(axis, morton, |material, coords, _| {
            // TODO - Find a way to move these out of the closure without
            //        breaking the borrow-checker
            let discoverer = Filler::new(&self.claims, &self.parity);

            if material == current_material
                || (material != 0 && current_material != 0)
                || !self.claims.is_unclaimed(layer, coords)
            {
                return;
            }
            let face_material = material.max(current_material);

            let filled = discoverer.fill_region(layer, &edges, coords);
            scanner.weave_region(layer, &filled, |t| {
                (self.on_triangle)(Triangle::from_strip_triangle(
                    face,
                    layer.index,
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
