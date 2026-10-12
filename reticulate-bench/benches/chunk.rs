use criterion::{Criterion, criterion_group, criterion_main};
use reticulate::Coords3D;
use reticulate_bench::helpers::bench_chunk;
use reticulate_demo::chunks::*;

fn chunk_sphere_single_mat(c: &mut Criterion) {
    bench_chunk(
        c,
        "single_material",
        geometries::sphere(Coords3D::from_xyz(16, 16, 16), 14, materials::single),
    );
}

fn chunk_full_checkerboard(c: &mut Criterion) {
    bench_chunk(c, "single_material", grids::checkerboard(materials::single));
}

fn chunk_ortho_labyrinth(c: &mut Criterion) {
    bench_chunk(
        c,
        "single_material",
        grids::ortho_labyrinth(materials::single),
    );
}

fn chunk_gyroid(c: &mut Criterion) {
    bench_chunk(c, "single_material", grids::gyroid(materials::single));
}

fn chunk_terrain_single_material(c: &mut Criterion) {
    bench_chunk(
        c,
        "single_material",
        heightmaps::terrain(materials::terrain_single),
    );
}

fn chunk_terrain_layered_material(c: &mut Criterion) {
    bench_chunk(
        c,
        "layered_material",
        heightmaps::terrain(materials::terrain_layered),
    );
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
        chunk_sphere_single_mat,
        chunk_full_checkerboard,
        chunk_ortho_labyrinth,
        chunk_gyroid,
        chunk_terrain_single_material,
        chunk_terrain_layered_material,
);

criterion_main!(benches);
