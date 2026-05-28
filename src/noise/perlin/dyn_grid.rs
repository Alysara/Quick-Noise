// DISCLAIMER: WIP Heap-based version of the algorithm. Allows runtime dimensions!

use itertools::izip;
use crate::perlin::Perlin;
use crate::perlin::constants::GRADIENTS_2D;
use crate::simd::arch_simd::{ArchMask, ArchSimd, NUM_SIMD_REG};
use crate::simd::simd_traits::*;
use crate::simd_vec;
use crate::{
    math::vec::Vec2,
    simd::{architectures::arch_impl::SimdArch, simd_vec::SimdVec},
};
use std::cmp::min;

impl Perlin {
    #[inline(always)]
    pub fn dyn_grid_2d<const INITIALIZE: bool>(
        dimensions: Vec2<usize>,
        seed: u32,
        result: &mut SimdVec<f32>,
        position: Vec2<i32>,
        frequency: Vec2<f32>,
        weight: f32,
        magnification: f32,
    ) {
        let padded_dim = dimensions + dimensions % ArchSimd::<f32>::LANES + ArchSimd::<f32>::LANES;
        let increment: Vec2<f32> = frequency * magnification;
        let block_pos: Vec2<i32> = position * dimensions.map(|x| x as i32);

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        let grid_start: Vec2<i32> = (block_pos.as_f32() * increment).floor().as_i32();
        let frac_start: Vec2<f32> =
            (block_pos.as_f32() * increment - grid_start.as_f32()).float_max(Vec2::splat(0.0));

        // Get the distances from the gradient gridpoints.
        let distances: Vec2<SimdVec<f32>> = Vec2 {
            x: SimdVec::iota_custom(padded_dim.x as usize, frac_start.x, increment.x).fract(),
            y: SimdVec::iota_custom(padded_dim.y as usize, frac_start.y, increment.y).fract(),
        };

        // Quintic lerp the distances to get the fade factor.
        let interpolations: Vec2<SimdVec<f32>> = Vec2 {
            x: distances.x.quintic_lerp(),
            y: distances.y.quintic_lerp(),
        };

        let mut grid_indices: Vec2<SimdVec<u32>> = Vec2 {
            x: SimdVec::with_capacity(dimensions.x as usize),
            y: SimdVec::with_capacity(dimensions.y as usize),
        };

        // Identify the cutoff points between frequency-based grid boundaries.
        const STEP: usize = 64 / ArchSimd::<f32>::LANES;
        for axis in 0..2 {
            let mut cur_index = 1;
            while cur_index < dimensions[axis] {
                let mut bits = 0u64;
                let num_left = dimensions[axis] - cur_index;
                let inner_cap = min(64, num_left);
                for i in (0..inner_cap).step_by(ArchSimd::<f32>::LANES) {
                    let cur = distances[axis].load_simd(cur_index + i);
                    let prev = distances[axis].load_simd(cur_index + i - 1);
                    let mask_bits = prev.simd_gt(cur).to_bits();
                    bits ^= mask_bits << i;
                }
                bits &= !0 >> (64 - inner_cap);

                while bits != 0 {
                    let index = bits.trailing_zeros();
                    // println!("bits: {:b}\nindex: {index}, grid_index: {}", bits, cur_index as u32 + index);
                    grid_indices[axis].push(cur_index as u32 + index);
                    bits ^= 1 << index;
                }

                cur_index += 64;
            }
            grid_indices[axis].push(dimensions[axis] as u32);
        }

        // println!("grid_indices: {:?}", grid_indices);

        // TODO: what are supposed to be the sizes of these grad vecs????
        // Initialize gradient vectors.
        let mut grad_tl = unsafe {
            Vec2::new(
                SimdVec::new_uninit(padded_dim.y),
                SimdVec::new_uninit(padded_dim.y),
            )
        };
        let mut grad_tr = unsafe {
            Vec2::new(
                SimdVec::new_uninit(padded_dim.y),
                SimdVec::new_uninit(padded_dim.y),
            )
        };
        let mut grad_bl = unsafe {
            Vec2::new(
                SimdVec::new_uninit(padded_dim.y),
                SimdVec::new_uninit(padded_dim.y),
            )
        };
        let mut grad_br = unsafe {
            Vec2::new(
                SimdVec::new_uninit(padded_dim.y),
                SimdVec::new_uninit(padded_dim.y),
            )
        };

        // Set the top gradients.
        Self::dyn_grid_gradients_2d(
            seed,
            &mut grad_tl,
            &mut grad_tr,
            padded_dim.y,
            grid_start.x,
            grid_start.y,
            &grid_indices.y,
            &distances.y,
        );

        // Iterate through single x chunks but full y chunks.
        let mut x_cur_index: usize = 0;
        let x_num_loops = grid_indices.x.len();
        for x_it in 0..x_num_loops {
            let x_next_index = grid_indices.x[x_it] as usize;
            let x_cur_fract = distances.x[x_cur_index];

            // Set bottom gradients.
            Self::dyn_grid_gradients_2d(
                seed,
                &mut grad_bl,
                &mut grad_br,
                padded_dim.y,
                grid_start.x + x_it as i32 + 1,
                grid_start.y,
                &grid_indices.y,
                &distances.y,
            );

            // Perform dot products on x and trilinear interpolation (with quintic fade).
            Self::dyn_grid_dotted_bilerp::<INITIALIZE>(
                dimensions,
                &grad_tl,
                &grad_tr,
                &grad_bl,
                &grad_br,
                x_cur_fract,
                increment.x,
                &interpolations,
                x_cur_index as usize,
                x_next_index as usize,
                weight,
                result,
            );

            // Reuse the top and bottom gradients.
            std::mem::swap(&mut grad_tl, &mut grad_bl);
            std::mem::swap(&mut grad_tr, &mut grad_br);

            x_cur_index = x_next_index;
        }
    }

