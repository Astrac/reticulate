use crate::octree::OctreeNode;
use crate::{Coords, Morton, Octree};

pub const FACE_LEFT: u8 = 0b00000001; // -X
pub const FACE_RIGHT: u8 = 0b00000010; // +X
pub const FACE_BOTTOM: u8 = 0b00000100; // -Y
pub const FACE_TOP: u8 = 0b00001000; // +Y
pub const FACE_BACK: u8 = 0b00010000; // -Z
pub const FACE_FRONT: u8 = 0b00100000; // +Z

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub struct BlockFaces {
    pub depth: u8,
    pub coords: Coords,
    pub face_mask: u8,
}

impl BlockFaces {
    pub fn from_octree(octree: &Octree<()>, root_node_index: usize) -> Vec<BlockFaces> {
        let octree_nodes = octree.nodes();
        // TraversalStack: Holds blocks cleared or currently being evaluated for splits
        let mut traversal_stack: Vec<OctreeBlockFaces> = Vec::new();

        // PendingStack: Tracks active siblings and neighbors waiting for
        // mutual face-culling checks before being committed to vertices.
        let mut pending_stack: Vec<OctreeBlockFaces> = Vec::new();

        // Established blocks
        let mut finalized_blocks: Vec<BlockFaces> = Vec::new();

        // Seed the traversal with the root block. All 6 exterior faces active.
        traversal_stack.push(OctreeBlockFaces {
            depth: 0,
            morton: 0.into(),
            face_mask: 0b00111111,
            node_array_index: 0,
        });

        while !traversal_stack.is_empty() || !pending_stack.is_empty() {
            // --- 1. PROCESS THE TRAVERSAL STACK ---
            if let Some(mut current_block) = traversal_stack.pop() {
                // Check cross-boundaries against anything already waiting in the pending buffer
                for pending in pending_stack.iter_mut() {
                    if let TouchingFace::Match(face_a, face_b) =
                        check_neighborhood(&current_block, pending)
                    {
                        // Cull touching faces using high-performance bitwise AND NOT
                        current_block.face_mask &= !face_a;
                        pending.face_mask &= !face_b;
                    }
                }

                // Read the linear-octree array directly using the Morton code path
                if requires_deeper_detail(&current_block, octree_nodes) {
                    // Generate all 8 children with inherited outer masks and active inner faces
                    let children = generate_children(&current_block, octree_nodes);
                    for child_block in children {
                        pending_stack.push(child_block);
                    }
                } else {
                    // If it doesn't need to split, check if it's visible (Full) or empty space
                    if is_block_solid(&current_block, octree_nodes, root_node_index) {
                        // Only emit vertices if faces remain exposed after neighbor evaluations
                        if current_block.face_mask != 0 {
                            finalized_blocks.push(current_block.finalize());
                        }
                    }
                }
            }

            // --- 2. CYCLE PENDING ELEMENTS ---
            // Move processed items from the pending lookup buffer back into the primary traversal
            if let Some(ready_block) = pending_stack.pop() {
                traversal_stack.push(ready_block);
            }
        }

        finalized_blocks
    }
}

fn requires_deeper_detail(block: &OctreeBlockFaces, octree_nodes: &[OctreeNode<()>]) -> bool {
    matches!(
        &octree_nodes[block.node_array_index],
        OctreeNode::Branch { .. }
    )
}

// Verifies if a terminal leaf block is filled geometry or air space
fn is_block_solid(
    block: &OctreeBlockFaces,
    octree_nodes: &[OctreeNode<()>],
    root_index: usize,
) -> bool {
    let mut current_idx = root_index;

    for step in 0..block.depth {
        let shift = (block.depth - 1 - step) * 3;
        let child_octant = ((block.morton.as_u32() >> shift) & 0b111) as usize;

        match &octree_nodes[current_idx] {
            OctreeNode::Branch { children_index } => {
                current_idx = (*children_index as usize) + child_octant;
            }
            OctreeNode::Leaf { data } => return data.is_some(),
        }
    }

    matches!(
        &octree_nodes[current_idx],
        OctreeNode::Leaf { data: Some(_) }
    )
}

