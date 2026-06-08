// use quick_noise::simd::simd_vec::core::Simd;
use std::hint::black_box;
use std::time::Instant;

use itertools::izip;
use quick_noise::emit::grayscale::NoiseImageExt;
use quick_noise::simd::arch_simd::ArchSimd;
use quick_noise::simd::simd_array::SimdArray;
use quick_noise::simd::simd_vec::SimdVec;
use quick_noise::simplex::Simplex;
use quick_noise::testing::profiler;
// use quick_noise::perlin::Perlin;
use quick_noise::testing::profiler as unofficial_profiler;
// use criterion::profiler;
// use quick_noise::emit::grayscale;
use quick_noise::{Batch2D, Grid2D, Grid3D, Octave2D, Perlin};

fn main() {
    let grid_2d = Grid2D::<500, 500, 250000>::new().position(0, 0).seed(102);
    //
    // grid_2d
    //     .fbm::<Perlin>()
    //     .octaves(6)
    //     .frequency(0.03)
    //     .into_iter()
    //     .to_grayscale_image::<500, 500>("noise_images/perlin_grid_2d.png");
    //
    // // Custom Octaves
    //
    let octave_list = vec![
        Octave2D::splat(0.05, 1.0),
        Octave2D::splat(0.02, 0.8),
        Octave2D::splat(0.03, 0.6),
        Octave2D::splat(0.04, 0.4),
        Octave2D::splat(0.05, 0.2),
    ];
    //
    // grid_2d
    //     .custom::<Perlin>(octave_list.as_slice())
    //     .into_iter()
    //     .to_grayscale_image::<500, 500>("noise_images/perlin_custom_2d.png");

    Batch2D::fbm::<Perlin, 250000>(grid_2d.x_iter(), grid_2d.y_iter())
        .octaves(6)
        .frequency(0.01)
        .into_iter()
        .to_grayscale_image::<500, 500>("noise_images/perlin_batch_2d.png");

    Batch2D::custom::<Perlin, 250000>(octave_list.as_slice(), grid_2d.x_iter(), grid_2d.y_iter())
        .into_iter()
        .to_grayscale_image::<500, 500>("noise_images/perlin_custom_batch_2d.png");

    // let res = Batch2D::<120000>::custom_perlin()
    //     .seed(8)
    //     .input_iters(grid_2d.x_iter(), grid_2d.y_iter())
    //     .octave_list(octave_list.as_slice())
    //     .into_iter()
    //     .to_grayscale_image::<300, 400>("noise_images/custom_batched_2d.png");
}
