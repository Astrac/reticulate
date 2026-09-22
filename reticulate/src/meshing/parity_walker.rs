use crate::{
    DenseWalker, Depth, Morton3D,
    axis::*,
    meshing::bitfields::{Bitfields, MaterialId},
};

pub struct ParityWalker<'a, M> {
    bitfields: Bitfields<'a>,
    to_material: M,
}

impl<'a, M> ParityWalker<'a, M> {
    pub fn new(bitfields: Bitfields<'a>, to_material: M) -> ParityWalker<'a, M> {
        ParityWalker {
            bitfields,
            to_material,
        }
    }

    // TODO - Optimization opportunities:
    //   * Use occlusion mask to skip fully occluded areas
    //   * Same thing could be done XORing the materials
    //     - How to actually get materials in the area?
    //     - Remember: XORing twice the same value is a noop
    fn register_material_at(
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

impl<'a, M, Data> DenseWalker<Data> for ParityWalker<'a, M>
where
    M: Fn(&Data) -> MaterialId,
{
    fn on_branch_start(&mut self, _depth: Depth, _morton: Morton3D, _children_mask: u8) {}

    fn on_branch_end(&mut self, _depth: Depth, _morton: Morton3D) {}

    fn on_leaf(&mut self, depth: Depth, morton: Morton3D, data: &Data) {
        let material = (self.to_material)(data);
        self.register_material_at(AXIS_X, morton, depth, material);
        self.register_material_at(AXIS_Y, morton, depth, material);
        self.register_material_at(AXIS_Z, morton, depth, material);
    }

    fn on_empty(&mut self, depth: Depth, morton: Morton3D) {
        self.register_material_at(AXIS_X, morton, depth, 0);
        self.register_material_at(AXIS_Y, morton, depth, 0);
        self.register_material_at(AXIS_Z, morton, depth, 0);
    }
}
