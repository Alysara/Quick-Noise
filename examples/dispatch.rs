use std::hint::black_box;
use std::time::Instant;

use quick_noise::{BatchNoise, Fbm, Grid, Perlin};
use quick_noise::simd::{dispatch_simd};

fn main() {
    simd_work();
}

#[dispatch_simd(A)]
fn simd_work() {
    let grid = Grid::<2, A>::new(1024, 1024);

    const GRID_2D: usize = 1024;
    const GRID_2D_AREA: usize = GRID_2D * GRID_2D;
    const OCTAVES_2D: usize = 3;
    const BASE_FREQ_2D: f64 = 1.0 / 128.0;

    let start = Instant::now();
    let mut result = vec![0.0; GRID_2D_AREA];
    const NUM_RUNS: usize = 100;
    for _ in 0..NUM_RUNS {
        BatchNoise::<2, Fbm, Perlin>::builder(grid.x_iter(), grid.y_iter())
            .octaves(OCTAVES_2D)
            .frequency(BASE_FREQ_2D as f32)
            .fill(result.as_mut_slice());
    }
    black_box(&result);

    let elapsed = start.elapsed();
    println!(
        "Batch 2D Perlin Results\n---------------------------\nTotal: {:?}\nAvg: {:?}",
        elapsed,
        elapsed.div_f32(NUM_RUNS as f32)
    );

    black_box(&result);
}

