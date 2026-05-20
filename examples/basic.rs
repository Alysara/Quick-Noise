use itertools::izip;
use quick_noise::api::builders::*;
use quick_noise::emit::grayscale::NoiseImageExt;
// use criterion::profiler;
// use quick_noise::emit::grayscale;
use quick_noise::perlin::{Octave2D, PerlinMap};
use quick_noise::api::grid::{Grid2D};
use quick_noise::api::batch::{Batch2D};
use quick_noise::simd::simd_array::SimdArray;
use quick_noise::simplex::Simplex;
use quick_noise::testing::profiler;
// use quick_noise::perlin::Perlin;
use quick_noise::testing::profiler as unofficial_profiler;
// use quick_noise::simd::simd_vec::core::SimdVec;

// DISCLAIMER: Rust nightly is *needed* to run the code.
fn main() {
    // For profiling performance:
    //   NOTE: Ensure to run with "RUSTFLAGS='-C target-cpu=native' cargo run --release --example basic".
    //   If the program appears to be stuck, this is likely why. Comment out the bench to run on debug.
    // profiler::bench_perlin_2d();
    // profiler::bench_perlin_2d();
    // profiler::profile_perlin_2d_call(1, 32., 0.5, 0.5);

    // Create noise from number of octaves, initial scale, lacunarity, and persistence.
    // - Initial scale is how far apart gradients are, higher scale means larger noise/slower to change per pixel.
    // - Lacunarity determines the scale of the next octave based on the previous one by dividing it. A lacunarity
    //     of two means the next octave halves in scale.
    // - Persistence determines the weight of the next octave based on the previous one. A persistence of 0.5 means
    //     each successive octave is half as noticeable as the previous one.
    // - Channel allows for differing results for the same seed and octave scale. Acts as a second seed.

    // unofficial_profiler::profile_perlin_2d_batched_call(1, 32.0, 2.0, 0.5);
    // grayscale::write_perlin_height_map_batched("noise_images/batched_pass.png", 32, 1, 1.0/32.0, 2.0, 0.5);
    // grayscale::write_simplex_height_map_batched("noise_images/simplex_batched.png", 32, 1, 1.0/64.0, 2.0, 0.5);
    // grayscale::write_value_height_map_batched("noise_images/value_batched.png", 32, 1, 1.0/32.0, 2.0, 0.5);
    // grayscale::write_cellular_height_map_batched("noise_images/cellular_batched.png", 32, 1, 1.0/64.0, 2.0, 0.5);
    // grayscale::write_perlin_height_map_batched_3d("noise_images/batched_pass_3d.png", 32, 1, 1.0/32.0, 2.0, 0.5);
    // grayscale::write_value_height_map_batched_3d("noise_images/batched_value_3d.png", 32, 1, 1.0/16.0, 2.0, 0.5);
    // grayscale::write_cellular_height_map_batched_3d("noise_images/batched_cellular_3d.png", 32, 1.0/32.0);
    // grayscale::write_simplex_height_map_batched_3d("noise_images/batched_simplex_3d.png",32, 1, 1.0/64.0, 2.0, 0.5);

    // grayscale::write_perlin_height_map_3d("noise_images/single_pass_3d.png", 128, 1, 2., 0.999, 0.999);

    // grayscale::write_perlin_height_map_warped("noise_images/perlin_warp.png", 32, 7, 64.0, 1.5, 0.5);
    // grayscale::write_perlin_height_map_3d_octaves(
    //     "noise_images/perlin_3d_custom.png",
    //     256,
    //     [
    //         ((800., 600., 800.), 1.),
    //         ((400., 300., 400.), 0.5),
    //         ((200., 150., 200.), 0.25),
    //         ((100., 75., 100.), 0.125),
    //         ((50., 37.5, 50.), 0.0625),
    //         ((25., 18.75, 25.), 0.03125),
    //         ((12.5, 9.375, 12.5), 0.015625),
    //         ((6.25, 4.6875, 6.25), 0.0078125),
    //     ],
    //     0
    // );
    // grayscale::write_perlin_height_map("noise_images/single_pass.png", 32, 1, 64.0, 2.0, 0.5, 1.0);

    // grayscale::write_perlin_height_map("noise_images/glossy.png", 16, 11, 256.0, 1.5, 0.7, 0.5);
    // grayscale::write_perlin_height_map("noise_images/chiseled.png", 32, 6, 256.0, 2.0, 0.8, 1.0);
    // grayscale::write_perlin_height_map("noise_images/smooth.png", 32, 20, 512.0, 1.2, 0.9, 1.0);
    // grayscale::write_perlin_height_map("noise_images/sharp.png", 32, 6, 64.0, 2.0, 0.9, 1.0);

    // For more control, determine the scale and weight of each octave:
    // grayscale::write_perlin_octaves_height_map(
    //     "noise_images/custom.png",  
    //     256,
    //     [
    //         (300.0, 8.0),
    //         (250.0, 7.0),
    //         (200.0, 4.0),
    //         (100.0, 3.0),
    //         (50.0, 1.0),
    //         (25.0, 2.0),
    //         (12.5, 1.0),
    //     ],
    //     1,
    // );

    // Each axis can be scaled independently of eachother as well, like so:
    // grayscale::write_perlin_octaves_height_map(
    //     "noise_images/denim.png",
    //     32,
    //     [
    //         ((64.0, 512.0), 8.0),
    //         ((512.0, 64.0), 8.0),
    //         ((32.0, 256.0), 4.0),
    //         ((256.0, 32.0), 4.0),
    //         ((16.0, 128.0), 2.0),
    //         ((128.0, 16.0), 2.0),
    //         ((8.0, 64.0), 1.0),
    //         ((64.0, 8.0), 1.0),
    //     ],
    //     1,
    // );

    // grayscale::write_perlin_height_map_3d("noise_images/single_pass_3d_2.png", 32, 1, 32.0, 2.0, 0.5);




    // Use black_box to prevent optimization
    use std::hint::black_box;

    let scale = 1.0 / 32.0;

    // VERTICAL
    // let mut simplex = Simplex::new(0);
    // let mut x_array = PerlinMap::new_uninit();
    // let mut y_array = PerlinMap::new_uninit();
    // let mut output = PerlinMap::new_uninit();
    // for x in black_box(0..32) {
    //     for y in black_box(0..32) {
    //         x_array[x * 32 + y] = x as f32;
    //         y_array[x * 32 + y] = y as f32;
    //     }
    // }
    // simplex.batched_2d(&mut output, &x_array, &y_array, scale, 1.0, 1, 0.0);
    // black_box(output);

    // HORIZONTAL
    // let arr: SimdArray::<f32, 1024> =
    //     izip!(Node::x_iter_2d(0.0), Node::y_iter_2d(0.0))
    //         .map(|(x, y)| Simplex::batch_2d(x, y, scale, 12345676543))
    //         .collect();
    // black_box(arr);

// let noise = grid.perlin()
//     .octaves(4)
//     .frequency(0.1)
//     .seed(0)
//     .build();

//     println!("res! {:?}", x_noise);

let grid = Grid2D::<1024,1024, 1048576>::new()
    .position(0, 0)
    .seed(0);

Batch2D::<1048576>::simplex()
    .input_iters(grid.x_iter(), grid.y_iter())
    .seed(100)
    .frequency(0.01)
    .into_iter()
    .to_grayscale_image::<1024, 1024>("noise_images_new/simplex_2d.png");

// let custom_noise = grid.custom_perlin()
//     .add_octave(16.0)
//     .add_octave(Octave2D::new((16.))
//     .add_octave(Octave2D::new(Vec2::splat(8.0), 1.0))
//     .build();

// let y_noise = grid.perlin()
//     .octaves(4)
//     .frequency(0.1)
//     .seed(1)
//     .into_iter();

// let result = grid.simplex_warp()
//     .x_offset_iter(x_noise)
//     .y_offset_iter(y_noise)
//     .frequency(0.1)
//     .strength(1.0)
//     .build();

// let batch = Batch2D::perlin()
//     .input(x_noise, y_noise)
//     .frequency(0.1)
//     .strength(1.0)
//     .build();
}