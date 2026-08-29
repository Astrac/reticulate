use crate::{Morton3D, depth::Depth, octree::TREE_DEPTH, utils::bitmasks::take_one_every_3_bits};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coords3D {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl Coords3D {
    #[inline(always)]
    pub const fn from_morton(value: Morton3D, depth: Depth) -> Self {
        let morton = value.as_u32();
        let depth_shift = (TREE_DEPTH as u8 - depth.0) * 3;
        Coords3D {
            x: take_one_every_3_bits(morton << depth_shift) as u8,
            y: take_one_every_3_bits(morton << depth_shift >> 1) as u8,
            z: take_one_every_3_bits(morton << depth_shift >> 2) as u8,
        }
    }

    #[inline(always)]
    pub const fn from_morton_unscaled(value: Morton3D) -> Self {
        let morton = value.as_u32();
        Coords3D {
            x: take_one_every_3_bits(morton) as u8,
            y: take_one_every_3_bits(morton >> 1) as u8,
            z: take_one_every_3_bits(morton >> 2) as u8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coords2D {
    pub v: u32,
    pub u: u32,
}
