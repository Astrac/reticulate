use crate::{
    Axis, DenseWalker, MaterialId, Morton3D,
    meshing::{
        bitfields::{Material, Parity},
        surface_layer::SurfaceLayer,
    },
};

pub struct GatheringWalker<'a, 'b, M> {
    material: &'b mut Material<'a>,
    parity: &'b mut Parity<'a>,
    to_material: &'b mut M,
}

impl<'a, 'b, M> GatheringWalker<'a, 'b, M> {
    pub fn new(
        material: &'b mut Material<'a>,
        parity: &'b mut Parity<'a>,
        to_material: &'b mut M,
    ) -> GatheringWalker<'a, 'b, M> {
        GatheringWalker {
            material,
            parity,
            to_material,
        }
    }

    fn set_transitions(&mut self, axis: Axis, morton: Morton3D, material: MaterialId) {
        let (morton_2d, layer) = morton.projected_axis_morton(axis);
        self.material
            .iterate_materials(axis, morton_2d, |existing_material, coords, area| {
                if existing_material != material {
                    self.parity
                        .set_transition(SurfaceLayer::new(axis, layer), coords, area);
                }
            });

        self.material.fill(axis, morton, material);
    }
}

impl<'a, 'b, M, Data> DenseWalker<Data> for GatheringWalker<'a, 'b, M>
where
    M: FnMut(&Data) -> MaterialId,
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
