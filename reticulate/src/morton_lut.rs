use crate::{Axis, ChunkCoord, Depth, Morton3D, TREE_DEPTH, utils::bitmasks};

// Split the 15 bit Morton_3d code in two tables.
const LOWER_BITS: usize = 1 << 8;
const HIGHER_BITS: usize = 1 << 7;

const HIGHER_BITS_MASK: u16 = 0x3FF;

const LAYER_SHIFT: u32 = 10;

pub struct MortonLut {
    lower_bits: [[u16; LOWER_BITS]; 3],
    higher_bits: [[u16; HIGHER_BITS]; 3],
}

impl MortonLut {
    pub const fn project(&self, morton: &Morton3D, axis: Axis) -> (u32, ChunkCoord) {
        let raw = morton.code << ((TREE_DEPTH - morton.depth.as_usize()) * 3);
        let a = axis.as_usize();
        let packed = self.lower_bits[a][(raw & 0xFF) as usize]
            | self.higher_bits[a][((raw >> 8) as usize) & (HIGHER_BITS - 1)];

        (
            (packed & HIGHER_BITS_MASK) as u32,
            (packed >> LAYER_SHIFT) as ChunkCoord,
        )
    }
}

const fn project_entry(axis: Axis, raw: u32) -> u16 {
    let ((u, v), layer) = Morton3D::from_code(raw, Depth::MAX).projected_axis_uv(axis);
    let code = bitmasks::interleave_zeroes(u as u32) | (bitmasks::interleave_zeroes(v as u32) << 1);
    (code | ((layer as u32) << LAYER_SHIFT)) as u16
}

const fn build_projection() -> MortonLut {
    let mut tables = MortonLut {
        lower_bits: [[0; 256]; 3],
        higher_bits: [[0; HIGHER_BITS]; 3],
    };

    let mut axis = 0;

    while axis < 3 {
        let mut index = 0;

        while index < 256 {
            tables.lower_bits[axis][index] = project_entry(Axis::AXES[axis], index as u32);
            index += 1;
        }

        index = 0;
        while index < HIGHER_BITS {
            tables.higher_bits[axis][index] = project_entry(Axis::AXES[axis], (index as u32) << 8);
            index += 1;
        }

        axis += 1;
    }
    tables
}

pub static MORTON_LUT: MortonLut = build_projection();
