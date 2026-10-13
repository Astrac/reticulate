use std::hint::black_box;

use criterion::Criterion;
use reticulate::{CompactOctree, Mesher};
use reticulate_demo::ChunkDemo;

#[derive(Debug)]
enum OctreeRepr {
    Compact,
    Fixed8,
}

fn bench_id(demo: &ChunkDemo, id: &str, repr: OctreeRepr) -> String {
    format!("Chunk::{}::{}::{:?}", demo.id(), id, repr)
}

pub fn bench_chunk(c: &mut Criterion, id: &str, demo: ChunkDemo) {
    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::init_metrics();

    let mut mesher = Mesher::new();
    let tree = demo.make_tree();

    c.bench_function(&bench_id(&demo, id, OctreeRepr::Fixed8), |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        });
    });

    mesher.reset();
    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();

    let tree = CompactOctree::from_fixed8(&tree);
    c.bench_function(&bench_id(&demo, id, OctreeRepr::Compact), |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        });
    });

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();
}

pub fn mesh_chunk(id: &str, demo: ChunkDemo) {
    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::init_metrics();

    let mut mesher = Mesher::new();
    let tree = demo.make_tree();

    println!(
        "Meshing scenario: {}",
        bench_id(&demo, id, OctreeRepr::Fixed8)
    );

    for _ in 0..100 {
        mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
    }

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();

    let tree = CompactOctree::from_fixed8(&tree);

    println!(
        "Meshing scenario: {}",
        bench_id(&demo, id, OctreeRepr::Compact)
    );

    for _ in 0..100 {
        mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
    }

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();
}
