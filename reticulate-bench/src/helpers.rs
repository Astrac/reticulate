use std::hint::black_box;

use criterion::Criterion;
use reticulate::{CompactOctree, Fixed8Octree, Mesher};

pub fn bench_chunk(c: &mut Criterion, id: &str, mut demo: impl FnMut(&mut Fixed8Octree<u16>)) {
    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::init_metrics();

    let mut mesher = Mesher::new();
    let mut tree = Fixed8Octree::new();
    demo(&mut tree);

    c.bench_function(&format!("{id}_fixed_8"), |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        });
    });

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();

    let tree = CompactOctree::from_fixed8(&tree);
    c.bench_function(&format!("{id}_compact"), |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        });
    });

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();
}

pub fn mesh_chunk(id: &str, mut demo: impl FnMut(&mut Fixed8Octree<u16>)) {
    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::init_metrics();

    let mut mesher = Mesher::new();
    let mut tree = Fixed8Octree::new();
    demo(&mut tree);

    println!("Meshing scenario: {id}_fixed_8");
    for _ in 0..100 {
        mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
    }

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();

    let tree = CompactOctree::from_fixed8(&tree);
    println!("Meshing scenario: {id}_compact");
    for _ in 0..100 {
        mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
    }

    #[cfg(feature = "metrics-recording")]
    reticulate::metrics::report_and_reset();
}
