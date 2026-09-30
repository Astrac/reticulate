use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    meshing::bitfields::{Bitfields, MaterialId},
};

pub struct GatheringWalker<'a, M> {
    bitfields: Bitfields<'a>,
    to_material: &'a mut M,
}

impl<'a, M> GatheringWalker<'a, M> {
    pub fn new(bitfields: Bitfields<'a>, to_material: &'a mut M) -> GatheringWalker<'a, M> {
        GatheringWalker {
            bitfields,
            to_material,
        }
    }

    fn set_transitions(
        &mut self,
        axis: Axis,
        morton: Morton3D,
        depth: Depth,
        material: MaterialId,
    ) {
        let (morton_2d, layer) = morton.projected_axis_morton(axis, depth);
        self.bitfields.material.iterate_materials(
            axis,
            morton_2d,
            depth,
            |existing_material, coords, area| {
                if existing_material != material {
                    self.bitfields
                        .parity
                        .set_transition(axis, layer, coords, area);
                }
            },
        );

        self.bitfields.material.fill(axis, morton, depth, material);
    }
}

impl<'a, M, Data> DenseWalker<Data> for GatheringWalker<'a, M>
where
    M: Fn(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}

    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);
        self.set_transitions(Axis::X, morton, depth, material);
        self.set_transitions(Axis::Y, morton, depth, material);
        self.set_transitions(Axis::Z, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        self.set_transitions(Axis::X, morton, depth, 0);
        self.set_transitions(Axis::Y, morton, depth, 0);
        self.set_transitions(Axis::Z, morton, depth, 0);
    }
}
