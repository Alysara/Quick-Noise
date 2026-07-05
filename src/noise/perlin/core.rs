use std::mem::MaybeUninit;
use std::time::Instant;

use crate::api::grid::interface::GridNoiseImpl;
use crate::math::vec::{Vec2, Vec3};
use crate::perlin::dyn_grid_2d::grid_2d;
use crate::perlin::{PerlinGridNoise2D, PerlinGridNoise3D};
use crate::simd::simd_array::SimdArray;

#[derive(Default, Copy, Clone)]
pub struct Perlin {}

impl GridNoiseImpl for Perlin {
    fn grid_2d<const INITIALIZE: bool>(
        seed: u32,
        result: &mut [f32],
        dimensions: Vec2<usize>,
        position: Vec2<i32>,
        frequency: Vec2<f32>,
        weight: f32,
        magnification: f32,
        tiling: Vec2<Option<u32>>,
    ) {
        // let result: &mut SimdArray<f32, 1024> = unsafe { std::mem::transmute(result.as_mut_ptr())};
        // PerlinGridNoise2D::<32, 32, 1024>::grid_2d::<INITIALIZE>(
        //     seed,
        //     result,
        //     position,
        //     frequency,
        //     weight,
        //     magnification,
        //     tiling,
        // );

        grid_2d::<INITIALIZE>(
            dimensions,
            result,
            seed,
            position,
            frequency,
            weight,
            magnification,
            tiling,
        );
    }

    fn grid_3d<const INITIALIZE: bool>(
        seed: u32,
        result: &mut [f32],
        dimensions: Vec3<usize>,
        position: Vec3<i32>,
        frequency: Vec3<f32>,
        weight: f32,
        magnification: f32,
        tiling: Vec3<Option<u32>>,
    ) {
        // PerlinGridNoise3D::<X, Y, Z, N>::grid_3d::<INITIALIZE>(
        //     seed,
        //     result,
        //     position,
        //     frequency,
        //     weight,
        //     magnification,
        //     tiling,
        // );
    }
}

// impl Perlin {
//     pub fn uniform_grid_2d(
//         random_gen: &mut Random,
//         result: &mut PerlinMap,
//         pos: Vec2<i32>,
//         octaves: u32,
//         scale: f32,
//         amplitude: f32,
//         lacunarity: f32,
//         persistence: f32,
//         channel: i32,
//         octave_offset: f32,
//     ) {
//         // Get the channel seed for gradient generation.
//         let channel_seed: u64 = Random::static_mix_u64(channel as u64);

//         // Identify weight sum for normalization to [-ampltiude, amplitude]
//         let mut weight_sum = amplitude;
//         let mut cur_weight = amplitude;
//         for _ in 1..octaves {
//             cur_weight *= persistence;
//             weight_sum += cur_weight;
//         }
//         let weight_coef = 1.0 / weight_sum;

//         let lacunarity_inv = 1.0 / lacunarity;

//         let mut cur_octave = Octave2D::splat(scale, 1.0);

//         // Add each noise pass to result. Slight performance boost for initializing on the first pass.
//         Self::uniform_grid_octave_2d::<true>(
//             random_gen,
//             result,
//             pos,
//             &cur_octave,
//             weight_coef,
//             channel_seed,
//             octave_offset,
//         );
//         for _ in 1..octaves {
//             cur_octave.scale *= lacunarity_inv;
//             cur_octave.weight *= persistence;

//             Self::uniform_grid_octave_2d::<false>(
//                 random_gen,
//                 result,
//                 pos,
//                 &cur_octave,
//                 weight_coef,
//                 channel_seed,
//                 octave_offset,
//             );
//         }
//     }

//     pub fn uniform_grid_2d_octaves(
//         random_gen: &mut Random,
//         result: &mut PerlinMap,
//         pos: Vec2<i32>,
//         octaves: impl IntoIterator<Item = impl Into<Octave2D>>,
//         amplitude: f32,
//         channel: i32,
//         octave_offset: f32,
//     ) {
//         // Get the channel seed for gradient generation.
//         let octaves_vec: Vec<Octave2D> = octaves.into_iter().map(Into::into).collect();
//         let channel_seed: u64 = Random::static_mix_u64(channel as u64);

