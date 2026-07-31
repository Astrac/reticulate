use std::fmt::Debug;

use crate::octree::Fixed8OctreeNode;
use crate::{Coords, Morton, Fixed8Octree};

pub const FACE_RIGHT: u8 = 0b00000001; // +X
pub const FACE_LEFT: u8 = 0b00000010; // -X
pub const FACE_TOP: u8 = 0b00000100; // +Y
pub const FACE_BOTTOM: u8 = 0b00001000; // -Y
pub const FACE_FRONT: u8 = 0b00010000; // +Z
pub const FACE_BACK: u8 = 0b00100000; // -Z

#[derive(Clone, Copy)]
struct OctreeBlockFaces {
    depth: u8,
    morton: Morton,
    face_mask: u8,
    node_array_index: usize,
}

impl OctreeBlockFaces {
    pub fn finalize(self) -> BlockFaces {
        BlockFaces {
            depth: self.depth,
            coords: self.morton.into(),
            face_mask: self.face_mask,
        }
    }
}

impl Debug for OctreeBlockFaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OctreeBlockFaces")
            .field("depth", &self.depth)
            .field("morton", &format!("{:#06o}", self.morton.as_u32()))
            .field("face_mask", &format!("{:#06b}", self.face_mask))
            .field("node_array_index", &self.node_array_index)
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct BlockFaces {
    pub depth: u8,
    pub coords: Coords,
    pub face_mask: u8,
}

impl Debug for BlockFaces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockFaces")
            .field("depth", &self.depth)
            .field("coords", &self.coords)
            .field("face_mask", &format!("{:#06b}", &self.face_mask))
            .finish()
    }
}

impl BlockFaces {
    pub fn from_octree(octree: &Fixed8Octree<()>, root_node_index: usize) -> Vec<BlockFaces> {
        let octree_nodes = octree.nodes();
        let mut traversal_stack: Vec<OctreeBlockFaces> = Vec::new();
        let mut pending_stack: Vec<OctreeBlockFaces> = Vec::new();
        let mut finalized_blocks: Vec<BlockFaces> = Vec::new();

        traversal_stack.push(OctreeBlockFaces {
            depth: 0,
            morton: 0.into(),
            face_mask: 0b00111111,
            node_array_index: root_node_index,
        });

        while !traversal_stack.is_empty() || !pending_stack.is_empty() {
            // --- 1. PROCESS THE TRAVERSAL STACK ---
            if let Some(mut current_block) = traversal_stack.pop() {
                log::trace!("Traversing: {current_block:?}");

                // Check cross-boundaries against anything already waiting in the pending buffer
                for pending in pending_stack.iter_mut() {
                    if let TouchingFace::Match(face_a, _) =
                        check_neighborhood(&current_block, pending)
                    {
                        if is_block_solid(pending, octree_nodes, root_node_index) {
                            // Cull touching faces using high-performance bitwise AND NOT
                            current_block.face_mask &= !face_a;
                            log::trace!("Culled current: {current_block:?}");
                        }
                    }
                }

                // Read the linear-octree array directly using the Morton code path
                if requires_deeper_detail(&current_block, octree_nodes) {
                    // Generate all 8 children with inherited outer masks and active inner faces
                    let children = generate_children(&current_block, octree_nodes);
                    log::trace!(
                        "Need more detail: {current_block:?}; generated children: {children:?}"
                    );
                    for child_block in children {
                        pending_stack.push(child_block);
                    }
                } else {
                    // If it doesn't need to split, check if it's visible (Full) or empty space
                    let is_solid = is_block_solid(&current_block, octree_nodes, root_node_index);
                    log::trace!(
                        "No details needed: {current_block:?}; is_solid: {is_solid}, face_mask: {:#06b}, coords: {:?}",
                        current_block.face_mask,
                        Coords::from(current_block.morton)
                    );
                    if is_solid {
                        if current_block.face_mask != 0 {
                            finalized_blocks.push(current_block.finalize());
                        }
                    }
                }
            }

            if let Some(next_block) = pending_stack.pop() {
                traversal_stack.push(next_block);
            }
        }

        finalized_blocks
    }
}

fn requires_deeper_detail(block: &OctreeBlockFaces, octree_nodes: &[Fixed8OctreeNode<()>]) -> bool {
    matches!(
        &octree_nodes[block.node_array_index],
        Fixed8OctreeNode::Branch { .. }
    )
}

// Verifies if a terminal leaf block is filled geometry or air space
fn is_block_solid(
    block: &OctreeBlockFaces,
    octree_nodes: &[Fixed8OctreeNode<()>],
    root_index: usize,
) -> bool {
    let mut current_idx = root_index;

    for step in 0..block.depth {
        let shift = (block.depth - 1 - step) * 3;
        let child_octant = ((block.morton.as_u32() >> shift) & 0b111) as usize;

        match &octree_nodes[current_idx] {
            Fixed8OctreeNode::Branch { children_index } => {
                current_idx = (*children_index as usize) + child_octant;
            }
            Fixed8OctreeNode::Leaf { data } => return data.is_some(),
        }
    }

    matches!(
        &octree_nodes[current_idx],
        Fixed8OctreeNode::Leaf { data: Some(_) }
    )
}

