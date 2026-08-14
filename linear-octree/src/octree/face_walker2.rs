use crate::{Morton3D, Walker, face::Face, octree::meshing_planes::MeshingPlanes};

// The coalescing represents the property we are trying to coalesce for at that particular
// level. We can coalesce a level only if all voxel share the same property, otherwise
// we emit the faces at that point.
#[allow(dead_code)]
pub struct FaceWalker<F, Data> {
    on_face: F,
    meshing_planes: MeshingPlanes,
    coalescing_stack: [CoalesceState<Data>; 6],
}

#[derive(Clone, Copy, PartialEq)]
enum CoalesceState<Data> {
    Empty,
    Target(Data),
    Heterogeneous,
}

impl<F, Data> FaceWalker<F, Data>
where
    Data: Copy + PartialEq,
{
    pub fn new(f: F) -> FaceWalker<F, Data> {
        FaceWalker {
            on_face: f,
            meshing_planes: MeshingPlanes::default(),
            coalescing_stack: [CoalesceState::Empty; 6],
        }
    }
}

impl<Data, F> Walker<Data> for FaceWalker<F, Data>
where
    F: FnMut(Morton3D, u8, Face),
{
    fn on_branch_start(&mut self, depth: u8, morton: Morton3D, children_mask: u8) {
    }

    fn on_branch_end(&mut self, depth: u8, morton: Morton3D) {
        todo!()
    }

    fn on_leaf(&mut self, depth: u8, morton: Morton3D, data: &Data) {
        let _ = !self.meshing_planes.out_pos_x_mut(morton, depth);
    }
}
