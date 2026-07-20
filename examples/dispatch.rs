use quick_noise::simd::StaticSimd;
use quick_noise::simd::dispatch::*;
use quick_noise::simd::register::Simd;

fn main() {
    let arch = detect_architecture();
    dispatch!(arch, simd_work());
}

fn simd_work<F: Arch>() {
    let array: [f32; 1024] = std::array::from_fn(|i| i as f32);

    let dyn_simd = Simd::<f32, F>::from_slice(array.as_slice());
    println!("dynamic dispatch: {:?}", dyn_simd);

    let simd = StaticSimd::<f32>::from_slice(array.as_slice());
    println!("static dispatch: {:?}", simd);
}
