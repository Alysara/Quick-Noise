use crate::grid_helpers::grid_fill_indices;
use crate::math::vec::Vec2;
use crate::noise::perlin::constants::*;
use crate::noise::perlin::containers::*;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::simd::simd_array::{SimdArray, TailInfo};
use crate::simd::simd_traits::*;

// ————————————————————————————————————————————————————————————————
// ————— 2D Perlin Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct PerlinGridNoise2D<const A: usize, const X: usize, const N: usize> {}

const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
const LANES: usize = ArchSimd::<f32>::LANES;
const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

impl<const X: usize, const Y: usize, const N: usize> PerlinGridNoise2D<X, Y, N> {
    const HAS_SIMD_TAIL: bool = SimdArray::<f32, X>::HAS_TAIL;
    const HAS_BLOCK_HEAD: bool = X >= BLOCK_LANES;
    const HAS_BLOCK_TAIL: bool = (X % BLOCK_LANES) > 0;

    const BLOCK_TAIL_SIZE: usize = (X % BLOCK_LANES + LANES - 1) / LANES;
    const SIMD_TAIL_SIZE: usize = SimdArray::<f32, X>::TAIL_SIZE;

    const BLOCK_TAIL_START: usize = (X / BLOCK_LANES) * BLOCK_LANES;
    const SIMD_TAIL_START: usize = SimdArray::<f32, X>::TAIL_START;

    #[inline(never)]
    pub fn grid_2d<const INITIALIZE: bool>(
        seed: u32,
        result: &mut SimdArray<f32, N>,
        position: Vec2<i32>,
        frequency: Vec2<f32>,
        weight: f32,
        magnification: f32,
    ) {
        let increment: Vec2<f32> = frequency * magnification;
        let block_pos: Vec2<i32> = position * Vec2::new(Y as i32, X as i32);

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        let grid_start: Vec2<i32> = (block_pos.as_f32() * increment).floor().as_i32();
        let frac_start: Vec2<f32> =
            (block_pos.as_f32() * increment - grid_start.as_f32()).float_max(Vec2::splat(0.0));

        // Get the distances from the gradient gridpoints.
        let x_distances = SimdArray::<f32, X>::iota_custom(frac_start.x, increment.x).fract();
        let y_distances = SimdArray::<f32, Y>::iota_custom(frac_start.y, increment.y).fract();

        // Quintic lerp the distances to get the fade factor.
        let x_lerp = x_distances.quintic_lerp();
        let y_lerp = y_distances.quintic_lerp();

        let mut x_grid_indices = unsafe { SimdArray::<u32, X>::new_uninit() };
        let mut y_grid_indices = unsafe { SimdArray::<u32, Y>::new_uninit() };
        let mut num_loops: Vec2<usize> = Vec2::splat(0);

        // Identify the cutoff points between frequency-based grid boundaries .
        grid_fill_indices(&mut x_grid_indices, &x_distances, &mut num_loops.x);
        grid_fill_indices(&mut y_grid_indices, &y_distances, &mut num_loops.y);

        // Initialize gradient vectors.
        let mut d_vecs: PerlinContainer2D<X> = unsafe { PerlinContainer2D::new_uninit() };

        // Set the top gradients.
        let (tl, tr) = d_vecs.tl_tr_mut();
        Self::grid_gradients_2d(
            seed,
            tl,
            tr,
            grid_start.x,
            grid_start.y,
            &x_grid_indices,
            num_loops.x,
            &x_distances,
        );

        // Iterate through single x chunks but full y chunks.
        let mut y_cur_index = 0;
        for y_it in 0..num_loops.y {
            let y_cur_fract = unsafe { y_distances.get_unchecked(y_cur_index) };
            let y_next_index = unsafe { y_grid_indices.get_unchecked(y_it) as usize };

            // Set bottom gradients.
            let (bl, br) = d_vecs.bl_br_mut();
            Self::grid_gradients_2d(
                seed,
                bl,
                br,
                grid_start.x,
                grid_start.y + y_it as i32 + 1,
                &x_grid_indices,
                num_loops.x,
                &x_distances,
            );

            // Perform dot products on x and trilinear interpolation (with quintic fade).
            Self::grid_dotted_bilerp::<INITIALIZE>(
                &d_vecs,
                y_cur_fract,
                increment.y,
                &x_lerp,
                &y_lerp,
                y_cur_index as usize,
                y_next_index as usize,
                weight,
                result,
            );

            // Reuse the top and bottom gradients.
            d_vecs.swap_top_bottom();

            y_cur_index = y_next_index;
        }
    }

