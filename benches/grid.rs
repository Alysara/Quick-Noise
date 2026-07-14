use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use quick_noise::{Fbm, Grid, Perlin};

const SCALES: [f32; 11] = [64.0, 48.0, 32.0, 24.0, 16.0, 12.0, 8.0, 6.0, 4.0, 3.0, 2.0];

fn grid_2d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("grid_noise_2d");
    group.throughput(Throughput::Elements(1024));

    let mut result = [0.0; 1024];
    for scale in SCALES {
        let grid = Grid::<2>::new(32, 32);

        group.bench_function(format!("scale: {scale}"), |b| {
            b.iter(|| {
                grid.builder::<Fbm, Perlin>()
                    .octaves(1)
                    .frequency(1.0 / scale)
                    .fill(&mut result);
                black_box(&result);
            });
        });
    }
}

// criterion_group!(benches, simd_array_benchmark, simd_vec_benchmark);
criterion_group!(benches, grid_2d_benchmark);
criterion_main!(benches);
