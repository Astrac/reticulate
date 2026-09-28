use crate::{
    Depth, Morton3D,
    face::Face,
    meshing::{
        Triangle,
        bitfields::{Claims, Material, MaterialId, Parity},
        triangulation::{discoverer::Discoverer, scanner::Scanner},
    },
};

pub struct Shaper<'a, F> {
    claims: Claims<'a>,
    parity: Parity<'a>,
    on_triangle: F,
}

impl<'a, F> Shaper<'a, F>
where
    F: FnMut(Triangle),
{
    #[inline(always)]
    pub const fn new(claims: Claims<'a>, parity: Parity<'a>, on_triangle: F) -> Shaper<'a, F> {
        Shaper {
            claims,
            parity,
            on_triangle,
        }
    }

    pub fn process(
        &mut self,
        face: Face,
        morton: Morton3D,
        depth: Depth,
        current_material: MaterialId,
        materials: &Material<'a>,
    ) {
        let axis = face.axis();
        let (morton, axis_coord) = morton.projected_axis_morton(axis, depth);
        let discoverer = Discoverer::new(&self.parity, axis, axis_coord);
        let scanner = Scanner::new(discoverer);

        materials.iterate_materials(axis, morton, depth, |material, coords, _| {
            if material == current_material || !self.claims.is_pending(axis, axis_coord, coords) {
                return;
            }
            let face_material = material.max(current_material);

            let filled = scanner.scan_seed(&self.claims, coords, |t| {
                (self.on_triangle)(Triangle::from_strip_triangle(
                    face,
                    axis_coord as u8,
                    t,
                    face_material,
                ))
            });

            #[cfg(debug_assertions)]
            scanner.discoverer.debug_assert_within_footprint(&filled);

            self.claims.mark_done(axis, axis_coord, &filled);
        });
    }
}
