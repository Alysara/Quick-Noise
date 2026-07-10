// use quick_noise::simd::simd_vec::core::Simd;
use std::hint::black_box;
use std::time::Instant;

use itertools::izip;
use quick_noise::emit::grayscale::NoiseImageExt;
use quick_noise::math::Vec2;
use quick_noise::simd::SimdSliceIterExt;
use quick_noise::simd::arch_simd::ArchSimd;
use quick_noise::simd::simd_array::SimdArray;
use quick_noise::simplex::Simplex;
use quick_noise::testing::profiler;
// use quick_noise::perlin::Perlin;
use quick_noise::testing::profiler as unofficial_profiler;
// use criterion::profiler;
// use quick_noise::emit::grayscale;
use quick_noise::{Cellular, GridNoise, Octave, Perlin, Value};

#[cfg(feature = "image")]
fn main() {
    // println!("array: {:?}", array);
    // let simd = ArchSimd::<f32>::iota(4.0);

    // use std::time::{SystemTime, UNIX_EPOCH};

    const GRID_SEED: i64 = 124384833;
    const FBM_SEED: i64 = 91191912;
    let grid_2d = GridNoise::<2>::new(32, 32)
        .position(0, 0)
        .seed(GRID_SEED);

    let grid_2d_big = GridNoise::<2>::new(2047, 2047)
        .position(0, 0)
        .seed(GRID_SEED);

    let grid_3d = GridNoise::<3>::new(32, 32, 32)
        .seed(GRID_SEED);

    let mut buffer = SimdArray::<f32, 32768>::new(0.0);

    let time = Instant::now();
    const NUM_RUNS: usize = 5_000_000;
    let freq = 1. / 64.;
    for _ in 0..NUM_RUNS {
        grid_3d.fbm::<Value>().frequency(freq).fill(&mut buffer);
        black_box(&buffer);
    }
    let total = time.elapsed();
    println!(
        "Total: {:?}, Average Completion: {:?}",
        total,
        total / NUM_RUNS as u32
    );

    // grid_2d
    //     .fbm::<Perlin>()
    //     .octaves(1)
    //     .frequency(1. / 32.)
    //     .seed(FBM_SEED)
    //     .into_iter()
    //     .to_grayscale_image(32, 32, "noise_images/perlin_grid_2d.png");

    // grid_2d_big
    //     .fbm::<Value>()
    //     .octaves(6)
    //     .frequency(1. / 128.0)
    //     .seed(FBM_SEED)
    //     .into_iter()
    //     .to_grayscale_image(2047, 2047, "noise_images/value_grid_2d.png");

    // grid_2d_big
    //     .fbm::<Perlin>()
    //     .octaves(1)
    //     .frequency(1. / 128.0)
    //     .seed(FBM_SEED)
    //     .into_iter()
    //     .to_grayscale_image(2047, 2047, "noise_images/perlin_grid_2d.png");

    // grid_3d.fbm::<Perlin>()
    //     .octaves(6)
    //     .frequency(1.0 / 32.0)
    //     .into_iter()
    //     .to_grayscale_image(255, 255, "noise_images/perlin_grid_3d.png");

    // grid_3d.fbm::<Value>()
    //     .octaves(1)
    //     .frequency(1.0 / 8.0)
    //     .into_iter()
    //     .to_grayscale_image(255, 255, "noise_images/value_grid_3d.png");
}
