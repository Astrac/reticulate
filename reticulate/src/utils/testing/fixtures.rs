use crate::{
    Axis, CHUNK_VOLUME, ChunkCoord, Coords2D, MaterialId, Morton3D, PLANE_SIDE,
    meshing::{
        bitfields::{Claims, ClaimsMask, PARITY_WORDS, Parity, ParityMask},
        surface_layer::SurfaceLayer,
    },
};

pub const EMPTY: MaterialId = 0;
pub const STONE: MaterialId = 1;
pub const DIRT: MaterialId = 2;

/// Three zeroed word buffers, one per axis, sized like a parity/claims mask.
pub fn storage() -> [Vec<u64>; 3] {
    words(PARITY_WORDS)
}

/// Three zeroed word buffers of `n` words each.
pub fn words(n: usize) -> [Vec<u64>; 3] {
    core::array::from_fn(|_| vec![0u64; n])
}

pub fn parity_of(s: &mut [Vec<u64>; 3]) -> Parity<'_> {
    let [x, y, z] = s;
    Parity {
        masks: [
            ParityMask::from_u64_slice(x),
            ParityMask::from_u64_slice(y),
            ParityMask::from_u64_slice(z),
        ],
    }
}

pub fn claims_of(s: &mut [Vec<u64>; 3]) -> Claims<'_> {
    let [x, y, z] = s;
    Claims {
        masks: [
            ClaimsMask::from_u64_slice(x),
            ClaimsMask::from_u64_slice(y),
            ClaimsMask::from_u64_slice(z),
        ],
    }
}

/// Builds a `filled`-shaped mask (one word per row, bits over columns) from
/// `(row, col)` cells.
pub fn cells(list: &[(usize, usize)]) -> [u32; PLANE_SIDE] {
    let mut m = [0u32; PLANE_SIDE];
    for &(row, col) in list {
        m[row] |= 1 << col;
    }
    m
}

/// A dense chunk of materials, indexed by Morton code.
pub struct VoxelGrid(pub Vec<MaterialId>);

impl VoxelGrid {
    pub fn new(voxels: &[((usize, usize, usize), MaterialId)]) -> VoxelGrid {
        let mut g = vec![EMPTY; CHUNK_VOLUME];
        for &((x, y, z), m) in voxels {
            g[Morton3D::from_xyz_max_depth(x as u8, y as u8, z as u8).as_u32() as usize] = m;
        }
        VoxelGrid(g)
    }

    /// Material at (x, y, z). Outside the chunk is empty.
    pub fn at(&self, (x, y, z): (i32, i32, i32)) -> MaterialId {
        let side = PLANE_SIDE as i32;
        if x < 0 || y < 0 || z < 0 || x >= side || y >= side || z >= side {
            return EMPTY;
        }
        self.0[Morton3D::from_xyz_max_depth(x as u8, y as u8, z as u8).as_u32() as usize]
    }
}

/// The voxel at coordinate `c` along axis `a`, at plane position (u, v),
/// using `projected_axis_uv`'s convention: X -> (y, z), Y -> (z, x), Z -> (x, y).
pub fn voxel(a: usize, c: i32, u: i32, v: i32) -> (i32, i32, i32) {
    match a {
        0 => (c, u, v),
        1 => (v, c, u),
        _ => (u, v, c),
    }
}

/// A voxel grid together with the parity and claims storage of its chunk.
///
/// Parity is built by brute force the way the gathering pass does it: a
/// transition along axis A at coordinate c wherever the material at c
/// differs from the one at c - 1, including between two different solids.
/// Outside the chunk counts as empty. Claims start out empty.
pub struct Fixture {
    pub grid: VoxelGrid,
    pub parity: [Vec<u64>; 3],
    pub claims: [Vec<u64>; 3],
}

impl Fixture {
    pub fn new(voxels: &[((usize, usize, usize), MaterialId)]) -> Fixture {
        let mut f = Fixture {
            grid: VoxelGrid::new(voxels),
            parity: storage(),
            claims: storage(),
        };
        let mut parity = parity_of(&mut f.parity);
        for a in 0..3 {
            for c in 0..PLANE_SIDE as i32 {
                for u in 0..PLANE_SIDE as i32 {
                    for v in 0..PLANE_SIDE as i32 {
                        let below = f.grid.at(voxel(a, c - 1, u, v));
                        let here = f.grid.at(voxel(a, c, u, v));
                        if below != here {
                            let uv = Coords2D {
                                u: u as ChunkCoord,
                                v: v as ChunkCoord,
                            };
                            parity.set_transition(
                                SurfaceLayer::new(Axis::AXES[a], c as ChunkCoord),
                                uv,
                                1,
                            );
                        }
                    }
                }
            }
        }
        f
    }

    /// Single-material shape: every listed voxel is stone.
    pub fn solid(solid: &[(usize, usize, usize)]) -> Fixture {
        let voxels: Vec<_> = solid.iter().map(|&p| (p, STONE)).collect();
        Fixture::new(&voxels)
    }

    /// A flat shape on the plane y = 0 given as `(row, col)` cells, where
    /// rows are z and columns are x (the Y axis' (u, v) convention).
    pub fn plane(cells_rc: &[(usize, usize)]) -> Fixture {
        let voxels: Vec<_> = cells_rc.iter().map(|&(r, c)| (c, 0, r)).collect();
        Fixture::solid(&voxels)
    }
}
