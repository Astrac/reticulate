use crate::{
    DenseWalker, Depth, MeshFace, Morton3D, axis::*,
    meshing::{decision_boundaries::DecisionBoundaries, harvest::harvest_faces},
};

pub struct SimpleFaceWalker<F> {
    boundaries: DecisionBoundaries,
    on_face: F,
}

impl<F> SimpleFaceWalker<F>
where
    F: FnMut(MeshFace),
{
    pub fn new(f: F) -> SimpleFaceWalker<F> {
        SimpleFaceWalker {
            on_face: f,
            boundaries: Default::default(),
        }
    }

    fn walk_on_solid(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        let deciding = self.boundaries.flip_deciding_face(axis, morton, depth);

        harvest_faces(deciding, depth, &mut |local_face_morton, face_size| {
            let mesh_face = MeshFace::from_morton_back_projection(
                axis.as_negative_face(),
                morton,
                Depth(depth),
                local_face_morton,
                face_size,
            );

            (self.on_face)(mesh_face);
        });

        self.boundaries.fill_deciding_face(axis, morton, depth);
    }

    fn walk_on_void(&mut self, axis: Axis, morton: Morton3D, depth: u8) {
        let deciding = self.boundaries.deciding_face_mut(axis, morton, depth);

        harvest_faces(deciding, depth, &mut |local_face_morton, face_size| {
            let mesh_face = MeshFace::from_morton_back_projection(
                axis.as_positive_face(),
                morton,
                Depth(depth),
                local_face_morton,
                face_size,
            );

            (self.on_face)(mesh_face);
        });

        deciding.fill(false);
    }
}

impl<Data: PartialEq + Copy, F> DenseWalker<Data> for SimpleFaceWalker<F>
where
    F: FnMut(MeshFace),
{
    fn on_branch_start(&mut self, _depth: u8, _morton: Morton3D, _children_mask: u8) {}

    fn on_branch_end(&mut self, _depth: u8, _morton: Morton3D) {
        // TODO: Harvest outermost+ faces
    }

    fn on_leaf(&mut self, depth: u8, morton: Morton3D, _data: &Data) {
        self.walk_on_solid(AXIS_X, morton, depth);
        self.walk_on_solid(AXIS_Y, morton, depth);
        self.walk_on_solid(AXIS_Z, morton, depth);
    }

    fn on_empty(&mut self, depth: u8, morton: Morton3D) {
        self.walk_on_void(AXIS_X, morton, depth);
        self.walk_on_void(AXIS_Y, morton, depth);
        self.walk_on_void(AXIS_Z, morton, depth);
    }
}
