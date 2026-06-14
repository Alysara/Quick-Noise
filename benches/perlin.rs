use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use quick_noise::simd::simd_array::SimdArray;
use quick_noise::{Batch2D, Grid2D, Grid3D, Perlin, ZeroIter};

const SCALES: [f32; 11] = [64.0, 48.0, 32.0, 24.0, 16.0, 12.0, 8.0, 6.0, 4.0, 3.0, 2.0];

// fn simd_vec_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("simd_vec");
//     group.throughput(Throughput::Elements(1024));
//     group.bench_function("iota_custom", |b| {
//         b.iter(|| {
//             // black_box(&arr1).mul_add(black_box(*&arr2), black_box(*&arr3))
//             black_box(SimdVec::<f32>::iota_custom(2050, black_box(0.), black_box(0.4)));
//         })
//     });
//
//     group.finish();
// }

fn simd_array_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_array");
    group.throughput(Throughput::Elements(1024));
    group.bench_function("iota_custom", |b| {
        b.iter(|| {
            // black_box(&arr1).mul_add(black_box(*&arr2), black_box(*&arr3))
            black_box(SimdArray::<f32, 2050>::iota_custom(
                black_box(0.),
                black_box(0.4),
            ));
        })
    });

    group.finish();
}

fn grid_perlin_2d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_2d");
    for scale in SCALES {
        group.throughput(Throughput::Elements(1024));

        let grid = Grid2D::<32, 32, 1024>::new();
        let mut result = unsafe { SimdArray::<f32, 1024>::new_uninit() };

        group.bench_function(format!("scale: {scale}"), |b| {
            b.iter(|| {
                grid.fbm::<Perlin>()
                    .frequency(1.0 / scale)
                    .fill(&mut result);
            });
        });
    }
}

fn grid_perlin_3d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_noise_2d");
    for scale in SCALES {
        group.throughput(Throughput::Elements(4096));

        let grid = Grid3D::<16, 16, 16, 4096>::new();
        let mut result = unsafe { SimdArray::<f32, 4096>::new_uninit() };

        group.bench_function(format!("scale: {scale}"), |b| {
            b.iter(|| {
                grid.fbm::<Perlin>()
                    .frequency(1.0 / scale)
                    .fill(&mut result);
                black_box(&result);
            });
        });
    }
}

fn batch_2d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_2d");
    group.throughput(Throughput::Elements(1024));

    let grid = Grid2D::<32, 32, 1024>::new();
    let mut result = unsafe { SimdArray::<f32, 1024>::new_uninit() };

    let x_coords: SimdArray<f32, 1024> = grid.x_iter().collect();
    let y_coords: SimdArray<f32, 1024> = grid.y_iter().collect();

    group.bench_function(format!("perlin_batch_2d"), |b| {
        b.iter(|| {
            Batch2D::fbm::<Perlin, 1024>(x_coords.iter(), y_coords.iter()).fill(&mut result);
        });
    });
}

// criterion_group!(benches, simd_array_benchmark, simd_vec_benchmark);
criterion_group!(benches, grid_perlin_2d_benchmark);
criterion_main!(benches);
