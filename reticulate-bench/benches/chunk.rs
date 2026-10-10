use criterion::{Criterion, criterion_group, criterion_main};
use reticulate::Coords3D;
use reticulate_bench::helpers::bench_chunk;

fn chunk_sphere_single_mat(c: &mut Criterion) {
    bench_chunk(c, "chunk_sphere_single_material", |t| {
        reticulate_demo::chunk_sphere(
            t,
            Coords3D::from_xyz(16, 16, 16),
            14,
            reticulate_demo::chunk_material::single,
        )
    });
}

fn chunk_full_checkerboard(c: &mut Criterion) {
    bench_chunk(c, "chunk_checkerboard_single_material", |t| {
        reticulate_demo::chunk_checkerboard(t, reticulate_demo::chunk_material::single)
    });
}

fn chunk_ortho_labyrinth(c: &mut Criterion) {
    bench_chunk(c, "chunk_ortho_labyrinth_single_material", |t| {
        reticulate_demo::chunk_ortho_labyrinth(t, reticulate_demo::chunk_material::single)
    });
}

fn chunk_terrain_single_material(c: &mut Criterion) {
    bench_chunk(c, "chunk_terrain_single_material", |t| {
        reticulate_demo::chunk_terrain(t, reticulate_demo::chunk_material::terrain_single)
    });
}

fn chunk_terrain_layered_material(c: &mut Criterion) {
    bench_chunk(c, "chunk_terrain_layered_material", |t| {
        reticulate_demo::chunk_terrain(t, reticulate_demo::chunk_material::terrain_layered)
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
        chunk_sphere_single_mat,
        chunk_full_checkerboard,
        chunk_ortho_labyrinth,
        chunk_terrain_single_material,
        chunk_terrain_layered_material,
);

criterion_main!(benches);
