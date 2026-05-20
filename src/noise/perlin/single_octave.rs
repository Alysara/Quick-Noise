use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3};
use crate::noise::perlin::Perlin;
use crate::noise::perlin::constants::*;
use crate::noise::perlin::containers::*;
use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::SimdMaskToBits;
use crate::simd::simd_traits::SimdPartialOrd;

impl Perlin {
    #[inline(never)]
    pub(super) fn uniform_grid_octave_2d<const INITIALIZE: bool>(
        random_gen: &mut Random,
        result: &mut PerlinMap,
        pos: Vec2<i32>,
        octave: &Octave2D,
        weight_coef: f32,
        channel_seed: u64,
        scale_multiplier: f32,
    ) {
        let increment: Vec2<f32> = 1.0 / (octave.scale * scale_multiplier);
        let block_pos: Vec2<i32> = pos * 32;
        let weight: f32 = octave.weight * weight_coef;

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        let grid_start: Vec2<i32> = (block_pos.as_f32() * increment).floor().as_i32();
        let frac_start: Vec2<f32> =
            (block_pos.as_f32() * increment - grid_start.as_f32()).float_max(Vec2::splat(0.0));

        // Get the distances from the gradient gridpoints.
        let distances: PerlinVecPair = PerlinVecPair {
            x: PerlinVec::iota_custom(frac_start.x, increment.x).fract(),
            y: PerlinVec::iota_custom(frac_start.y, increment.y).fract(),
        };

        // Quintic lerp the distances to get the fade factor.
        let interpolations: PerlinVecPair = PerlinVecPair {
            x: distances.x.quintic_lerp(),
            y: distances.y.quintic_lerp(),
        };

        // Set the channel for random number generation based on the octave scale and selected channel.
        // Note: Octave offset does not currently work.
        random_gen.set_channel(channel_seed ^ octave.scale.sum() as u64);

        let mut grid_indices: Vec2<u32> = Vec2::splat(0);

        for axis in 0..2 {
            for i in (1..ROW_SIZE).step_by(ArchSimd::<f32>::LANES) {
                let cur = distances[axis].load_simd(i);
                let prev = distances[axis].load_simd(i - 1);
                let bits = prev.simd_gt(cur).to_bits() as u32;
                grid_indices[axis] ^= bits << i;
            }
        }

        // Initialize gradient vectors.
        let mut d_vecs: PerlinContainer2D = unsafe { PerlinContainer2D::new_uninit() };

        // Set the top gradients.
        let (tl, tr) = d_vecs.tl_tr_mut();
        Self::set_uniform_grid_gradients_2d(
            random_gen,
            tl,
            tr,
            grid_start.x,
            grid_start.y,
            grid_indices.y,
            &distances.y,
        );

        // Iterate through single x chunks but full y chunks.
        let mut x_cur_index: usize = 0;
        let x_num_loops = grid_indices.x.count_ones() + 1;
        for x_it in 0..x_num_loops {
            let x_cur_fract = distances.x[x_cur_index];
            let x_next_index: usize = grid_indices.x.trailing_zeros() as usize;

            // Set bottom gradients.
            let (bl, br) = d_vecs.bl_br_mut();
            Self::set_uniform_grid_gradients_2d(
                random_gen,
                bl,
                br,
                grid_start.x + x_it as i32 + 1,
                grid_start.y,
                grid_indices.y,
                &distances.y,
            );

            // Perform dot products on x and trilinear interpolation (with quintic fade).
            Self::uniform_grid_interpolate_2d::<INITIALIZE>(
                &d_vecs,
                x_cur_fract,
                increment.x,
                &interpolations,
                x_cur_index as usize,
                x_next_index as usize,
                weight,
                result,
            );

            // Reuse the top and bottom gradients.
            d_vecs.swap_top_bottom();

            grid_indices.x ^= 1 << x_next_index;
            x_cur_index = x_next_index;
        }
    }

