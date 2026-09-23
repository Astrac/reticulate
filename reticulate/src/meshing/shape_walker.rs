use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    face::*,
    meshing::{
        Triangle,
        bitfields::{Bitfields, MaterialId},
        face::strip_weaver::StripWeaver,
    },
};

pub struct ShapeWalker<'a, M, F> {
    bitfields: Bitfields<'a>,
    to_material: M,
    on_triangle: &'a mut F,
}

impl<'a, M, F> ShapeWalker<'a, M, F>
where
    F: FnMut(Triangle),
{
    pub fn new(
        bitfields: Bitfields<'a>,
        to_material: M,
        on_triangle: &'a mut F,
    ) -> ShapeWalker<'a, M, F> {
        ShapeWalker {
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
        let axis_u = axis.next();
        let axis_v = axis_u.next();
        let (morton, axis_coord) = morton.projected_axis_morton(axis, depth);

        self.bitfields
            .material
            .iterate_materials(axis, morton, depth, |material, coords, _| {
                if self
                    .bitfields
                    .occlusion
                    .is_pending(axis, axis_coord, coords)
                    && current_material != material
                {
                    let layer = axis_coord - (1 >> current_material.min(1));
                    let edges_layer =
                        self.bitfields.parity.masks[axis_v.as_usize()].get_layer(layer);

                    let bottom_edge = edges_layer[coords.u as usize];
                    let mut strip_weaver =
                        StripWeaver::from_bottom_edge(bottom_edge, coords.u, coords.v);
                    let mut row = coords.u as usize;

                    while strip_weaver.runs_count > 0 && row < 32 {
                        row += 1;
                        let (edge, guard) = if row < 32 {
                            let mut g = 0u32;
                            for i in 0..strip_weaver.runs_count {
                                let run = strip_weaver.runs[i];
                                g |= self.bitfields.parity.gather_guard(
                                    axis_u,
                                    layer,
                                    row,
                                    run.start as usize,
                                    run.end as usize,
                                );
                            }
                            (edges_layer[row], g)
                        } else {
                            (0u32, 0u32)
                        };

                        strip_weaver.weave_strip(row, edge, guard, |t| {
                            (self.on_triangle)(Triangle::from_strip_triangle(
                                face,
                                axis_coord as u8,
                                t,
                                material.max(current_material),
                            ))
                        });
                    }

                    self.bitfields
                        .occlusion
                        .mark_done(axis, axis_coord, &strip_weaver.filled);
                }
            });
    }
}

impl<'a, M, F, Data> DenseWalker<Data> for ShapeWalker<'a, M, F>
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
