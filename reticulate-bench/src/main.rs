use reticulate::Coords3D;
use reticulate_bench::helpers::mesh_chunk;

pub fn main() {
    println!("Running profiling workload");

    mesh_chunk("chunk_sphere_single_material", |t| {
        reticulate_demo::chunk_sphere(
            t,
            Coords3D::from_xyz(16, 16, 16),
            14,
            reticulate_demo::chunk_material::single,
        )
    });

    mesh_chunk("chunk_checkerboard_single_material", |t| {
        reticulate_demo::chunk_checkerboard(t, reticulate_demo::chunk_material::single)
    });

    mesh_chunk("chunk_ortho_labyrinth_single_material", |t| {
        reticulate_demo::chunk_ortho_labyrinth(t, reticulate_demo::chunk_material::single)
    });

    mesh_chunk("chunk_terrain_single_material", |t| {
        reticulate_demo::chunk_terrain(t, reticulate_demo::chunk_material::terrain_single)
    });

    mesh_chunk("chunk_terrain_layered_material", |t| {
        reticulate_demo::chunk_terrain(t, reticulate_demo::chunk_material::terrain_layered)
    });
}
