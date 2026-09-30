use crate::ChunkCoord;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coords2D {
    pub v: ChunkCoord,
    pub u: ChunkCoord,
}
