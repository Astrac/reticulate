use reticulate::Coords3D;
use reticulate_bench::helpers::mesh_chunk;
use reticulate_demo::chunks::*;

pub fn main() {
    println!("Running profiling workload");

    mesh_chunk(
        "single_material",
        geometries::sphere(Coords3D::from_xyz(16, 16, 16), 14, materials::single),
    );

    mesh_chunk("single_material", grids::checkerboard(materials::single));

    mesh_chunk("single_material", grids::ortho_labyrinth(materials::single));

    mesh_chunk("single_material", grids::gyroid(materials::single));

    mesh_chunk(
        "single_material",
        heightmaps::terrain(materials::terrain_single),
    );

    mesh_chunk(
        "layered_material",
        heightmaps::terrain(materials::terrain_layered),
    );
}
