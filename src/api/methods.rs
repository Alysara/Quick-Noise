use std::ops::Index;

// use crate::api::batch::interface::BatchNoise;
use crate::api::configs::GridConfig;
// use crate::api::batch::interface::BatchNoise;
use crate::api::grid::interface::{GridNoiseImpl, GridNoiseParams};
use crate::math::random::Random;
use crate::math::vec::{ArithmeticVec, BasicVec, Vec2, Vec3, VecHorz};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::{EmptyIter, GridNoise, ZeroIter};
//
// pub enum NoiseDim {
//     Dim2,
//     Dim3,
// }
//
// #[derive(Default, Clone)]
// pub struct Dim2;
// #[derive(Default, Clone)]
// pub struct Dim3;
//
// pub trait NoiseDimension: Default {
//     type FVec: ArithmeticVec<f32> + Into<(ArchSimd<f32>, ArchSimd<f32>, ArchSimd<f32>)>;
//     type IVec: ArithmeticVec<i32>;
//     type USizeVec: ArithmeticVec<usize>;
//     type Vec<T: Copy>: Copy + BasicVec<T>;
//
//     const DIM: NoiseDim;
//
//     fn grid<T: GridNoiseImpl, const INITIALIZE: bool>(
//         parameters: GridNoiseParams<Self>,
//         result: &mut [f32],
//     );
//
//     fn batch<T: BatchNoise, const N: usize>(
//         seed: u32,
//         x_input: ArchSimd<f32>,
//         y_input: ArchSimd<f32>,
//         z_input: ArchSimd<f32>,
//         x_freq: ArchSimd<f32>,
//         y_freq: ArchSimd<f32>,
//         z_freq: ArchSimd<f32>,
//     ) -> ArchSimd<f32>;
//
//     fn octave_seed(vec: Self::FVec, seed: u64) -> u32;
//
//     fn get_iters<const X: usize, const Y: usize, const Z: usize, const N: usize>(
//         config: GridConfig<Self>,
//     ) -> (
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//     );
// }
//
// impl NoiseDimension for Dim2 {
//     type FVec = Vec2<f32>;
//     type IVec = Vec2<i32>;
//     type USizeVec = Vec2<usize>;
//     type Vec<T: Copy> = Vec2<T>;
//     const DIM: NoiseDim = NoiseDim::Dim2;
//
//     fn grid<T: GridNoiseImpl, const INITIALIZE: bool>(
//         parameters: GridNoiseParams<Self>,
//         result: &mut [f32],
//     ) {
//         T::grid_2d::<INITIALIZE>(parameters, result);
//     }
//
//     #[inline(always)]
//     fn batch<T: BatchNoise, const N: usize>(
//         seed: u32,
//         x_input: ArchSimd<f32>,
//         y_input: ArchSimd<f32>,
//         _z_input: ArchSimd<f32>,
//         x_freq: ArchSimd<f32>,
//         y_freq: ArchSimd<f32>,
//         _z_freq: ArchSimd<f32>,
//     ) -> ArchSimd<f32> {
//         T::batch_2d(seed, x_input, y_input, x_freq, y_freq)
//     }
//
//     fn octave_seed(vec: Self::FVec, seed: u64) -> u32 {
//         Random::static_mix_u64_pair(
//             seed.wrapping_mul(vec.x.to_bits() as u64),
//             seed.wrapping_mul(vec.y.to_bits() as u64),
//         ) as u32
//     }
//
//     fn get_iters<const X: usize, const Y: usize, const Z: usize, const N: usize>(
//         config: GridConfig<Self>,
//     ) -> (
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//     ) {
//         let grid = GridNoise::<Dim2>::from_config(config);
//         (grid.x_iter(), grid.y_iter(), ZeroIter::<N>::default())
//     }
// }
//
// impl NoiseDimension for Dim3 {
//     type FVec = Vec3<f32>;
//     type IVec = Vec3<i32>;
//     type USizeVec = Vec3<usize>;
//     type Vec<T: Copy> = Vec3<T>;
//     const DIM: NoiseDim = NoiseDim::Dim3;
//
//     fn grid<T: GridNoiseImpl, const INITIALIZE: bool>(
//         seed: u32,
//         result: &mut [f32],
//         dimensions: Self::USizeVec,
//         position: Self::IVec,
//         frequency: Self::FVec,
//         weight: f32,
//         magnification: f32,
//         tiling: Self::Vec<Option<u32>>,
//     ) {
//         T::grid_3d::<INITIALIZE>(
//             seed,
//             result,
//             dimensions,
//             position,
//             frequency,
//             weight,
//             magnification,
//             tiling,
//         );
//     }
//
//     #[inline(always)]
//     fn batch<T: BatchNoise, const N: usize>(
//         seed: u32,
//         x_input: ArchSimd<f32>,
//         y_input: ArchSimd<f32>,
//         z_input: ArchSimd<f32>,
//         x_freq: ArchSimd<f32>,
//         y_freq: ArchSimd<f32>,
//         z_freq: ArchSimd<f32>,
//     ) -> ArchSimd<f32> {
//         T::batch_3d(seed, x_input, y_input, z_input, x_freq, y_freq, z_freq)
//     }
//
//     fn octave_seed(vec: Self::FVec, seed: u64) -> u32 {
//         Random::mix_u64_triple(
//             seed.wrapping_mul(vec.x.to_bits() as u64),
//             seed.wrapping_mul(vec.y.to_bits() as u64),
//             seed.wrapping_mul(vec.z.to_bits() as u64),
//         ) as u32
//     }
//
//     fn get_iters<const X: usize, const Y: usize, const Z: usize, const N: usize>(
//         config: GridConfig<Self>,
//     ) -> (
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//     ) {
//         let grid = GridNoise::<Dim3>::from_config(config);
//         (grid.x_iter(), grid.y_iter(), grid.z_iter())
//     }
// }