    #[inline(always)]
    pub(super) fn grid_gradients_2d(
        seed: u32,
        left: &mut Vec2<SimdArray<f32, X>>,
        right: &mut Vec2<SimdArray<f32, X>>,
        x_start: i32,
        y_start: i32,
        x_grid_indices: &SimdArray<u32, X>,
        x_num_loops: usize,
        x_distances: &SimdArray<f32, X>,
    ) {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(seed);
        let y_vec = ArchSimd::splat((y_start as u32).wrapping_mul(seed));
        let mut x_vec = ArchSimd::splat((x_start as u32).wrapping_mul(seed)) + iota_vec;
        let x_vec_stride = ArchSimd::splat((ArchSimd::<f32>::LANES as u32).wrapping_mul(seed));

        const BYTE_SHUFFLE: [u8; 64] = [
            3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
            10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2,
            1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13,
        ];

        let shuffle_indices = ArchSimd::<u8>::load(&BYTE_SHUFFLE[..]);

        let prime = ArchSimd::splat(0x85ebca6b_u32);
        let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;

        // Temporary buffer to store indices for gradient values.
        let mut grad_array = unsafe { SimdArray::<u32, X>::new_uninit() };

        // Main vectorized bit mixing loop.
        let end_index = x_num_loops as usize + 1;
        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe { grad_array.store_simd_tail_checked(i, indices) };
            x_vec += x_vec_stride;
        }

        let mut arrays = [&mut left.y, &mut left.x, &mut right.y, &mut right.x];

        // Loop through the y chunks.
        let mut x_cur_index = 0;
        for x_it in 0..x_num_loops {
            let x_next_index = x_grid_indices[x_it];

            // Find range of gradients to set.
            let set_amount = x_next_index - x_cur_index;

            unsafe {
                let l = grad_array.get_unchecked(x_it as usize) as usize;
                let r = grad_array.get_unchecked(x_it as usize + 1) as usize;

                debug_assert!(l < 32);
                debug_assert!(r < 32);
                let values = [
                    GRADIENTS_2D.get_unchecked(l).y,
                    GRADIENTS_2D.get_unchecked(l).x,
                    GRADIENTS_2D.get_unchecked(r).y,
                    GRADIENTS_2D.get_unchecked(r).x,
                ];

                SimdArray::multiset_many::<4>(
                    &mut arrays,
                    &values,
                    x_cur_index as usize,
                    set_amount as isize,
                );
            }

            x_cur_index = x_next_index;
        }

