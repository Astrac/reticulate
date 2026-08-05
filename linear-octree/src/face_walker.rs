use crate::{BlockFaces, Morton, Walker};

pub struct FaceWalker {
    faces: Vec<BlockFaces>,
    // (original_branch_mask, face_mask_inherited_from_parent)
    pending_branches: Vec<(u8, u8)>,
    original_branch_mask: u8,
    current_inherited_faces: u8,
}

impl FaceWalker {
    pub fn new() -> FaceWalker {
        FaceWalker {
            faces: Vec::new(),
            pending_branches: Vec::new(),
            original_branch_mask: 1,
            current_inherited_faces: 0b00111111,
        }
    }

    pub fn faces(&self) -> &[BlockFaces] {
        &self.faces
    }
}

impl<_Data> Walker<_Data> for FaceWalker {
    fn on_branch_start(&mut self, _depth: u8, octant: u8, _morton: Morton, mask: u8) {
        self.pending_branches
            .push((self.original_branch_mask, self.current_inherited_faces));

        self.current_inherited_faces = culled_node_faces(
            octant,
            self.current_inherited_faces,
            self.original_branch_mask,
        );
        self.original_branch_mask = mask;
    }

    fn on_branch_end(&mut self, _depth: u8, _octant: u8, _morton: Morton) {
        if let Some((b_orig, faces_inh)) = self.pending_branches.pop() {
            self.original_branch_mask = b_orig;
            self.current_inherited_faces = faces_inh;
        } // TODO Else errors?
    }

    fn on_leaf(&mut self, depth: u8, octant: u8, morton: Morton, _data: &_Data) {
        let node_face_mask = culled_node_faces(
            octant,
            self.current_inherited_faces,
            self.original_branch_mask,
        );

        if node_face_mask != 0 {
            self.faces.push(BlockFaces {
                depth,
                coords: morton.into(),
                face_mask: node_face_mask,
            });
        }
    }
}

fn culled_node_faces(octant: u8, current_inherited_faces: u8, original_branch_mask: u8) -> u8 {
    // Calculate the culling for this specific octant based on its siblings in the current branch
    let mut node_face_mask = current_inherited_faces;

    // X-Axis Culling (Bit 0 of octant)
    if (octant & 1) == 0 {
        if (original_branch_mask & (1 << (octant + 1))) != 0 {
            node_face_mask &= !(1 << 0); // Cull -X face
        }
    } else {
        if (original_branch_mask & (1 << (octant - 1))) != 0 {
            node_face_mask &= !(1 << 1); // Cull +X face
        }
    }

    // Y-Axis Culling (Bit 1 of octant)
    if (octant & 2) == 0 {
        if (original_branch_mask & (1 << (octant + 2))) != 0 {
            node_face_mask &= !(1 << 2); // Cull -Y face
        }
    } else {
        if (original_branch_mask & (1 << (octant - 2))) != 0 {
            node_face_mask &= !(1 << 3); // Cull +Y face
        }
    }

    // Z-Axis Culling (Bit 2 of octant)
    if (octant & 4) == 0 {
        if (original_branch_mask & (1 << (octant + 4))) != 0 {
            node_face_mask &= !(1 << 4); // Cull -Z face
        }
    } else {
        if (original_branch_mask & (1 << (octant - 4))) != 0 {
            node_face_mask &= !(1 << 5); // Cull +Z face
        }
    }

    node_face_mask
}