//         // Identify weight sum for normalization to [-ampltiude, amplitude]
//         let mut weight_sum = 0.0;
//         for octave in &octaves_vec {
//             weight_sum += octave.weight;
//         }
//         let weight_coef = amplitude / weight_sum;

//         // Add each noise pass to result. Slight performance boost for initialize on the first pass.
//         Self::uniform_grid_octave_2d::<true>(
//             random_gen,
//             result,
//             pos,
//             &octaves_vec[0],
//             weight_coef,
//             channel_seed,
//             octave_offset,
//         );
//         for i in 1..octaves_vec.len() {
//             Self::uniform_grid_octave_2d::<false>(
//                 random_gen,
//                 result,
//                 pos,
//                 &octaves_vec[i],
//                 weight_coef,
//                 channel_seed,
//                 octave_offset,
//             );
//         }
//     }

//     pub fn uniform_grid_3d(
//         random_gen: &mut Random,
//         result: &mut PerlinVol,
//         pos: Vec3<i32>,
//         octaves: u32,
//         scale: f32,
//         amplitude: f32,
//         lacunarity: f32,
//         persistence: f32,
//         channel: i32,
//         octave_offset: f32,
//     ) {
//         // Get the channel seed for gradient generation.
//         let channel_seed: u64 = Random::static_mix_u64(channel as u64);

//         // Identify weight sum for normalization to [-ampltiude, amplitude]
//         let mut weight_sum = amplitude;
//         let mut cur_weight = amplitude;
//         for _ in 1..octaves {
//             cur_weight *= persistence;
//             weight_sum += cur_weight;
//         }
//         let weight_coef = 1.0 / weight_sum;

//         let lacunarity_inv = 1.0 / lacunarity;

//         let mut cur_octave = Octave3D::splat(scale, 1.0);

//         // Add each noise pass to result. Slight performance boost for initializing on the first pass.
//         Self::uniform_grid_octave_3d::<true>(
//             random_gen,
//             result,
//             pos,
//             &cur_octave,
//             weight_coef,
//             channel_seed,
//             octave_offset,
//         );
//         for _ in 1..octaves {
//             cur_octave.scale *= lacunarity_inv;
//             cur_octave.weight *= persistence;

//             Self::uniform_grid_octave_3d::<false>(
//                 random_gen,
//                 result,
//                 pos,
//                 &cur_octave,
//                 weight_coef,
//                 channel_seed,
//                 octave_offset,
//             );
//         }
//     }

//     pub fn uniform_grid_3d_octaves(
//         random_gen: &mut Random,
//         result: &mut PerlinVol,
//         pos: Vec3<i32>,
//         octaves: impl IntoIterator<Item = impl Into<Octave3D>>,
//         amplitude: f32,
//         channel: i32,
//         octave_offset: f32,
//     ) {
//         // Get the channel seed for gradient generation.
//         let octaves_vec: Vec<Octave3D> = octaves.into_iter().map(Into::into).collect();
//         let channel_seed: u64 = Random::static_mix_u64(channel as u64);

//         // Identify weight sum for normalization to [-ampltiude, amplitude]
//         let mut weight_sum = 0.0;
//         for octave in &octaves_vec {
//             weight_sum += octave.weight;
//         }
//         let weight_coef = amplitude / weight_sum;

//         // Add each noise pass to result. Slight performance boost for initialize on the first pass.
//         Self::uniform_grid_octave_3d::<true>(
//             random_gen,
//             result,
//             pos,
//             &octaves_vec[0],
//             weight_coef,
//             channel_seed,
//             octave_offset,
//         );
//         for i in 1..octaves_vec.len() {
//             Self::uniform_grid_octave_3d::<false>(
//                 random_gen,
//                 result,
//                 pos,
//                 &octaves_vec[i],
//                 weight_coef,
//                 channel_seed,
//                 octave_offset,
//             );
//         }
//     }
// }
