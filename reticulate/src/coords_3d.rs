use crate::{Morton3D, axis::Axis, octree::TREE_DEPTH, utils::bitmasks::take_one_every_3_bits};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coords3D {
    pub x: u8,
    pub y: u8,
    pub z: u8,
}

impl Coords3D {
    #[inline(always)]
    pub const fn from_morton(value: Morton3D) -> Self {
        let morton = value.code;
        let depth_shift = (TREE_DEPTH as u8 - value.depth().0) * 3;
        Coords3D {
            x: take_one_every_3_bits(morton << depth_shift) as u8,
            y: take_one_every_3_bits(morton << depth_shift >> 1) as u8,
            z: take_one_every_3_bits(morton << depth_shift >> 2) as u8,
        }
    }

    #[inline(always)]
    pub const fn from_morton_unscaled(value: Morton3D) -> Self {
        let morton = value.code;
        Coords3D {
            x: take_one_every_3_bits(morton) as u8,
            y: take_one_every_3_bits(morton >> 1) as u8,
            z: take_one_every_3_bits(morton >> 2) as u8,
        }
    }

    #[inline(always)]
    pub const fn from_2d_projection(axis: Axis, axis_coord: u8, uv_coords: (u8, u8)) -> Coords3D {
        match axis {
            Axis::X => Coords3D {
                x: axis_coord,
                y: uv_coords.1 as u8,
                z: uv_coords.0 as u8,
            },
            Axis::Y => Coords3D {
                x: uv_coords.0 as u8,
                y: axis_coord,
                z: uv_coords.1 as u8,
            },
            Axis::Z => Coords3D {
                x: uv_coords.1 as u8,
                y: uv_coords.0 as u8,
                z: axis_coord,
            },
        }
    }
}