    #[inline(never)]
    pub(super) fn uniform_grid_octave_3d<const INITIALIZE: bool>(
        random_gen: &mut Random,
        result: &mut PerlinVol,
        pos: Vec3<i32>,
        octave: &Octave3D,
        weight_coef: f32,
        channel_seed: u64,
        scale_multiplier: f32,
    ) {
        let increment: Vec3<f32> = 1.0 / (octave.scale * scale_multiplier);
        let block_pos: Vec3<i32> = pos * 32;
        let weight: f32 = octave.weight * weight_coef;

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        let grid_start: Vec3<i32> = (block_pos.as_f32() * increment).floor().as_i32();
        let frac_start: Vec3<f32> =
            (block_pos.as_f32() * increment - grid_start.as_f32()).float_max(Vec3::splat(0.0));

        // Get the distances from the gradient gridpoints.
        let distances: PerlinVecTriple = PerlinVecTriple {
            x: PerlinVec::iota_custom(frac_start.x, increment.x).fract(),
            y: PerlinVec::iota_custom(frac_start.y, increment.y).fract(),
            z: PerlinVec::iota_custom(frac_start.z, increment.z).fract(),
        };

        // Quintic lerp the distances to get the fade factor.
        let interpolations: PerlinVecTriple = PerlinVecTriple {
            x: distances.x.quintic_lerp(),
            y: distances.y.quintic_lerp(),
            z: distances.z.quintic_lerp(),
        };

        // Set the channel for random number generation based on the octave scale and selected channel.
        // Note: Octave offset does not currently work.
        random_gen.set_channel(channel_seed ^ octave.scale.sum() as u64);

        // Identify the number of loops to iterate through (better compiler optimization when known).
        let mut grid_indices: Vec3<u32> = Vec3::splat(0);

        for axis in 0..3 {
            for i in (1..ROW_SIZE).step_by(ArchSimd::<f32>::LANES) {
                let cur = distances[axis].load_simd(i);
                let prev = distances[axis].load_simd(i - 1);
                let bits = prev.simd_gt(cur).to_bits() as u32;
                grid_indices[axis] ^= bits << i;
            }
        }

        // Initialize gradient vectors.
        let mut d_vecs: PerlinContainer3D = unsafe { PerlinContainer3D::new_uninit() };

        // Iterate through single x chunks but full y chunks.
        let mut x_cur_index: usize = 0;
        let x_num_loops = grid_indices.x.count_ones() + 1;
        for x_it in 0..x_num_loops {
            let x_cur_fract = distances.x[x_cur_index];
            let x_next_index: usize = grid_indices.x.trailing_zeros() as usize;

            // Set the top gradients.
            let (tlf, trf, tlb, trb) = d_vecs.tlf_trf_tlb_trb_mut();
            Self::set_uniform_grid_gradients_3d(
                random_gen,
                tlf,
                trf,
                tlb,
                trb,
                grid_start.x + x_it as i32,
                grid_start.y,
                grid_start.z,
                grid_indices.z,
                &distances.z,
            );

            // Iterate through single x chunks but full y chunks.
            let mut y_cur_index: usize = 0;
            let mut y_indices = grid_indices.y;
            let y_num_loops = grid_indices.y.count_ones() + 1;
            for y_it in 0..y_num_loops {
                let y_cur_fract = distances.y[y_cur_index];
                debug_assert!(y_cur_fract >= 0.0 && y_cur_fract.is_finite());

                // Set the bottom gradients.
                let (blf, brf, blb, brb) = d_vecs.blf_brf_blb_brb_mut();
                Self::set_uniform_grid_gradients_3d(
                    random_gen,
                    blf,
                    brf,
                    blb,
                    brb,
                    grid_start.x + x_it as i32,
                    grid_start.y + y_it as i32 + 1,
                    grid_start.z,
                    grid_indices.z,
                    &distances.z,
                );

                // Identify the current range of y gradients.
                let y_next_index: usize = y_indices.trailing_zeros() as usize;

                // Perform dot products on x,y and trilinear interpolation (with quintic fade).
                Self::uniform_grid_interpolate_3d::<INITIALIZE>(
                    &d_vecs,
                    x_cur_fract,
                    y_cur_fract,
                    increment.x,
                    increment.y,
                    &interpolations,
                    x_cur_index as usize,
                    y_cur_index as usize,
                    x_next_index as usize,
                    y_next_index as usize,
                    weight,
                    result,
                );

                // Reuse the top and bottom gradients.
                d_vecs.swap_top_bottom();

                y_indices ^= 1 << y_next_index;
                y_cur_index = y_next_index;
            }
            grid_indices.x ^= 1 << x_next_index;
            x_cur_index = x_next_index;
        }
    }
}
