use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use quick_noise::{Fbm, Grid, Value};

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

// fn simd_array_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("simd_array");
//     group.throughput(Throughput::Elements(1024));
//     group.bench_function("iota_custom", |b| {
//         b.iter(|| {
//             // black_box(&arr1).mul_add(black_box(*&arr2), black_box(*&arr3))
//             black_box(SimdArray::<f32, 2050>::iota_custom(
//                 black_box(0.),
//                 black_box(0.4),
//             ));
//         })
//     });
//
//     group.finish();
// }
//
// fn grid_perlin_2d_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("perlin_noise_2d");
//
//     const GRID_SIZE: usize = 32;
//     const GRID_AREA: usize = GRID_SIZE * GRID_SIZE;
//     for scale in SCALES {
//         group.throughput(Throughput::Elements(GRID_AREA as u64));
//
//         let grid = Grid::<2>::new(GRID_SIZE, GRID_SIZE);
//         // let mut result = unsafe { SimdArray::<f32, 1024>::new_uninit() };
//         // let mut result = vec![0.0; GRID_AREA];
//         let mut result = [0.0; 1024];
//
//         let freq = 1.0 / scale;
//         group.bench_function(format!("scale: {scale}"), |b| {
//             b.iter(|| {
//                 // for _ in 0..100_000_000 {
//                 grid.builder::<Fbm, Perlin>()
//                     .octaves(1)
//                     .frequency(freq)
//                     .fill(result.as_mut_slice());
//
//                 // result.fill(123.0);
//
//                 black_box(&result);
//                 // }
//                 // manual_timing_check();
//
//                 // Perlin::grid_2d::<true>(
//                 //     123123123,
//                 //     result.as_array_mut().as_mut_slice(),
//                 //     Vec2::new(32, 32),
//                 //     Vec2::new(0, 0),
//                 //     Vec2::new(freq, freq),
//                 //     1.0,
//                 //     1.0,
//                 //     Vec2::new(None, None),
//                 // );
//
//                 // PerlinGridNoise2D::<32, 32, 1024>::grid_2d::<true>(
//                 //     12312312,
//                 //     &mut result,
//                 //     Vec2::new(0, 0),
//                 //     Vec2::new(freq, freq),
//                 //     1.0,
//                 //     1.0,
//                 //     Vec2::new(None, None),
//                 // );
//             });
//         });
//
//         black_box(&result);
//     }
// }

fn grid_value_2d_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_noise_2d");

    const GRID_SIZE: usize = 32;
    const GRID_AREA: usize = GRID_SIZE * GRID_SIZE;
    for scale in SCALES {
        group.throughput(Throughput::Elements(GRID_AREA as u64));

        let grid = Grid::<2>::new(GRID_SIZE, GRID_SIZE);
        // let mut result = unsafe { SimdArray::<f32, 1024>::new_uninit() };
        // let mut result = vec![0.0; GRID_AREA];
        let mut result = [0.0; 1024];

        let freq = 1.0 / scale;
        group.bench_function(format!("scale: {scale}"), |b| {
            b.iter(|| {
                // for _ in 0..100_000_000 {
                grid.builder::<Fbm, Value>()
                    .octaves(1)
                    .frequency(freq)
                    .fill(result.as_mut_slice());
                black_box(&result);
            });
        });

        black_box(&result);
    }
}
//
// fn grid_perlin_3d_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("perlin_noise_3d");
//     for scale in SCALES {
//         const GRID_SIZE: usize = 32;
//         const GRID_VOLUME: usize = GRID_SIZE * GRID_SIZE * GRID_SIZE;
//         group.throughput(Throughput::Elements(GRID_VOLUME as u64));
//
//         let grid = Grid::<3>::new(GRID_SIZE, GRID_SIZE, GRID_SIZE);
//         // let mut result = vec![0.0; GRID_VOLUME];
//         // let mut result = unsafe { SimdArray::<f32, 32768>::new_uninit() };
//         let mut result = [0.0; 32768];
//
//         group.bench_function(format!("scale: {scale}"), |b| {
//             b.iter(|| {
//                 grid.builder::<Fbm, Perlin>()
//                     .frequency(1.0 / scale)
//                     .fill(result.as_mut_slice());
//                 black_box(&result);
//             });
//         });
//     }
// }
//
// fn grid_value_3d_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("value_noise_3d");
//     for scale in SCALES {
//         const GRID_SIZE: usize = 32;
//         const GRID_VOLUME: usize = GRID_SIZE * GRID_SIZE * GRID_SIZE;
//         group.throughput(Throughput::Elements(GRID_VOLUME as u64));
//
//         let grid = Grid::<3>::new(GRID_SIZE, GRID_SIZE, GRID_SIZE);
//         // let mut result = vec![0.0; GRID_VOLUME];
//         // let mut result = unsafe { SimdArray::<f32, 32768>::new_uninit() };
//         let mut result = [0.0; 32768];
//
//         group.bench_function(format!("scale: {scale}"), |b| {
//             b.iter(|| {
//                 grid.builder::<Fbm, Value>()
//                     .frequency(1.0 / scale)
//                     .fill(result.as_mut_slice());
//                 black_box(&result);
//             });
//         });
//     }
// }

// fn batch_2d_benchmark(c: &mut Criterion) {
//     let mut group = c.benchmark_group("batch_2d");
//     group.throughput(Throughput::Elements(1024));

//     let grid = Grid2D::<32, 32, 1024>::new();
//     let mut result = unsafe { SimdArray::<f32, 1024>::new_uninit() };

//     let x_coords: SimdArray<f32, 1024> = grid.x_iter().collect();
//     let y_coords: SimdArray<f32, 1024> = grid.y_iter().collect();

//     group.bench_function(format!("perlin_batch_2d"), |b| {
//         b.iter(|| {
//             Batch2D::fbm::<Perlin, 1024>(x_coords.iter_simd(), y_coords.iter_simd()).fill(&mut result);
//         });
//     });
// }

// fn manual_timing_check() {
//     let grid = Grid::<2>::new(32, 32);
//     let mut result = unsafe { SimdArray::<f32, 1024>::new_uninit() };
//     let freq = 1.0 / 64.0;
//
//     const NUM_RUNS: usize = 100_000_000;
//     let time = Instant::now();
//     for _ in 0..NUM_RUNS {
//         grid.fbm::<Perlin>()
//             .octaves(1)
//             .frequency(freq)
//             .fill(result.as_array_mut().as_mut_slice());
//     }
//     std::hint::black_box(&result);
//     println!(
//         "Manual (in bench binary): {:?}",
//         time.elapsed() / NUM_RUNS as u32
//     );
// }

// criterion_group!(benches, simd_array_benchmark, simd_vec_benchmark);
criterion_group!(benches, grid_value_2d_benchmark);
criterion_main!(benches);
