use std::hint::black_box;
use std::time::Instant;

use quick_noise::simd::array_trait::Array;
use quick_noise::simd::register::Simd;
use quick_noise::simd::{Arch, StaticSimd, dispatch_simd};
use quick_noise::{BatchNoise, Fbm, Grid, Perlin};

// #[cfg(feature = "image")]
fn main() {
    simd_work(100);
}

#[dispatch_simd(A)]
fn simd_work(val: usize) {
    let grid = Grid::<2, A>::new(1024, 1024);

    // grid.builder::<Fbm, Perlin>()
    //     .octaves(6)
    //     .into_iter()
    //     .to_grayscale_image(1024, 1024, "noise_images/dispatch.png");

    const GRID_2D: usize = 1024;
    const GRID_2D_AREA: usize = GRID_2D * GRID_2D;
    const OCTAVES_2D: usize = 3;
    const BASE_FREQ_2D: f64 = 1.0 / 128.0;

    let start = Instant::now();
    let mut result = vec![0.0; GRID_2D_AREA];
    const NUM_RUNS: usize = 100;
    for _ in 0..NUM_RUNS {
        let iter = BatchNoise::<2, Fbm, Perlin>::builder(grid.x_iter(), grid.y_iter())
            .octaves(OCTAVES_2D)
            .frequency(BASE_FREQ_2D as f32)
            .into_iter();


        black_box(&iter);
    }
    let elapsed = start.elapsed();
    println!(
        "Batch 2D Perlin Results\n---------------------------\nTotal: {:?}\nAvg: {:?}",
        elapsed,
        elapsed.div_f32(NUM_RUNS as f32)
    );

    black_box(&result);
}

// #[enable_targets(A)]
// fn simd_work_inner<A: Arch>(val: usize, depth: usize) -> f32 {
//     let simd = Simd::<f32, A>::splat(val as f32);
//     let doubled = simd + simd;
//     let scaled = doubled * Simd::<f32, A>::splat(1.0001);
//     let reduced = scaled.to_array().iter().sum();
//
//     black_box(&simd);
//
//     if depth == 0 {
//         return reduced;
//     }
//
//     let next = if (reduced as usize).is_multiple_of(2) {
//         simd_work_inner::<A>(val.wrapping_add(1), depth - 1)
//     } else {
//         simd_work_inner::<A>(val.wrapping_mul(3).wrapping_add(1), depth - 1)
//     };
//
//     black_box(next) + reduced
// }
