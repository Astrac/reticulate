use crate::{ChunkCoord, Coords2D, PLANE_AREA, morton_2d::decode_morton_portable};

const fn build_coords() -> [Coords2D; PLANE_AREA] {
    let mut table = [Coords2D { u: 0, v: 0 }; PLANE_AREA];
    let mut code = 0;
    while code < PLANE_AREA {
        let (u, v) = decode_morton_portable(code as u32);
        table[code] = Coords2D {
            u: u as ChunkCoord,
            v: v as ChunkCoord,
        };
        code += 1;
    }
    table
}

pub static COORDS_LUT: [Coords2D; PLANE_AREA] = build_coords();