fn generate_children(
    parent: &OctreeBlockFaces,
    octree_nodes: &[OctreeNode<()>],
) -> [OctreeBlockFaces; 8] {
    let next_depth = parent.depth + 1;
    let base_morton = parent.morton.as_u32() << 3;

    // Get the base block address of the 8 children in the array
    let children_start = match &octree_nodes[parent.node_array_index] {
        OctreeNode::Branch { children_index } => *children_index as usize,
        _ => panic!("Unexpected: generate_children called on a Leaf"),
    };

    let make_mask = |octant: u8| -> u8 {
        let mut mask = 0b00111111; // All 6 faces open initially

        // Flips the X bit to find the adjacent internal sibling
        let sibling_x_octant = octant ^ 1; // Flips the X bit
        if matches!(
            &octree_nodes[children_start + sibling_x_octant as usize],
            OctreeNode::Leaf { .. }
        ) {
            // If my immediate internal sibling is solid, the face pointing to it is occluded
            mask = mask
                & (if (octant & 1) == 0 {
                    !FACE_RIGHT
                } else {
                    !FACE_LEFT
                });
        }
        // Inherit parent's outer X constraint if on the exterior boundary
        if (octant & 1) == 0 {
            if (parent.face_mask & FACE_LEFT) == 0 {
                mask &= !FACE_LEFT;
            }
        } else {
            if (parent.face_mask & FACE_RIGHT) == 0 {
                mask &= !FACE_RIGHT;
            }
        }

        // Y-Axis Internal Sibling Occlusion
        let sibling_y_octant = octant ^ 2; // Flips the Y bit
        if matches!(
            &octree_nodes[children_start + sibling_y_octant as usize],
            OctreeNode::Leaf { .. }
        ) {
            mask &= if (octant & 2) == 0 {
                !FACE_TOP
            } else {
                !FACE_BOTTOM
            };
        }
        if (octant & 2) == 0 {
            if (parent.face_mask & FACE_BOTTOM) == 0 {
                mask &= !FACE_BOTTOM;
            }
        } else {
            if (parent.face_mask & FACE_TOP) == 0 {
                mask &= !FACE_TOP;
            }
        }

        // Z-Axis Internal Sibling Occlusion
        let sibling_z_octant = octant ^ 4; // Flips the Z bit
        if matches!(
            &octree_nodes[children_start + sibling_z_octant as usize],
            OctreeNode::Leaf { .. }
        ) {
            mask &= if (octant & 4) == 0 {
                !FACE_FRONT
            } else {
                !FACE_BACK
            };
        }
        if (octant & 4) == 0 {
            if (parent.face_mask & FACE_BACK) == 0 {
                mask &= !FACE_BACK;
            }
        } else {
            if (parent.face_mask & FACE_FRONT) == 0 {
                mask &= !FACE_FRONT;
            }
        }

        mask
    };

    [
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 0).into(),
            face_mask: make_mask(0),
            node_array_index: children_start + 0,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 1).into(),
            face_mask: make_mask(1),
            node_array_index: children_start + 1,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 2).into(),
            face_mask: make_mask(2),
            node_array_index: children_start + 2,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 3).into(),
            face_mask: make_mask(3),
            node_array_index: children_start + 3,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 4).into(),
            face_mask: make_mask(4),
            node_array_index: children_start + 4,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 5).into(),
            face_mask: make_mask(5),
            node_array_index: children_start + 5,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 6).into(),
            face_mask: make_mask(6),
            node_array_index: children_start + 6,
        },
        OctreeBlockFaces {
            depth: next_depth,
            morton: (base_morton | 7).into(),
            face_mask: make_mask(7),
            node_array_index: children_start + 7,
        },
    ]
}

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

// // Array tracking the 4 corner layout offsets of a standard unit cube face.
// /// Ordered counter-clockwise to match Bevy's backface culling engine.
// static FACE_CORNER_OFFSETS: [[[f32; 3]; 4]; 6] = [
//     // 0: Front Face (+Z)
//     [
//         [0.0, 0.0, 1.0],
//         [1.0, 0.0, 1.0],
//         [1.0, 1.0, 1.0],
//         [0.0, 1.0, 1.0],
//     ],
//     // 1: Back Face (-Z)
//     [
//         [1.0, 0.0, 0.0],
//         [0.0, 0.0, 0.0],
//         [0.0, 1.0, 0.0],
//         [1.0, 1.0, 0.0],
//     ],
//     // 2: Left Face (-X)
//     [
//         [0.0, 0.0, 0.0],
//         [0.0, 0.0, 1.0],
//         [0.0, 1.0, 1.0],
//         [0.0, 1.0, 0.0],
//     ],
//     // 3: Right Face (+X)
//     [
//         [1.0, 0.0, 1.0],
//         [1.0, 0.0, 0.0],
//         [1.0, 1.0, 0.0],
//         [1.0, 1.0, 1.0],
//     ],
//     // 4: Top Face (+Y)
//     [
//         [0.0, 1.0, 1.0],
//         [1.0, 1.0, 1.0],
//         [1.0, 1.0, 0.0],
//         [0.0, 1.0, 0.0],
//     ],
//     // 5: Bottom Face (-Y)
//     [
//         [0.0, 0.0, 0.0],
//         [1.0, 0.0, 0.0],
//         [1.0, 0.0, 1.0],
//         [0.0, 0.0, 1.0],
//     ],
// ];
//
// /// Matching face directional unit vectors.
// static FACE_NORMALS: [[f32; 3]; 6] = [
//     [0.0, 0.0, 1.0],  // Front
//     [0.0, 0.0, -1.0], // Back
//     [-1.0, 0.0, 0.0], // Left
//     [1.0, 0.0, 0.0],  // Right
//     [0.0, 1.0, 0.0],  // Top
//     [0.0, -1.0, 0.0], // Bottom
// ];
