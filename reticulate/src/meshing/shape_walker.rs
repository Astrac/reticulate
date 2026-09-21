use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    meshing::{
        Triangle,
        bitfields::{Bitfields, MaterialId},
        triangle_emitter::TriangleEmitter,
    },
};

pub struct ShapeWalker<'a, M, F> {
    bitfields: Bitfields<'a>,
    triangle_emitter: TriangleEmitter,
    to_material: M,
    on_triangle: &'a F,
}

impl<'a, M, F> ShapeWalker<'a, M, F> {
    pub fn new(
        bitfields: Bitfields<'a>,
        to_material: M,
        on_triangle: &'a F,
    ) -> ShapeWalker<'a, M, F> {
        ShapeWalker {
            bitfields,
            on_triangle,
            to_material,
            triangle_emitter: TriangleEmitter::default(),
        }
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

        let (morton_2d, layer) = morton.projected_axis_morton(AXIS_Y, depth);
        let edges_layer = self.bitfields.parity.get_layer(AXIS_X, layer);
        let guards_layer = self.bitfields.parity.get_layer(AXIS_Z, layer);

        self.bitfields.material.iterate_materials(
            AXIS_X,
            morton_2d,
            depth,
            |material, coords, _| {
                if material == 0 && self.bitfields.occlusion.is_pending(AXIS_X, layer, coords) {
                    // self.bitfields
                    //     .shape_finder
                    //     .from_corner(coords.u as usize, coords.v as usize);
                    //
                    // self.triangle_emitter.process(
                    //     AXIS_X,
                    //     layer,
                    //     self.bitfields.shape_finder.found,
                    //     material,
                    //     self.on_triangle,
                    // );
                    //
                    // self.bitfields.occlusion.mark_done(
                    //     AXIS_X,
                    //     layer,
                    //     self.bitfields.shape_finder.found,
                    // );
                }
            },
        );

        self.bitfields
            .material
            .fill(AXIS_X, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        let (morton_2d, layer) = morton.projected_axis_morton(AXIS_X, depth);
        let parity_layer = self.bitfields.parity.get_layer(AXIS_X, layer);
        // self.bitfields.shape_finder.load_layer(parity_layer);

        self.bitfields.material.iterate_materials(
            AXIS_X,
            morton_2d,
            depth,
            |material, coords, _| {
                if material != 0 && self.bitfields.occlusion.is_pending(AXIS_X, layer, coords) {
                    // self.bitfields
                    //     .shape_finder
                    //     .from_corner(coords.u as usize, coords.v as usize);
                    //
                    // self.triangle_emitter.process(
                    //     AXIS_X,
                    //     layer,
                    //     self.bitfields.shape_finder.found,
                    //     material,
                    //     self.on_triangle,
                    // );
                    //
                    // self.bitfields.occlusion.mark_done(
                    //     AXIS_X,
                    //     layer,
                    //     self.bitfields.shape_finder.found,
                    // );
                }
            },
        );

        self.bitfields.material.fill(AXIS_X, morton, depth, 0);
    }
}
