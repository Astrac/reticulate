mod camera;
mod chunk_mesh;
mod setup;

pub use camera::{CameraPlugin, CameraSettingsPlugin};
pub use chunk_mesh::ChunkMesh;
use reticulate::Coords3D;
pub use setup::setup;

use std::sync::LazyLock;

use reticulate_demo::{ChunkDemo, chunks::*};

pub static DEMOS: LazyLock<Vec<ChunkDemo>> = LazyLock::new(|| {
    vec![
        geometries::sphere(Coords3D::from_xyz(16, 16, 16), 14, materials::single),
        geometries::cube_2x2_missing_corner(materials::single),
        geometries::nested_cubes(materials::single),
        geometries::tunnel(materials::single),
        grids::checkerboard(materials::single),
        grids::ortho_labyrinth(materials::single),
        grids::gyroid(materials::single),
        heightmaps::terrain(materials::terrain_single),
    ]
});
