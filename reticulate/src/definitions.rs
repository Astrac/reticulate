pub type MaterialId = u16;

pub const CHUNK_VOLUME: usize = PLANE_SIDE * PLANE_SIDE * PLANE_SIDE;
pub const PLANE_AREA: usize = PLANE_SIDE * PLANE_SIDE;
pub const PLANE_SIDE: usize = 1 << TREE_DEPTH;
pub const TREE_DEPTH: usize = 5;
