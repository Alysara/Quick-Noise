use quick_noise::emit::grayscale::NoiseImageExt;
use quick_noise::simd::arch_simd::ArchSimd;
use quick_noise::{Cellular, Grid, GridNoiseBuilder, Octave, Perlin, Value};

#[cfg(feature = "image")]
fn main() {
    // println!("array: {:?}", array);
    // let simd = ArchSimd::<f32>::iota(4.0);

    // use std::time::{SystemTime, UNIX_EPOCH};

    use quick_noise::{BatchNoise, BatchNoiseBuilder, Billow, Fbm, Ridged};

    const GRID_SEED: i64 = 124384833;
    const FBM_SEED: i64 = 91191912;
    let grid_2d = Grid::<2>::new(32, 32)
        .sample_position(123, 73)
        .seed(GRID_SEED);

    let grid_2d_big = Grid::<2>::new(2047, 2047)
        .sample_position(0, 0)
        .seed(GRID_SEED);

    let grid_3d = Grid::<3>::new(255, 255, 255)
        .grid_position(-1, 0, 0)
        .seed(GRID_SEED);

    // let mut buffer = SimdArray::<f32, 32768>::new(0.0);
    // let 

    // let time = Instant::now();
    // const NUM_RUNS: usize = 5_000_000;
    // let freq = 1. / 64.;
    // for _ in 0..NUM_RUNS {
    //     grid_3d.fbm::<Value>().frequency(freq).fill(&mut buffer);
    //     black_box(&buffer);
    // }
    // let total = time.elapsed();
    // println!(
    //     "Total: {:?}, Average Completion: {:?}",
    //     total,
    //     total / NUM_RUNS as u32
    // );

    // grid_2d
    //     .fbm::<Perlin>()
    //     .octaves(1)
    //     .frequency(1. / 32.)
    //     .seed(FBM_SEED)
    //     .into_iter()
    //     .to_grayscale_image(32, 32, "noise_images/perlin_grid_2d.png");

    let octave_list: [Octave<2>; _] = [
        Octave::new([1.0 / 480.0, 1.0 / 110.0], 1.0), // wide horizontal bands, tight vertical ripple
        Octave::new([1.0 / 290.0, 1.0 / 230.0], 0.65), // axes converge toward similar scale
        Octave::new([1.0 / 130.0, 1.0 / 310.0], 0.8), // weight bumps up, axes swap dominance
        Octave::new([1.0 / 60.0, 1.0 / 520.0], 0.35), // x sharpens fast, y pulls back out
                                                      // Octave::new([1.0 / 3.0,  1.0 / 9.0],  0.12),  // both fine, low influence to finish
    ];

    grid_2d_big
        .builder_with_octaves::<Ridged, Perlin>(&octave_list)
        .into_iter()
        .map(|x| ArchSimd::splat(0.5) * x - ArchSimd::splat(1.0))
        .to_grayscale_image(2047, 2047, "noise_images/grid_test.png");

    // grid_2d_big
    //     .sample::<Ridged, Perlin>()
    //     .octaves(2)
    //     .frequency(1. / 16.0)
    //     .seed(FBM_SEED)
    //     .seed(12)
    //     // .gain(0.8)
    //     .into_iter()
    //     // .map(|x| ArchSimd::splat(0.5) * x - ArchSimd::splat(1.0))
    //     .map(|x| ArchSimd::splat(1.0) * x - ArchSimd::splat(1.0))
    //     .to_grayscale_image(2047, 2047, "noise_images/grid_test.png");

    BatchNoise::<2, Ridged, Cellular>::builder(grid_2d_big.x_iter(), grid_2d_big.y_iter())
        .octaves(4)
        .frequency(1.0 / 512.0)
        .into_iter()
        .map(|x| ArchSimd::splat(0.5) * x - ArchSimd::splat(1.0))
        .to_grayscale_image(2047, 2047, "noise_images/batch_test.png");

    // BatchNoise::<3, Ridged, Cellular>::sample_builder(
    //     grid_3d.x_iter(),
    //     grid_3d.y_iter(),
    //     grid_3d.z_iter(),
    // )
    // .octaves(4)
    // .frequency(1.0 / 64.0)
    // .into_iter()
    // .map(|x| ArchSimd::splat(0.5) * x - ArchSimd::splat(1.0))
    // .to_grayscale_image(255, 255, "noise_images/batch_test.png");
}