    #[inline(always)]
    pub(super) fn dyn_grid_gradients_2d(
        seed: u32,
        left: &mut Vec2<SimdVec<f32>>,
        right: &mut Vec2<SimdVec<f32>>,
        grad_len: usize,
        x_start: i32,
        y_start: i32,
        y_grid_indices: &SimdVec<u32>,
        y_distances: &SimdVec<f32>,
    ) {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(seed);
        let x_vec = ArchSimd::splat((x_start as u32).wrapping_mul(seed));
        let mut y_vec = ArchSimd::splat((y_start as u32).wrapping_mul(seed)) + iota_vec;
        let y_vec_stride = ArchSimd::splat((ArchSimd::<f32>::LANES as u32).wrapping_mul(seed));

        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::load(&BYTE_SHUFFLE[..]);

        let prime = ArchSimd::splat(0x85ebca6b_u32);
        let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;

        // Temporary buffer to store indices for gradient values.
        let y_num_loops = y_grid_indices.len();
        // println!("y_num_loops: {y_num_loops}");
        let grad_size = (y_num_loops + 1)
            + (ArchSimd::<f32>::LANES - (y_num_loops + 1) % ArchSimd::<f32>::LANES);
        let mut grad_vec = unsafe { SimdVec::new_uninit(grad_size) };
        // println!("grad_size: {grad_size}");

        // Main vectorized bit mixing loop.
        let end_index = y_num_loops as usize + 1;
        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (x_shuf * y_shuf) >> 29;
            grad_vec.store_simd(i, indices);
            y_vec += y_vec_stride;
        }

        let mut vecs = [&mut left.x, &mut left.y, &mut right.x, &mut right.y];

