// use criterion::profiler;
use quick_noise::emit::grayscale;
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
    // grayscale::write_worley_height_map_batched("noise_images/worley_batched.png", 32, 1, 1.0/64.0, 2.0, 0.5);
    // grayscale::write_perlin_height_map_batched_3d("noise_images/batched_pass_3d.png", 32, 1, 1.0/32.0, 2.0, 0.5);
    // grayscale::write_value_height_map_batched_3d("noise_images/batched_value_3d.png", 32, 1, 1.0/16.0, 2.0, 0.5);
    // grayscale::write_cellular_height_map_batched_3d("noise_images/batched_cellular_3d.png", 32, 1.0/32.0);
    // grayscale::write_simplex_height_map_batched_3d("noise_images/batched_simplex_3d.png",32, 1, 1.0/64.0, 2.0, 0.5);

    // grayscale::write_perlin_height_map_3d("noise_images/single_pass_3d.png", 256, 8, 600., 1.9, 0.5);
    grayscale::write_perlin_height_map_3d_octaves(
        "noise_images/perlin_3d_custom.png",
        32,
        [
            (1024.0, 1.0)
        ],
        0
    );
    // grayscale::write_perlin_height_map("noise_images/single_pass.png", 32, 1, 32.0, 2.0, 0.5);

    // grayscale::write_perlin_height_map("noise_images/glossy.png", 32, 11, 256.0, 1.5, 0.7);
    // grayscale::write_perlin_height_map("noise_images/chiseled.png", 32, 6, 256.0, 2.0, 0.8);
    // grayscale::write_perlin_height_map("noise_images/smooth.png", 32, 20, 512.0, 1.2, 0.9);
    // grayscale::write_perlin_height_map("noise_images/sharp.png", 32, 6, 64.0, 2.0, 0.9);

    // For more control, determine the scale and weight of each octave:
    // grayscale::write_perlin_octaves_height_map(
    //     "noise_images/custom.png",
    //     32,
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

    // grayscale::write_perlin_height_map_3d("noise_images/single_pass_3d.png", 32, 1, 32.0, 2.0, 0.5);
}
