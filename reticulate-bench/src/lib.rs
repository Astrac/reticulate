use std::hint::black_box;

use criterion::Criterion;
use reticulate::{CompactOctree, Fixed8Octree, Mesher};

pub fn bench_chunk(c: &mut Criterion, id: &str, mut demo: impl FnMut(&mut Fixed8Octree<u16>)) {
    let mut mesher = Mesher::new();
    let mut tree = Fixed8Octree::new();
    demo(&mut tree);
    let compact = CompactOctree::from_fixed8(&tree);

    c.bench_function(&format!("{id}_fixed_8"), |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&tree), |d| *d, |_| ());
        });
    });

    c.bench_function(&format!("{id}_compact"), |b| {
        b.iter(|| {
            mesher.make_triangles(black_box(&compact), |d| *d, |_| ());
        });
    });
}