// pub type Octave2D = Octave<Dim2>;
// pub type Octave3D = Octave<Dim3>;
#[derive(Copy, Clone)]
pub struct Octave<const D: usize> {
    pub weight: f32,
    pub frequency: [f32; D],
}

impl<const D: usize> Octave<D> {
    pub fn new(frequency: [f32; D], weight: f32) -> Self {
        Self { frequency, weight }
    }

    pub fn splat(frequency: f32, weight: f32) -> Self {
        let frequency = [frequency; D];
        Self { frequency, weight }
    }
}

// pub enum NoiseMethod {
//     Perlin = 0,
//     Value = 1,
//     Simplex = 2,
//     Cellular = 3,
// }
//
// /// Necessary converter due to limitation of types for const generics.
// impl NoiseMethod {
//     pub const PERLIN_U8: u8 = NoiseMethod::Perlin as u8;
//     pub const VALUE_U8: u8 = NoiseMethod::Value as u8;
//     pub const SIMPLEX_U8: u8 = NoiseMethod::Simplex as u8;
//     pub const CELLULAR_U8: u8 = NoiseMethod::Cellular as u8;
//
//     #[inline(always)]
//     pub const fn from_u8_const(val: u8) -> Self {
//         match val {
//             0 => NoiseMethod::Perlin,
//             1 => NoiseMethod::Value,
//             2 => NoiseMethod::Simplex,
//             3 => NoiseMethod::Cellular,
//             _ => panic!("Invalid NoiseMethod enum value!"),
//         }
//     }
//
//     #[inline(always)]
//     pub fn batch_2d(
//         &self,
//         seed: u32,
//         x: ArchSimd<f32>,
//         y: ArchSimd<f32>,
//         x_freq: ArchSimd<f32>,
//         y_freq: ArchSimd<f32>,
//     ) -> ArchSimd<f32> {
//         match &self {
//             NoiseMethod::Perlin => Perlin::batch_2d(seed, x, y, x_freq, y_freq),
//             NoiseMethod::Value => Value::batch_2d(seed, x, y, x_freq, y_freq),
//             NoiseMethod::Simplex => Simplex::batch_2d(seed, x, y, x_freq, y_freq),
//             NoiseMethod::Cellular => Cellular::batch_2d(seed, x, y, x_freq, y_freq),
//         }
//     }
//
//     #[inline(always)]
//     pub fn batch_3d(
//         &self,
//         seed: u32,
//         x: ArchSimd<f32>,
//         y: ArchSimd<f32>,
//         z: ArchSimd<f32>,
//         x_freq: ArchSimd<f32>,
//         y_freq: ArchSimd<f32>,
//         z_freq: ArchSimd<f32>,
//     ) -> ArchSimd<f32> {
//         match &self {
//             NoiseMethod::Perlin => Perlin::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
//             NoiseMethod::Value => Value::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
//             NoiseMethod::Simplex => Simplex::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
//             NoiseMethod::Cellular => Cellular::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
//         }
//     }
// 
