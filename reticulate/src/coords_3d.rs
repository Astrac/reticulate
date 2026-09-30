use crate::{
    ChunkCoord, Morton3D, axis::Axis, meshing::surface_layer::SurfaceLayer, octree::TREE_DEPTH,
    utils::bitmasks::take_one_every_3_bits,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coords3D {
    pub x: ChunkCoord,
    pub y: ChunkCoord,
    pub z: ChunkCoord,
}

impl Coords3D {
    #[inline(always)]
    pub const fn from_morton(value: Morton3D) -> Self {
        let morton = value.code;
        let depth_shift = (TREE_DEPTH as ChunkCoord - value.depth().0) * 3;
        Coords3D {
            x: take_one_every_3_bits(morton << depth_shift) as ChunkCoord,
            y: take_one_every_3_bits(morton << depth_shift >> 1) as ChunkCoord,
            z: take_one_every_3_bits(morton << depth_shift >> 2) as ChunkCoord,
        }
    }

    #[inline(always)]
    pub const fn from_morton_unscaled(value: Morton3D) -> Self {
        let morton = value.code;
        Coords3D {
            x: take_one_every_3_bits(morton) as ChunkCoord,
            y: take_one_every_3_bits(morton >> 1) as ChunkCoord,
            z: take_one_every_3_bits(morton >> 2) as ChunkCoord,
        }
    }

    #[inline(always)]
    pub const fn from_2d_projection(layer: SurfaceLayer, uv_coords: (u8, u8)) -> Coords3D {
        match layer.axis {
            Axis::X => Coords3D {
                x: layer.layer as ChunkCoord,
                y: uv_coords.1 as ChunkCoord,
                z: uv_coords.0 as ChunkCoord,
            },
            Axis::Y => Coords3D {
                x: uv_coords.0 as ChunkCoord,
                y: layer.layer as ChunkCoord,
                z: uv_coords.1 as ChunkCoord,
            },
            Axis::Z => Coords3D {
                x: uv_coords.1 as ChunkCoord,
                y: uv_coords.0 as ChunkCoord,
                z: layer.layer as ChunkCoord,
            },
        }
    }
}