        // Compute y dot products (Better to do here since these dot products get reused and operate per element).
        left.x *= *x_distances;
        right.x = right.x.mul_sub(*x_distances, right.x); // equivalent to -> right.x *= x_distances - 1.0
    }

    #[inline(always)]
    pub(super) fn grid_dotted_bilerp<const INITIALIZE: bool>(
        gradients: &PerlinContainer2D<X>,
        y_frac_start: f32,
        y_increment: f32,
        x_lerp_array: &SimdArray<f32, X>,
        y_lerp_array: &SimdArray<f32, Y>,
        y_start_index: usize,
        y_end_index: usize,
        weight: f32,
        result: &mut SimdArray<f32, N>,
    ) {
        let weight_vec = ArchSimd::splat(weight);
        let y_weighted_increment = ArchSimd::splat(y_increment * weight);
        let y_upper_increment = ArchSimd::splat(y_frac_start);
        let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);

        // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
        let mut base_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
        let mut base_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
        let mut y_offset_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
        let mut y_offset_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

        if Self::HAS_BLOCK_HEAD {
            Self::grid_dotted_bilerp_helper::<INITIALIZE, false>(
                gradients,
                &mut base_dif,
                &mut base_top,
                &mut y_offset_dif,
                &mut y_offset_top,
                x_lerp_array,
                y_lerp_array,
                y_upper_increment,
                y_lower_increment,
                y_start_index,
                y_end_index,
                weight_vec,
                y_weighted_increment,
                result,
            );
        }

        if Self::HAS_BLOCK_TAIL {
            Self::grid_dotted_bilerp_helper::<INITIALIZE, true>(
                gradients,
                &mut base_dif,
                &mut base_top,
                &mut y_offset_dif,
                &mut y_offset_top,
                x_lerp_array,
                y_lerp_array,
                y_upper_increment,
                y_lower_increment,
                y_start_index,
                y_end_index,
                weight_vec,
                y_weighted_increment,
                result,
            );
        }
    }

    #[inline(always)]
    fn grid_dotted_bilerp_helper<const INITIALIZE: bool, const IS_TAIL: bool>(
        gradients: &PerlinContainer2D<X>,
        base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
        base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
        y_offset_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
        y_offset_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
        x_lerp_array: &SimdArray<f32, X>,
        y_lerp_array: &SimdArray<f32, Y>,
        y_upper_increment: ArchSimd<f32>,
        y_lower_increment: ArchSimd<f32>,
        y_start_index: usize,
        y_end_index: usize,
        weight_vec: ArchSimd<f32>,
        y_weighted_increment: ArchSimd<f32>,
        result: &mut SimdArray<f32, N>,
    ) {
        let range = if IS_TAIL {
            Self::BLOCK_TAIL_START..X
        } else {
            0..Self::BLOCK_TAIL_START
        };

        let num_blocks = if IS_TAIL {
            Self::BLOCK_TAIL_SIZE
        } else {
            NUM_BLOCKS
        };

        for x_it in range.step_by(BLOCK_LANES) {
            // These blocked loops will get entirely unrolled by the compiler.
            for block in 0..num_blocks {
                // Load gradients into registers.
                let index = x_it + LANES * block;
                let x_lerp = x_lerp_array.load_simd_chunked::<IS_TAIL>(index);
                let x_tl = gradients.tl().x.load_simd_chunked::<IS_TAIL>(index);
                let x_tr = gradients.tr().x.load_simd_chunked::<IS_TAIL>(index);
                let x_bl = gradients.bl().x.load_simd_chunked::<IS_TAIL>(index);
                let x_br = gradients.br().x.load_simd_chunked::<IS_TAIL>(index);
                let y_tl = gradients.tl().y.load_simd_chunked::<IS_TAIL>(index);
                let y_tr = gradients.tr().y.load_simd_chunked::<IS_TAIL>(index);
                let y_bl = gradients.bl().y.load_simd_chunked::<IS_TAIL>(index);
                let y_br = gradients.br().y.load_simd_chunked::<IS_TAIL>(index);

                // Compute base dot products.
                let prod_sum_tl = y_tl.mul_add(y_upper_increment, x_tl);
                let prod_sum_tr = y_tr.mul_add(y_upper_increment, x_tr);
                let prod_sum_bl = y_bl.mul_add(y_lower_increment, x_bl);
                let prod_sum_br = y_br.mul_add(y_lower_increment, x_br);

                // Base interpolation.
                let prod_sum_top_dif = prod_sum_tr - prod_sum_tl;
                let prod_sum_low_dif = prod_sum_br - prod_sum_bl;
                base_top[block] = x_lerp.mul_add(prod_sum_top_dif, prod_sum_tl) * weight_vec;
                let base_lerp_bottom = x_lerp.mul_add(prod_sum_low_dif, prod_sum_bl) * weight_vec;
                base_dif[block] = base_lerp_bottom - base_top[block];

                // Offset interpolation.
                y_offset_top[block] = x_lerp.mul_add(y_tr - y_tl, y_tl) * y_weighted_increment;
                let y_offset_lerp_bottom = x_lerp.mul_add(y_br - y_bl, y_bl) * y_weighted_increment;
                y_offset_dif[block] = y_offset_lerp_bottom - y_offset_top[block];
            }

            let mut y_it = y_start_index;
            while y_it < y_end_index {
                if y_it + 4 > y_end_index {
                    Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
                        base_dif,
                        base_top,
                        y_offset_dif,
                        y_offset_top,
                        x_it,
                        y_it,
                        &y_lerp_array,
                        result,
                        0,
                    );
                    y_it += 1;
                } else {
                    for i in 0..4 {
                        Self::process_lerp_block::<INITIALIZE, IS_TAIL>(
                            base_dif,
                            base_top,
                            y_offset_dif,
                            y_offset_top,
                            x_it,
                            y_it,
                            &y_lerp_array,
                            result,
                            i,
                        );
                    }
                    y_it += 4;
                }
            }
        }
    }

    #[inline(always)]
    fn process_lerp_block<const INITIALIZE: bool, const IS_TAIL: bool>(
        base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
        base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
        y_offset_dif: &[ArchSimd<f32>; NUM_BLOCKS],
        y_offset_top: &[ArchSimd<f32>; NUM_BLOCKS],
        x_it: usize,
        y_it: usize,
        y_lerp_array: &SimdArray<f32, Y>,
        result: &mut SimdArray<f32, N>,
        y_idx: usize,
    ) {
        let y_lerp = ArchSimd::splat(unsafe { y_lerp_array.get_unchecked(y_it + y_idx) });

        let range = if IS_TAIL {
            0..Self::BLOCK_TAIL_SIZE
        } else {
            0..NUM_BLOCKS
        };

        for block in range {
            let x_index = x_it + LANES * block;
            let index = x_index + y_it * X + X * y_idx;
            let output = y_lerp.mul_add(base_dif[block], base_top[block]);

            let val = if INITIALIZE {
                output
            } else {
                unsafe { output + result.load_simd_tail_checked(index) }
            };

            unsafe {
                if IS_TAIL && Self::HAS_SIMD_TAIL && x_index >= Self::SIMD_TAIL_START {
                    result.partial_store_simd_unchecked(index, val, Self::SIMD_TAIL_SIZE);
                } else {
                    result.store_simd_unchecked(index, val)
                };
            }

            base_dif[block] += y_offset_dif[block];
            base_top[block] += y_offset_top[block];
        }
    }
}
