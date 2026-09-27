use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    face::*,
    meshing::{
        Triangle,
        bitfields::{Bitfields, MaterialId},
        triangulation::scanner::Scanner,
    },
};

pub struct ClaimingWalker<'a, M, F> {
    bitfields: Bitfields<'a>,
    to_material: M,
    on_triangle: &'a mut F,
}

impl<'a, M, F> ClaimingWalker<'a, M, F>
where
    F: FnMut(Triangle),
{
    pub fn new(
        bitfields: Bitfields<'a>,
        to_material: M,
        on_triangle: &'a mut F,
    ) -> ClaimingWalker<'a, M, F> {
        ClaimingWalker {
            bitfields,
            on_triangle,
            to_material,
        }
    }

    fn emit_shapes(
        &mut self,
        face: Face,
        morton: Morton3D,
        depth: Depth,
        current_material: MaterialId,
    ) {
        let axis = face.axis();
        let (morton, axis_coord) = morton.projected_axis_morton(axis, depth);
        let Bitfields {
            material,
            parity,
            claims,
            ..
        } = &mut self.bitfields;
        let scanner = Scanner::new(parity, axis, axis_coord);

        material.iterate_materials(axis, morton, depth, |material, coords, _| {
            if material == current_material || !claims.is_pending(axis, axis_coord, coords) {
                return;
            }
            let face_material = material.max(current_material);

            let filled = scanner.scan_seed(claims, coords, |t| {
                (self.on_triangle)(Triangle::from_strip_triangle(
                    face,
                    axis_coord as u8,
                    t,
                    face_material,
                ))
            });

            scanner.debug_assert_within_footprint(&filled);
            claims.mark_done(axis, axis_coord, &filled);
        });
    }
}

impl<'a, M, F, Data> DenseWalker<Data> for ClaimingWalker<'a, M, F>
where
    F: FnMut(Triangle),
    M: Fn(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}
    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);

        self.emit_shapes(FACE_X_N, morton, depth, material);
        self.emit_shapes(FACE_Y_N, morton, depth, material);
        self.emit_shapes(FACE_Z_N, morton, depth, material);

        self.bitfields
            .material
            .fill(AXIS_X, morton, depth, material);
        self.bitfields
            .material
            .fill(AXIS_Y, morton, depth, material);
        self.bitfields
            .material
            .fill(AXIS_Z, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        let material = 0;

        self.emit_shapes(FACE_X_P, morton, depth, 0);
        self.emit_shapes(FACE_Y_P, morton, depth, 0);
        self.emit_shapes(FACE_Z_P, morton, depth, 0);

        self.bitfields
            .material
            .fill(AXIS_X, morton, depth, material);
        self.bitfields
            .material
            .fill(AXIS_Y, morton, depth, material);
        self.bitfields
            .material
            .fill(AXIS_Z, morton, depth, material);
    }
}
