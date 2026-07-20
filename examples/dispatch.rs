use quick_noise::Fbm;
use quick_noise::Grid;
use quick_noise::Perlin;
use quick_noise::simd::StaticSimd;
use quick_noise::simd::dispatch::*;
use quick_noise::simd::register::Simd;
use quick_noise::emit::NoiseImageExt;

fn main() {
    dispatch!(simd_work());
}

fn simd_work<A: Arch>() {
    let array: [f32; 1024] = std::array::from_fn(|i| i as f32);

    let dyn_simd = Simd::<f32, A>::from_slice(array.as_slice());
    println!("dynamic dispatch: {:?}", dyn_simd);

    let simd = StaticSimd::<f32>::from_slice(array.as_slice());
    println!("static dispatch: {:?}", simd);

    let grid = Grid::<2>::new(1024, 1024);
    
    grid.builder_for::<Fbm, Perlin, A>()
        .frequency(1.0 / 32.0)
        .octaves(6)
        .into_iter()
        .to_grayscale_image(1024, 1024, "noise_images/dispatch.png");
}
