use crate::{
    Axis, DenseWalker, MaterialId, Morton3D,
    meshing::{bitfields::Bitfields, surface_layer::SurfaceLayer},
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

    fn set_transitions(&mut self, axis: Axis, morton: Morton3D, material: MaterialId) {
        let (morton_2d, layer) = morton.projected_axis_morton(axis);
        self.bitfields.material.iterate_materials(
            axis,
            morton_2d,
            |existing_material, coords, area| {
                if existing_material != material {
                    self.bitfields.parity.set_transition(
                        SurfaceLayer::new(axis, layer),
                        coords,
                        area,
                    );
                }
            },
        );

        self.bitfields.material.fill(axis, morton, material);
    }
}

impl<'a, M, Data> DenseWalker<Data> for GatheringWalker<'a, M>
where
    M: Fn(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _morton: Morton3D, _children_mask: u8) {}

    fn on_branch_end(&mut self, _morton: Morton3D) {}

    fn on_leaf(&mut self, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);
        self.set_transitions(Axis::X, morton, material);
        self.set_transitions(Axis::Y, morton, material);
        self.set_transitions(Axis::Z, morton, material);
    }

    fn on_empty(&mut self, morton: Morton3D) {
        self.set_transitions(Axis::X, morton, 0);
        self.set_transitions(Axis::Y, morton, 0);
        self.set_transitions(Axis::Z, morton, 0);
    }
}
