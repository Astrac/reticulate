use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use reticulate::meshing::triangulation::{Triangle2D, zipper::Zipper};

const ALL_COLUMNS: u32 = u32::MAX;

fn zipper_with(mask: u32, height: u8) -> Zipper {
    Zipper {
        bottom_column_heights: [height; 32],
        bottom_columns_mask: mask,
    }
}

/// Deterministic pseudo-random mask with roughly `density` out of 8 bits set.
fn scattered_mask(seed: u32, density: u32) -> u32 {
    let mut state = seed;
    let mut mask = 0;
    for bit in 0..32 {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        if (state >> 24) % 8 < density {
            mask |= 1 << bit;
        }
    }
    mask
}

fn bench_zip(c: &mut Criterion) {
    let mut group = c.benchmark_group("zipper/zip");

    let cases: [(&str, u32, u32, u32); 4] = [
        ("sparse", 0x8000_0001, 0x8000_0001, ALL_COLUMNS),
        ("dense", ALL_COLUMNS, ALL_COLUMNS, ALL_COLUMNS),
        (
            "scattered",
            scattered_mask(1, 4),
            scattered_mask(2, 4),
            ALL_COLUMNS,
        ),
        (
            "scattered_partial_scan",
            scattered_mask(3, 6),
            scattered_mask(4, 6),
            0x00FF_FF00,
        ),
    ];

    for (name, bottom, top, scan) in cases {
        group.bench_function(name, |b| {
            b.iter_batched(
                || zipper_with(bottom, 0),
                |mut zipper| {
                    let mut count = 0usize;
                    zipper.zip(
                        black_box(scan),
                        black_box(1),
                        black_box(top),
                        &mut |t: Triangle2D| {
                            count += 1;
                            black_box(t);
                        },
                    );
                    black_box((count, zipper))
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn bench_advance(c: &mut Criterion) {
    let mut group = c.benchmark_group("zipper/advance");

    for (name, mask) in [
        ("sparse", 0x8000_0001),
        ("scattered", scattered_mask(5, 4)),
        ("dense", ALL_COLUMNS),
    ] {
        group.bench_function(name, |b| {
            b.iter_batched(
                || zipper_with(0, 0),
                |mut zipper| {
                    zipper.advance(black_box(3), black_box(mask), black_box(ALL_COLUMNS));
                    black_box(zipper)
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// Zips many rows in sequence, the way a real run scan drives the zipper.
fn bench_row_sweep(c: &mut Criterion) {
    let rows: Vec<u32> = (0..32).map(|i| scattered_mask(100 + i, 5)).collect();

    c.bench_function("zipper/row_sweep_32", |b| {
        b.iter(|| {
            let mut zipper = zipper_with(0, 0);
            let mut count = 0usize;
            for (row, &mask) in rows.iter().enumerate() {
                zipper.zip(ALL_COLUMNS, row, mask, &mut |t| {
                    count += 1;
                    black_box(t);
                });
                zipper.advance(row, mask, ALL_COLUMNS);
            }
            black_box(count)
        })
    });
}

criterion_group!(benches, bench_zip, bench_advance, bench_row_sweep);
criterion_main!(benches);
