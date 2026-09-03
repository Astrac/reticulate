use crate::{
    Morton2D, Morton3D, axis::*, depth::Depth, octree::TREE_DEPTH,
    utils::bitmasks::take_one_every_3_bits,
};

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

    #[inline(always)]
    pub const fn from_morton_back_projection(
        axis: Axis,
        morton_3d: Morton3D,
        depth_3d: Depth,
        morton_2d: Morton2D,
    ) -> Self {
        let coords_2d = morton_2d.as_coords();
        let mut coords = Coords3D::from_morton(morton_3d, depth_3d);
        let (dx, dy, dz) = match axis {
            AXIS_X => (0, coords_2d.u, coords_2d.v),
            AXIS_Y => (coords_2d.u, 0, coords_2d.v),
            AXIS_Z => (coords_2d.u, coords_2d.v, 0),
            _ => unreachable!(),
        };

        coords.x = coords.x.saturating_add(dx as u8);
        coords.y = coords.y.saturating_add(dy as u8);
        coords.z = coords.z.saturating_add(dz as u8);

        coords
    }
}

// TODO - Fix types here
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Coords2D {
    pub v: u32,
    pub u: u32,
}