        // Loop through the y chunks.
        let mut y_cur_index = 0u32;
        for y_it in 0..y_num_loops {
            // Find range of gradients to set.
            let y_next_index = y_grid_indices[y_it];
            let set_amount = y_next_index - y_cur_index;

            unsafe {
                let l = *grad_vec.get_unchecked(y_it as usize) as usize;
                let r = *grad_vec.get_unchecked(y_it as usize + 1) as usize;

                debug_assert!(l < 32);
                debug_assert!(r < 32);
                let values = [
                    GRADIENTS_2D.get_unchecked(l).x,
                    GRADIENTS_2D.get_unchecked(l).y,
                    GRADIENTS_2D.get_unchecked(r).x,
                    GRADIENTS_2D.get_unchecked(r).y,
                ];

                SimdVec::<f32>::multiset_many::<4>(
                    &mut vecs,
                    &values,
                    grad_len,
                    y_cur_index as usize,
                    set_amount as isize,
                );
            }

            y_cur_index = y_next_index;
        }

        // Compute y dot products (Better to do here since these dot products get reused and operate per element).
        // izip!(left.y.iter_mut(), y_distances.iter())
        //     .for_each(|(mut x, y)| *x = *x * y);

        left.y *= y_distances.clone();
        right.y = right.y.mul_sub(y_distances, &right.y); // equivalent to -> right.y *= y_distances - 1.0
    }

    #[inline(always)]
    pub(super) fn dyn_grid_dotted_bilerp<const INITIALIZE: bool>(
        dimensions: Vec2<usize>,
        grad_tl: &Vec2<SimdVec<f32>>,
        grad_tr: &Vec2<SimdVec<f32>>,
        grad_bl: &Vec2<SimdVec<f32>>,
        grad_br: &Vec2<SimdVec<f32>>,
        x_frac_start: f32,
        x_increment: f32,
        interpolations: &Vec2<SimdVec<f32>>,
        x_start_index: usize,
        x_end_index: usize,
        weight: f32,
        result: &mut SimdVec<f32>,
    ) {
        let weight_vec = ArchSimd::splat(weight);
        let x_weighted_increment_vec = ArchSimd::splat(x_increment * weight);
        let x_upper_increment = ArchSimd::splat(x_frac_start);
        let x_lower_increment = ArchSimd::splat(x_frac_start - 1.0);

        const NUM_BLOCKS_POSSIBLE: usize = NUM_SIMD_REG / 8;
        // let max_num_blocks = dimensions.y / ArchSimd::<f32>::LANES;
        // let num_blocks: usize = if NUM_BLOCKS_POSSIBLE < max_num_blocks { NUM_BLOCKS_POSSIBLE } else { max_num_blocks };

        for y_it in (0..dimensions.y).step_by(ArchSimd::<f32>::LANES * NUM_BLOCKS_POSSIBLE) {
            // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
            let mut base_lerps_top: [ArchSimd<f32>; NUM_BLOCKS_POSSIBLE] = Default::default();
            let mut base_lerps_dif: [ArchSimd<f32>; NUM_BLOCKS_POSSIBLE] = Default::default();
            let mut x_offset_lerps_top: [ArchSimd<f32>; NUM_BLOCKS_POSSIBLE] = Default::default();
            let mut x_offset_lerps_dif: [ArchSimd<f32>; NUM_BLOCKS_POSSIBLE] = Default::default();

            // These blocked loops will get entirely unrolled by the compiler.
            for block in 0..NUM_BLOCKS_POSSIBLE {
                unsafe {
                    // Load gradients into registers.
                    let y_lerp = interpolations
                        .y
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let x_tl = grad_tl
                        .x
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let x_tr = grad_tr
                        .x
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let x_bl = grad_bl
                        .x
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let x_br = grad_br
                        .x
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let y_tl = grad_tl
                        .y
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let y_tr = grad_tr
                        .y
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let y_bl = grad_bl
                        .y
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);
                    let y_br = grad_br
                        .y
                        .load_simd_unchecked(y_it + ArchSimd::<f32>::LANES * block);

                    // Compute base dot products.
                    let prod_sum_tl = x_tl.mul_add(x_upper_increment, y_tl);
                    let prod_sum_tr = x_tr.mul_add(x_upper_increment, y_tr);
                    let prod_sum_bl = x_bl.mul_add(x_lower_increment, y_bl);
                    let prod_sum_br = x_br.mul_add(x_lower_increment, y_br);

                    // Base interpolation.
                    base_lerps_top[block] =
                        y_lerp.mul_add(prod_sum_tr - prod_sum_tl, prod_sum_tl) * weight_vec;
                    let base_lerp_bottom =
                        y_lerp.mul_add(prod_sum_br - prod_sum_bl, prod_sum_bl) * weight_vec;
                    base_lerps_dif[block] = base_lerp_bottom - base_lerps_top[block];

                    // Offset interpolation.
                    x_offset_lerps_top[block] =
                        y_lerp.mul_add(x_tr - x_tl, x_tl) * x_weighted_increment_vec;
                    let x_offset_lerp_bottom =
                        y_lerp.mul_add(x_br - x_bl, x_bl) * x_weighted_increment_vec;
                    x_offset_lerps_dif[block] = x_offset_lerp_bottom - x_offset_lerps_top[block];
                }
            }

            let mut x_it = x_start_index;
            while x_it < x_end_index {
                if x_it + 4 > x_end_index {
                    let x_lerp = ArchSimd::splat(unsafe { *interpolations.x.get_unchecked(x_it) });
                    for block in 0..NUM_BLOCKS_POSSIBLE {
                        let index: usize =
                            x_it * dimensions.x + y_it + block * ArchSimd::<f32>::LANES;
                        let output = x_lerp.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        unsafe {
                            let val = if INITIALIZE {
                                output
                            } else {
                                output + result.load_simd_unchecked(index)
                            };
                            result.store_simd_unchecked(index, val);
                        }
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    x_it += 1;
                } else {
                    let x_lerp_1 =
                        ArchSimd::splat(unsafe { *interpolations.x.get_unchecked(x_it) });
                    let x_lerp_2 =
                        ArchSimd::splat(unsafe { *interpolations.x.get_unchecked(x_it + 1) });
                    let x_lerp_3 =
                        ArchSimd::splat(unsafe { *interpolations.x.get_unchecked(x_it + 2) });
                    let x_lerp_4 =
                        ArchSimd::splat(unsafe { *interpolations.x.get_unchecked(x_it + 3) });

                    for block in 0..NUM_BLOCKS_POSSIBLE {
                        let index: usize =
                            x_it * dimensions.x + y_it + block * ArchSimd::<f32>::LANES;
                        let output = x_lerp_1.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        unsafe {
                            let val = if INITIALIZE {
                                output
                            } else {
                                output + result.load_simd_unchecked(index)
                            };
                            result.store_simd_unchecked(index, val);
                        }
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    for block in 0..NUM_BLOCKS_POSSIBLE {
                        let index: usize = x_it * dimensions.x
                            + y_it
                            + block * ArchSimd::<f32>::LANES
                            + dimensions.y;
                        let output = x_lerp_2.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        unsafe {
                            let val = if INITIALIZE {
                                output
                            } else {
                                output + result.load_simd_unchecked(index)
                            };
                            result.store_simd_unchecked(index, val);
                        }
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    for block in 0..NUM_BLOCKS_POSSIBLE {
                        let index: usize = x_it * dimensions.x
                            + y_it
                            + block * ArchSimd::<f32>::LANES
                            + dimensions.y * 2;
                        let output = x_lerp_3.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        unsafe {
                            let val = if INITIALIZE {
                                output
                            } else {
                                output + result.load_simd_unchecked(index)
                            };
                            result.store_simd_unchecked(index, val);
                        }
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    for block in 0..NUM_BLOCKS_POSSIBLE {
                        let index: usize = x_it * dimensions.x
                            + y_it
                            + block * ArchSimd::<f32>::LANES
                            + dimensions.y * 3;
                        let output = x_lerp_4.mul_add(base_lerps_dif[block], base_lerps_top[block]);
                        unsafe {
                            let val = if INITIALIZE {
                                output
                            } else {
                                output + result.load_simd_unchecked(index)
                            };
                            result.store_simd_unchecked(index, val);
                        }
                        base_lerps_dif[block] += x_offset_lerps_dif[block];
                        base_lerps_top[block] += x_offset_lerps_top[block];
                    }
                    x_it += 4;
                }
            }
        }
    }
}
