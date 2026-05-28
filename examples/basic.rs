// use quick_noise::simd::simd_vec::core::Simd;
use std::hint::black_box;
use std::time::Instant;

use itertools::izip;
use quick_noise::emit::grayscale::NoiseImageExt;
use quick_noise::perlin::Octave2D;
use quick_noise::simd::arch_simd::ArchSimd;
use quick_noise::simd::simd_array::SimdArray;
use quick_noise::simd::simd_vec::SimdVec;
use quick_noise::simplex::Simplex;
use quick_noise::testing::profiler;
// use quick_noise::perlin::Perlin;
use quick_noise::testing::profiler as unofficial_profiler;
// use criterion::profiler;
// use quick_noise::emit::grayscale;
use quick_noise::{Batch2D, Batch3D, Grid2D, Grid3D};

fn main() {
    let grid_2d = Grid2D::<500, 500, 250000>::new().position(0, 0).seed(102);
    
    let noise1 = grid_2d.perlin()
        .octaves(5)
        .frequency(0.01)
        .seed(0)
        .into_iter();

    let noise2 = grid_2d.perlin()
        .octaves(5)
        .frequency(0.01)
        .seed(1)
        .into_iter();

    grid_2d.perlin_warp()
        .input_iters(noise1, noise2)
        .strength(100.0)
        .octaves(6)
        .frequency(0.03)
        .into_iter()
        .to_grayscale_image::<500, 500>("noise_images_new/warp_2d.png");

    // Custom Octaves

    // let octave_list = vec![
    //     Octave2D::splat(0.05, 1.0),
    //     Octave2D::splat(0.02, 0.8),
    //     Octave2D::splat(0.03, 0.6),
    //     Octave2D::splat(0.04, 0.4),
    //     Octave2D::splat(0.05, 0.2),
    // ];

    // grid_2d.custom_perlin()
    //     .octave_list(&octave_list.as_slice())
    //     .into_iter()
    //     .to_grayscale_image::<500, 500>("noise_images_new/perlin_custom_2d.png");

    // let res = Batch2D::<120000>::custom_perlin()
    //     .seed(8)
    //     .input_iters(grid_2d.x_iter(), grid_2d.y_iter())
    //     .octave_list(octave_list.as_slice())
    //     .into_iter()
    //     .to_grayscale_image::<300, 400>("noise_images_new/custom_batched_2d.png");
}