use std::hint::black_box;

use quick_noise::emit::NoiseImageExt;
use quick_noise::simd::array_trait::Array;
use quick_noise::simd::register::Simd;
use quick_noise::simd::{Arch, StaticSimd};
use quick_noise::{Fbm, Grid, Perlin};
use quick_noise_macros::{dispatch_simd, enable_targets};

#[cfg(feature = "image")]
fn main() {
    simd_work(100);
}

#[dispatch_simd(A)]
fn simd_work(val: usize) {
    let grid = Grid::<2, A>::new(1024, 1024);

    grid.builder::<Fbm, Perlin>()
        .octaves(6)
        .into_iter()
        .to_grayscale_image(1024, 1024, "noise_images/dispatch.png");
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
