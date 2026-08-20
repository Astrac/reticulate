use crate::{
    DenseWalker, Depth, MeshFace, Morton3D,
    axis::*,
    meshing::{decision_boundaries::DecisionBoundaries, harvest::harvest_faces},
};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
enum BranchCoalescence {
    #[default]
    Undecided,
    Void,
    Solid,
    Subdivided,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
enum PendingWalk {
    Solid,
    #[default]
    Void,
}

pub struct CompatingFaceWalker<F> {
    boundaries: DecisionBoundaries,
    on_face: F,
    stack: [BranchCoalescence; 6],
    pending_walks: [[PendingWalk; 8]; 6],
}

impl<F> CompatingFaceWalker<F>
where
    F: FnMut(MeshFace),
{
    pub fn new(f: F) -> CompatingFaceWalker<F> {
        CompatingFaceWalker {
            on_face: f,
            boundaries: Default::default(),
            stack: Default::default(),
            pending_walks: Default::default(),
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

    fn emit_pending_walks(&mut self, morton: Morton3D, depth: u8) {
        let octant = morton.octant();
        for i in 0..octant {
            match self.pending_walks[depth as usize][i as usize] {
                PendingWalk::Solid => {
                    let morton = morton.parent().child_octant(i);
                    self.walk_on_solid(AXIS_X, morton, depth + 1);
                    self.walk_on_solid(AXIS_Y, morton, depth + 1);
                    self.walk_on_solid(AXIS_Z, morton, depth + 1);
                }
                PendingWalk::Void => {
                    let morton = morton.parent().child_octant(i);
                    self.walk_on_void(AXIS_X, morton, depth + 1);
                    self.walk_on_void(AXIS_Y, morton, depth + 1);
                    self.walk_on_void(AXIS_Z, morton, depth + 1);
                }
            }
        }
    }
}

impl<Data: PartialEq + Copy, F> DenseWalker<Data> for CompatingFaceWalker<F>
where
    F: FnMut(MeshFace),
{
    fn on_branch_start(&mut self, depth: u8, morton: Morton3D, _children_mask: u8) {
        self.stack[depth as usize] = BranchCoalescence::Undecided;

        if depth > 0 {
            let parent_idx = (depth - 1) as usize;
            if self.stack[parent_idx] == BranchCoalescence::Undecided {
                self.stack[parent_idx] = BranchCoalescence::Subdivided;
            }

            if self.stack[parent_idx] != BranchCoalescence::Subdivided {
                self.stack[parent_idx] = BranchCoalescence::Subdivided;
                self.emit_pending_walks(morton, parent_idx as u8);
            }
        }
    }

    fn on_branch_end(&mut self, depth: u8, morton: Morton3D) {
        match self.stack[depth as usize] {
            BranchCoalescence::Undecided => unreachable!(),
            BranchCoalescence::Void => {
                self.walk_on_void(AXIS_X, morton, depth);
                self.walk_on_void(AXIS_Y, morton, depth);
                self.walk_on_void(AXIS_Z, morton, depth);
            }
            BranchCoalescence::Solid => {
                self.walk_on_solid(AXIS_X, morton, depth);
                self.walk_on_solid(AXIS_Y, morton, depth);
                self.walk_on_solid(AXIS_Z, morton, depth);
            }
            BranchCoalescence::Subdivided => (),
        }
    }

    fn on_leaf(&mut self, depth: u8, morton: Morton3D, _data: &Data) {
        let stack_idx = (depth - 1) as usize;

        if self.stack[stack_idx] == BranchCoalescence::Undecided {
            self.stack[stack_idx] = BranchCoalescence::Solid;
        }

        if self.stack[stack_idx] == BranchCoalescence::Solid {
            self.pending_walks[stack_idx][morton.octant() as usize] = PendingWalk::Solid;
        }

        if self.stack[stack_idx] == BranchCoalescence::Void {
            self.stack[stack_idx] = BranchCoalescence::Subdivided;
            self.emit_pending_walks(morton, depth - 1);
        }

        if self.stack[stack_idx] == BranchCoalescence::Subdivided {
            self.walk_on_solid(AXIS_X, morton, depth);
            self.walk_on_solid(AXIS_Y, morton, depth);
            self.walk_on_solid(AXIS_Z, morton, depth);
        }
    }

    fn on_empty(&mut self, depth: u8, morton: Morton3D) {
        let stack_idx = (depth - 1) as usize;

        if self.stack[stack_idx] == BranchCoalescence::Undecided {
            self.stack[stack_idx] = BranchCoalescence::Void;
        }

        if self.stack[stack_idx] == BranchCoalescence::Void {
            self.pending_walks[stack_idx][morton.octant() as usize] = PendingWalk::Void;
        }

        if self.stack[stack_idx] == BranchCoalescence::Solid {
            self.stack[stack_idx] = BranchCoalescence::Subdivided;
            self.emit_pending_walks(morton, depth - 1);
        }

        if self.stack[stack_idx] == BranchCoalescence::Subdivided {
            self.walk_on_void(AXIS_X, morton, depth);
            self.walk_on_void(AXIS_Y, morton, depth);
            self.walk_on_void(AXIS_Z, morton, depth);
        }
    }
}
