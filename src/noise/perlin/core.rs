use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3};
use crate::noise::perlin::constants::*;
use crate::noise::perlin::containers::*;

pub struct Perlin {}

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