fn generate_children(
    parent: &OctreeBlockFaces,
    octree_nodes: &[Fixed8OctreeNode<()>],
) -> [OctreeBlockFaces; 8] {
    let next_depth = parent.depth + 1;
    let base_morton = parent.morton.as_u32() << 3;

    // Fetch the pointer where the 8 sibling chunks live in the array
    let children_start = match &octree_nodes[parent.node_array_index] {
        Fixed8OctreeNode::Branch { children_index } => *children_index as usize,
        _ => unreachable!("generate_children was called on a terminal leaf node"),
    };

    // Initialize our array container for the 8 children
    let mut children = [OctreeBlockFaces {
        depth: next_depth,
        morton: 0.into(),
        face_mask: 0b00111111,
        node_array_index: 0,
    }; 8];

    for octant in 0..8 {
        let mut mask = 0b00111111; // Start completely unoccluded

        // 1. AXIS EXTRACTOR (Bit 0 = X, Bit 1 = Y, Bit 2 = Z)
        let is_right = (octant & 1) != 0; // true if +X, false if -X
        let is_top = (octant & 2) != 0; // true if +Y, false if -Y
        let is_front = (octant & 4) != 0; // true if +Z, false if -Z

        // 2. INTERNAL SIBLING CULLING (Only cull faces if the true neighbor is SOLID)
        // Check X Sibling (Flip bit 0)
        let sibling_x = octant ^ 1;
        if matches!(
            &octree_nodes[children_start + sibling_x as usize],
            Fixed8OctreeNode::Leaf { data: Some(..) }
        ) {
            mask &= if is_right { !FACE_LEFT } else { !FACE_RIGHT };
        }

        // Check Y Sibling (Flip bit 1)
        let sibling_y = octant ^ 2;
        if matches!(
            &octree_nodes[children_start + sibling_y as usize],
            Fixed8OctreeNode::Leaf { data: Some(..) }
        ) {
            mask &= if is_top { !FACE_BOTTOM } else { !FACE_TOP };
        }

        // Check Z Sibling (Flip bit 2)
        let sibling_z = octant ^ 4;
        if matches!(
            &octree_nodes[children_start + sibling_z as usize],
            Fixed8OctreeNode::Leaf { data: Some(..) }
        ) {
            mask &= if is_front { !FACE_BACK } else { !FACE_FRONT };
        }

        // 3. EXTERIOR PARENT BOUNDARY INHERITANCE
        // A child only drops a face if it sits on that outer border AND the parent face was culled
        if !is_right && (parent.face_mask & FACE_LEFT) == 0 {
            mask &= !FACE_LEFT;
        }
        if is_right && (parent.face_mask & FACE_RIGHT) == 0 {
            mask &= !FACE_RIGHT;
        }
        if !is_top && (parent.face_mask & FACE_BOTTOM) == 0 {
            mask &= !FACE_BOTTOM;
        }
        if is_top && (parent.face_mask & FACE_TOP) == 0 {
            mask &= !FACE_TOP;
        }
        if !is_front && (parent.face_mask & FACE_BACK) == 0 {
            mask &= !FACE_BACK;
        }
        if is_front && (parent.face_mask & FACE_FRONT) == 0 {
            mask &= !FACE_FRONT;
        }

        // 4. EMIT ASSIGNMENT
        children[octant as usize] = OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | octant).into(),
            face_mask: mask,
            node_array_index: children_start + octant as usize,
        };
    }

    children
}

#[allow(unused)]
enum TouchingFace {
    Match(u8, u8), // (Face mask flag for block A, Face mask flag for block B)
    None,
}

fn check_neighborhood(a: &OctreeBlockFaces, b: &OctreeBlockFaces) -> TouchingFace {
    if a.depth != b.depth {
        return TouchingFace::None;
    }

    let Coords {
        x: ax,
        y: ay,
        z: az,
    } = a.morton.into();
    let Coords {
        x: bx,
        y: by,
        z: bz,
    } = b.morton.into();

    // X-axis adjacency (Y and Z match exactly)
    if ay == by && az == bz {
        if ax + 1 == bx {
            return TouchingFace::Match(FACE_RIGHT, FACE_LEFT);
        }
        if bx + 1 == ax {
            return TouchingFace::Match(FACE_LEFT, FACE_RIGHT);
        }
    }

    // Y-axis adjacency (X and Z match exactly)
    if ax == bx && az == bz {
        if ay + 1 == by {
            return TouchingFace::Match(FACE_TOP, FACE_BOTTOM);
        }
        if by + 1 == ay {
            return TouchingFace::Match(FACE_BOTTOM, FACE_TOP);
        }
    }

    // Z-axis adjacency (X and Y match exactly)
    if ax == bx && ay == by {
        if az + 1 == bz {
            return TouchingFace::Match(FACE_FRONT, FACE_BACK);
        }
        if bz + 1 == az {
            return TouchingFace::Match(FACE_BACK, FACE_FRONT);
        }
    }

    TouchingFace::None
}
