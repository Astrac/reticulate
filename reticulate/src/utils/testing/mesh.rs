#[cfg(debug_assertions)]
use crate::utils::debug_assertions::debug_assert_within_footprint;
use crate::{
    Axis, CHUNK_VOLUME, ChunkCoord, Coords2D, DenseWalker, MaterialId, Morton3D, PLANE_AREA,
    PLANE_SIDE, TREE_DEPTH,
    meshing::{
        Triangle,
        bitfields::Material,
        surface_layer::SurfaceLayer,
        surface_scanner::SurfaceScanner,
        surface_walker::SurfaceWalker,
        triangulation::{Triangle2D, edges::Edges, filler::Filler, weaver::Weaver},
    },
    utils::testing::fixtures::{EMPTY, Fixture, VoxelGrid, claims_of, parity_of},
};

/// An aligned block of the chunk that is walked as a single node:
/// `((x, y, z), depth)`.
pub type Block = ((usize, usize, usize), usize);

/// Walks `grid` in Morton order as a dense tree. Every voxel is a leaf at
/// maximum depth except inside `blocks`, which are reported as single
/// larger nodes (and must therefore be uniform).
pub fn dense_walk<W: DenseWalker<MaterialId>>(walker: &mut W, grid: &VoxelGrid, blocks: &[Block]) {
    let starts: Vec<Morton3D> = blocks
        .iter()
        .map(|&((x, y, z), d)| {
            let side = PLANE_SIDE >> d;
            assert!(
                x % side == 0 && y % side == 0 && z % side == 0,
                "block must be aligned"
            );
            Morton3D::from_xyz_and_depth(x as u8, y as u8, z as u8, d.into())
        })
        .collect();

    let mut code = 0u32;
    while (code as usize) < CHUNK_VOLUME {
        let depth = starts
            .iter()
            .find(|&&s| s.as_u32() == code)
            .map_or(TREE_DEPTH, |&s| s.depth().as_usize());

        let shift = 3 * (TREE_DEPTH - depth) as u32;
        let span = 1u32 << shift;
        let material = grid.0[code as usize];

        assert!(
            (code..code + span).all(|i| grid.0[i as usize] == material),
            "block at Morton {code} is not uniform"
        );

        let morton = Morton3D::from_code(code >> shift, depth.into());
        if material == EMPTY {
            walker.on_empty(morton);
        } else {
            walker.on_leaf(morton, &material);
        }
        code += span;
    }
}

/// Meshes the chunk once with fresh material planes and the fixture's
/// (brute-force) parity and claims. Claims persist in the fixture, so a
/// second call only emits what the first one did not claim.
pub fn mesh(f: &mut Fixture, blocks: &[Block]) -> Vec<Triangle> {
    let mut tris = Vec::new();
    let mut cache: [[MaterialId; PLANE_AREA]; 3] = [[0; PLANE_AREA]; 3];
    {
        let [mx, my, mz] = &mut cache;
        let mut materials = Material {
            planes: [mx, my, mz],
        };
        let mut claims = claims_of(&mut f.claims);
        let parity = parity_of(&mut f.parity);
        let scanner = SurfaceScanner::new(&mut claims, &parity, |t| tris.push(t));
        let mut to_material = |m: &MaterialId| *m;
        let mut walker = SurfaceWalker::new(&mut materials, scanner, &mut to_material);
        dense_walk(&mut walker, &f.grid, blocks);
    }
    tris
}

/// Fills and weaves one face plane from `seed` = (row, col), without
/// claiming anything. Returns the filled cells and the emitted triangles.
pub fn scan_plane(
    f: &mut Fixture,
    axis: Axis,
    axis_coord: ChunkCoord,
    seed: (u32, u32),
) -> ([u32; PLANE_SIDE], Vec<Triangle2D>) {
    let parity = parity_of(&mut f.parity);
    let claims = claims_of(&mut f.claims);
    let weaver = Weaver::new(&parity);
    let mut tris = Vec::new();
    let layer = SurfaceLayer::new(axis, axis_coord);
    let filler = Filler::new(&claims, &parity);
    let edges = Edges::from_parity(&parity, layer);

    let filled = filler.fill_region(
        layer,
        &edges,
        Coords2D {
            u: seed.0 as u8,
            v: seed.1 as u8,
        },
    );

    weaver.weave_region(layer, &edges, &filled, |t| tris.push(t));
    #[cfg(debug_assertions)]
    debug_assert_within_footprint(&parity, layer, &filled);

    (filled, tris)
}
