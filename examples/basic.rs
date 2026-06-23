// use quick_noise::simd::simd_vec::core::Simd;
use std::hint::black_box;
use std::time::Instant;

use itertools::izip;
use quick_noise::emit::grayscale::NoiseImageExt;
use quick_noise::math::Vec2;
use quick_noise::simd::arch_simd::ArchSimd;
use quick_noise::simd::simd_array::SimdArray;
use quick_noise::simplex::Simplex;
use quick_noise::testing::profiler;
// use quick_noise::perlin::Perlin;
use quick_noise::testing::profiler as unofficial_profiler;
// use criterion::profiler;
// use quick_noise::emit::grayscale;
use quick_noise::{Batch2D, Batch3D, Cellular, Grid2D, Grid3D, Octave2D, Perlin, Value};

#[cfg(feature = "image")]
fn main() {
    use std::time::{SystemTime, UNIX_EPOCH};

    const GRID_SEED: i64 = 124384833;
    const FBM_SEED: i64 = 91191912;
    let grid_2d = Grid2D::<512, 512, 262144>::new()
        .position(0, 0)
        .seed(GRID_SEED);

    let grid_3d = Grid3D::<64, 64, 64, 262144>::new()
        .position(0, 0, 0)
        .seed(GRID_SEED);

    let n: i64 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64; // or as_secs(), as_nanos()

    grid_2d
        .fbm::<Perlin>()
        .frequency(1. / 128.)
        .octaves(6)
        .seed(FBM_SEED)
        .into_iter()
        .to_grayscale_image::<512, 512>("noise_images/perlin_grid_2d_seeded.png");

    Batch2D::fbm::<Perlin, 262144>(grid_2d.x_iter(), grid_2d.y_iter())
        .frequency(1. / 128.)
        .octaves(6)
        .seed_with_grid(GRID_SEED, FBM_SEED)
        .into_iter()
        .to_grayscale_image::<512, 512>("noise_images/perlin_batch_2d_seeded.png");

    // let mut array = unsafe { SimdArray::<f32, 262144>::new_uninit() };

    // grid_2d
    //     .fbm::<Perlin>()
    //     .octaves(2)
    //     .frequency(1.0 / 64.0)
    //     .amplitude(0.75)
    //     .fill(&mut array);

    // grid_2d
    //     .fbm::<Value>()
    //     .octaves(6)
    //     .frequency(1.0 / 32.0)
    //     .amplitude(0.25)
    //     .fill_onto(&mut array);

    // array
    // .into_iter()
    // .to_grayscale_image::<512, 512>("noise_images/perlin_value_grid_combo.png");

    // .into_iter()
    // .to_grayscale_image::<512, 512>("noise_images/value_grid_2d.png");

    // grid_3d
    //     .fbm::<Value>()
    //     .octaves(1)
    //     .frequency(1.0 / 4.0)
    //     .into_iter()
    //     .to_grayscale_image::<64, 64>("noise_images/value_grid_3d.png");
    //
    // Batch3D::fbm::<Value, 262144>(grid_3d.x_iter(), grid_3d.y_iter(), grid_3d.z_iter())
    //     .octaves(1)
    //     .frequency(1.0 / 4.0)
    //     .into_iter()
    //     .to_grayscale_image::<64, 64>("noise_images/value_batch_3d.png");

    // grid_2d
    //     .fbm::<Perlin>()
    //     .octaves(6)
    //     .frequency(0.03)
    //     .into_iter()
    //     .to_grayscale_image::<500, 500>("noise_images/perlin_grid_2d.png");
    //
    // // Custom Octaves

    // let octave_list = vec![
    //     // Creates octaves from frequency and weight.
    //     Octave2D::splat(0.05, 7.0),
    //     Octave2D::splat(0.02, 4.0),
    //     Octave2D::splat(0.03, 15.0),
    //     Octave2D::splat(0.04, 9.0),
    //     Octave2D::splat(0.05, 15.0),
    //     // Allows axis-specific granularity for frequency.
    //     // This creates 'stretched' noise.
    //     Octave2D::new(Vec2::new(0.01, 0.015), 50.0),
    // ];

    // FBM Grid noise with all parameters.
    // let noise = grid_2d
    //     .fbm::<Perlin>()
    //     .seed(0)
    //     .octaves(1)
    //     .frequency(0.03125)
    //     .lacunarity(2.0)
    //     .persistence(0.5)
    //     .amplitude(1.0)
    //     .normalization(true)
    //     .scaling(1.0, 1.0);

    // grid_2d
    //     .custom::<Perlin>(octave_list.as_slice())
    //     .into_iter()
    //     .to_grayscale_image::<500, 500>("noise_images/perlin_custom_2d.png");

    // let noise1 = grid_2d.fbm::<Perlin>().octaves(1).seed(0).build();

    // let grid_2d = Grid2D::<512, 512, 262144>::new()
    //   .position(0, 0)
    //   .seed(100)
    //   .tiling(Some(128), Some(128));
    //
    // grid_2d.fbm::<Perlin>()
    //   .octaves(6)
    //   .frequency(1.0 / 128.0)
    //   .into_iter()
    //   .to_grayscale_image::<512, 512>("noise_images/perlin_tiles_2d.png");

    // grid_3d.fbm::<Perlin>()
    //     .seed(81)
    //     .octaves(4)
    //     .frequency(1.0 / 16.0)
    //     .into_iter()
    //     .to_grayscale_image::<64, 4096>("noise_images/perlin_tiling_3d.png");

    // grid_2d
    //     .fbm::<Perlin>()
    //     .seed(47)
    //     .octaves(6)
    //     .frequency(1.0 / 32.0)
    //     .into_iter()
    //     .to_grayscale_image::<512, 512>("noise_images/perlin_tiling_test.png");

    // let noise2 = grid_2d.fbm::<Perlin>().octaves(6).seed(1).into_iter();
    // grid_2d
    //     .warp::<Perlin>(noise1, noise2)
    //     .octaves(1)
    //     .seed(2)
    //     .into_iter()
    //     .to_grayscale_image::<150, 100>("noise_images/perlin_warp_2d.png");

    // let noise = Batch2D::fbm::<Perlin, 150000>(grid_2d.x_iter(), grid_2d.y_iter())
    //     .octaves(6)
    //     .frequency(0.01)
    //     .build();
    // .into_iter()
    // .to_grayscale_image::<500, 300>("noise_images/perlin_batch_2d.png");

    // black_box(&noise1);

    // Batch3D::fbm::<Perlin, 125000>(grid_3d.x_iter(), grid_3d.y_iter(), grid_3d.z_iter())
    //     .octaves(1)
    //     .frequency(0.1)
    //     .into_iter()
    //     .take(2500 * 40)
    //     .to_grayscale_image::<50, 50>("noise_images/perlin_batch_3d.png");
    //
    // Batch2D::custom::<Perlin, 150000>(octave_list.as_slice(), grid_2d.x_iter(), grid_2d.y_iter())
    //     .into_iter()
    //     .to_grayscale_image::<500, 300>("noise_images/perlin_custom_batch_2d.png");

    // let res = Batch2D::<120000>::custom_perlin()
    //     .seed(8)
    //     .input_iters(grid_2d.x_iter(), grid_2d.y_iter())
    //     .octave_list(octave_list.as_slice())
    //     .into_iter()
    //     .to_grayscale_image::<300, 400>("noise_images/custom_batched_2d.png");
}
