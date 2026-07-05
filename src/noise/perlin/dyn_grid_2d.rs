use std::mem::{MaybeUninit, take};
use std::time::Instant;

use crate::grid_helpers::{
    Arena, ArenaCache, configure_tiling, grid_fill_indices, grid_fill_indices_slice, multiset_slice,
};
use crate::math::vec::{BasicVec, Vec2};
use crate::noise::perlin::constants::*;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::simd::simd_array::{SimdArray, TailInfo};

// ————————————————————————————————————————————————————————————————
// ————— 2D Perlin Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct PerlinContainer2D<'a> {
    pub tl: Vec2<&'a mut [MaybeUninit<f32>]>,
    pub tr: Vec2<&'a mut [MaybeUninit<f32>]>,
    pub bl: Vec2<&'a mut [MaybeUninit<f32>]>,
    pub br: Vec2<&'a mut [MaybeUninit<f32>]>,
}

impl<'a> PerlinContainer2D<'a> {
    #[inline(always)]
    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
        Self {
            tl: Vec2::new(arena.allocate(size), arena.allocate(size)),
            tr: Vec2::new(arena.allocate(size), arena.allocate(size)),
            bl: Vec2::new(arena.allocate(size), arena.allocate(size)),
            br: Vec2::new(arena.allocate(size), arena.allocate(size)),
        }
    }

    pub fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.tl, &mut self.bl);
        std::mem::swap(&mut self.tr, &mut self.br);
    }
}

const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
const LANES: usize = ArchSimd::<f32>::LANES;
const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

pub struct BilerpConfig {
    pub simd_tail_size: usize,
    pub has_simd_tail: bool,
    pub has_block_head: bool,
    pub has_block_tail: bool,
    pub block_tail_size: usize,
    pub block_tail_start: usize,
    pub simd_tail_start: usize,
}

impl BilerpConfig {
    pub fn new(x_dim: usize) -> Self {
        let simd_tail_size: usize = x_dim % LANES;
        let has_simd_tail: bool = simd_tail_size > 0;
        let has_block_head: bool = x_dim >= BLOCK_LANES;
        let has_block_tail: bool = !x_dim.is_multiple_of(BLOCK_LANES);
        let block_tail_size: usize = (x_dim % BLOCK_LANES).div_ceil(LANES);
        let block_tail_start: usize = (x_dim / BLOCK_LANES) * BLOCK_LANES;
        let simd_tail_start: usize = x_dim - simd_tail_size;

        Self {
            simd_tail_size,
            has_simd_tail,
            has_block_head,
            has_block_tail,
            block_tail_size,
            block_tail_start,
            simd_tail_start,
        }
    }
}

#[inline(always)]
pub fn grid_2d<const INITIALIZE: bool>(
    dimensions: Vec2<usize>,
    result: &mut [f32],
    // cache: &mut [f32],
    seed: u32,
    position: Vec2<i32>,
    frequency: Vec2<f32>,
    weight: f32,
    magnification: f32,
    tiling: Vec2<Option<u32>>,
) {
    let size = dimensions.x * dimensions.y;
    assert!(
        result.len() >= size,
        "Uniform grid with dimensions {:?} has a size of {size}, which is less than the given slice length of {}",
        dimensions,
        result.len()
    );

    let num_lanes = Vec2::splat(ArchSimd::<f32>::LANES);
    let padded_dim = num_lanes - dimensions % num_lanes + dimensions;
    let required_cache = padded_dim.y * 3 + padded_dim.x * 12;

    let mut cache = ArenaCache::with_capacity(required_cache);
    let mut arena = Arena::with_cache(&mut cache);

    // SIMD Slice constants.
    let bilerp_config = BilerpConfig::new(dimensions.x);

    let increment: Vec2<f32> = frequency * magnification;
    let block_pos: Vec2<i32> = position * Vec2::new(dimensions.x as i32, dimensions.y as i32);

    // Get the starting gradient coordinates and how far the first sample is to the next one.
    let grid_start: Vec2<i32> = (block_pos.as_f32() * increment).floor().as_i32();
    let frac_start: Vec2<f32> =
        (block_pos.as_f32() * increment - grid_start.as_f32()).float_max(Vec2::splat(0.0));

    // Get the distances from the gradient gridpoints.
    let mut x_cur_dist = ArchSimd::iota(frac_start.x) * ArchSimd::splat(increment.x);
    let mut y_cur_dist = ArchSimd::iota(frac_start.y) * ArchSimd::splat(increment.y);
    let x_chunk_increment = ArchSimd::splat(increment.x * LANES as f32);
    let y_chunk_increment = ArchSimd::splat(increment.y * LANES as f32);

    let x_distances = arena.allocate(padded_dim.x);
    let y_distances = arena.allocate(padded_dim.y);

    // Quintic lerp the distances to get the fade factor.
    let x_lerp = arena.allocate(padded_dim.x);
    let y_lerp = arena.allocate(padded_dim.y);

    for i in (0..padded_dim.x).step_by(LANES) {
        let cur_dist = x_cur_dist.fract();
        let cur_lerp = cur_dist.quintic_lerp();

        unsafe {
            cur_dist.copy_to_slice_unchecked(x_distances.get_unchecked_mut(i..).assume_init_mut());
            cur_lerp.copy_to_slice_unchecked(x_lerp.get_unchecked_mut(i..).assume_init_mut());
        }
        x_cur_dist += x_chunk_increment;
    }

    for i in (0..padded_dim.y).step_by(LANES) {
        let cur_dist = y_cur_dist.fract();
        let cur_lerp = cur_dist.quintic_lerp();

        unsafe {
            cur_dist.copy_to_slice_unchecked(y_distances.get_unchecked_mut(i..).assume_init_mut());
            cur_lerp.copy_to_slice_unchecked(y_lerp.get_unchecked_mut(i..).assume_init_mut());
        }
        y_cur_dist += y_chunk_increment;
    }

    let x_grid_indices = arena.allocate(dimensions.x);
    let y_grid_indices = arena.allocate(dimensions.y);

    // let mut x_grid_indices = unsafe { SimdArray::<u32, X>::new_uninit() };
    // let mut y_grid_indices = unsafe { SimdArray::<u32, Y>::new_uninit() };
    let mut num_loops: Vec2<usize> = Vec2::splat(0);

    // Identify the cutoff points between frequency-based grid boundaries .
    grid_fill_indices_slice(x_grid_indices, x_distances, &mut num_loops.x);
    grid_fill_indices_slice(y_grid_indices, y_distances, &mut num_loops.y);

    // println!("x_grid_indices: {:?}", x_grid_indices);

    // Adjust the tiling.
    let octave_tiling = configure_tiling(&tiling, &frequency);

    // Allocate scratch buffer for gradients.
    let grad_buffer = arena.allocate(padded_dim.x);

    // Initialize gradient vectors.
    let mut d_vecs = PerlinContainer2D::new(&mut arena, padded_dim.x);

    // Set the top gradients.
    grid_gradients_2d(
        seed,
        &mut d_vecs.tl,
        &mut d_vecs.tr,
        grad_buffer,
        grid_start.x,
        grid_start.y,
        x_grid_indices,
        num_loops.x,
        x_distances,
        &octave_tiling,
    );

    // Iterate through single x chunks but full y chunks.
    let mut y_cur_index = 0;
    for y_it in 0..num_loops.y {
        let y_cur_fract = unsafe { y_distances.get_unchecked(y_cur_index).assume_init() };
        let y_next_index = unsafe { y_grid_indices.get_unchecked(y_it).assume_init() as usize };

        // Set bottom gradients.
        grid_gradients_2d(
            seed,
            &mut d_vecs.bl,
            &mut d_vecs.br,
            grad_buffer,
            grid_start.x,
            grid_start.y + y_it as i32 + 1,
            x_grid_indices,
            num_loops.x,
            x_distances,
            &octave_tiling,
        );

        // Perform dot products on x and trilinear interpolation (with quintic fade).
        grid_dotted_bilerp::<INITIALIZE>(
            &bilerp_config,
            &d_vecs,
            &dimensions,
            y_cur_fract,
            increment.y,
            x_lerp,
            y_lerp,
            y_cur_index,
            y_next_index,
            weight,
            result,
        );

        // Reuse the top and bottom gradients.
        d_vecs.swap_top_bottom();

        y_cur_index = y_next_index;
    }
}

#[inline(always)]
pub(super) fn grid_gradients_2d<'a>(
    seed: u32,
    left: &mut Vec2<&'a mut [MaybeUninit<f32>]>,
    right: &mut Vec2<&'a mut [MaybeUninit<f32>]>,
    grad_buffer: &mut [MaybeUninit<u32>],
    x_start: i32,
    y_start: i32,
    x_grid_indices: &[MaybeUninit<u32>],
    x_num_loops: usize,
    x_distances: &[MaybeUninit<f32>],
    tiling: &Vec2<Option<u32>>,
) {
    // let time = Instant::now();
    let y_rem = tiling.y.map_or(y_start, |t| y_start % t as i32);
    let y_vec = ArchSimd::splat((y_rem as u32).wrapping_mul(seed));

    let prime = ArchSimd::splat(0x85ebca6b_u32);
    const BYTE_SHUFFLE: [u8; 64] = [
        3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9,
        15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6,
        5, 11, 8, 10, 9, 15, 12, 14, 13,
    ];
    let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);
    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;

    if let Some(x_tiling) = tiling.x {
        let x_tiling = ArchSimd::splat(x_tiling as f32);
        let mut x_vec = ArchSimd::splat(x_start) + ArchSimd::iota(0);
        let x_vec_stride = ArchSimd::splat(ArchSimd::<f32>::LANES as i32);
        let seed_vec = ArchSimd::splat(seed);

        let end_index = x_num_loops + 1;
        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let x_floats = x_vec.cast_float();
            let x_rem = x_floats - (x_floats / x_tiling).floor() * x_tiling;
            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;

            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe {
                indices
                    .copy_to_slice_unchecked(grad_buffer.get_unchecked_mut(i..).assume_init_mut())
            };
            x_vec += x_vec_stride;
        }
    } else {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(seed);
        let mut x_vec = ArchSimd::splat((x_start as u32).wrapping_mul(seed)) + iota_vec;
        let x_vec_stride = ArchSimd::splat((ArchSimd::<f32>::LANES as u32).wrapping_mul(seed));

        // Main vectorized bit mixing loop.
        let end_index = x_num_loops + 1;
        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe {
                indices
                    .copy_to_slice_unchecked(grad_buffer.get_unchecked_mut(i..).assume_init_mut())
            };
            x_vec += x_vec_stride;
        }
    }

    let mut arrays = [
        std::mem::take(&mut left.y),
        std::mem::take(&mut left.x),
        std::mem::take(&mut right.y),
        std::mem::take(&mut right.x),
    ];

    // Loop through the y chunks.
    let mut x_cur_index = 0;
    for (x_it, x_next_index) in x_grid_indices.iter().enumerate().take(x_num_loops) {
        // let x_next_index = x_grid_indices[x_it];

        // Find range of gradients to set.
        let x_next_index = unsafe { x_next_index.assume_init() };
        let set_amount = x_next_index - x_cur_index;

        unsafe {
            let l = grad_buffer.get_unchecked(x_it).assume_init() as usize;
            let r = grad_buffer.get_unchecked(x_it + 1).assume_init() as usize;

            let values = [
                GRADIENTS_2D.get_unchecked(l).y,
                GRADIENTS_2D.get_unchecked(l).x,
                GRADIENTS_2D.get_unchecked(r).y,
                GRADIENTS_2D.get_unchecked(r).x,
            ];

            multiset_slice::<4>(
                &mut arrays,
                &values,
                x_cur_index as usize,
                set_amount as isize,
            );
        }

        x_cur_index = x_next_index;
    }

    let [ly, lx, ry, rx] = arrays;
    left.y = ly;
    left.x = lx;
    right.y = ry;
    right.x = rx;

    // Compute y dot products (Better to do here since these dot products get reused and operate per element).
    for i in (0..x_distances.len()).step_by(ArchSimd::<f32>::LANES) {
        unsafe {
            let cur_dist =
                ArchSimd::from_slice_unchecked(x_distances.get_unchecked(i..).assume_init_ref());
            let cur_left =
                ArchSimd::from_slice_unchecked(left.x.get_unchecked(i..).assume_init_ref());
            let cur_right =
                ArchSimd::from_slice_unchecked(right.x.get_unchecked(i..).assume_init_ref());

            let new_left = cur_dist * cur_left;
            let new_right = cur_right.mul_sub(cur_dist, cur_right);

            new_left.copy_to_slice_unchecked(left.x.get_unchecked_mut(i..).assume_init_mut());
            new_right.copy_to_slice_unchecked(right.x.get_unchecked_mut(i..).assume_init_mut());
        }
    }
}

#[inline(always)]
pub(super) fn grid_dotted_bilerp<const INITIALIZE: bool>(
    config: &BilerpConfig,
    gradients: &PerlinContainer2D,
    dimensions: &Vec2<usize>,
    y_frac_start: f32,
    y_increment: f32,
    x_lerp_array: &[MaybeUninit<f32>],
    y_lerp_array: &[MaybeUninit<f32>],
    y_start_index: usize,
    y_end_index: usize,
    weight: f32,
    result: &mut [f32],
) {
    // let time = Instant::now();
    let weight_vec = ArchSimd::splat(weight);
    let y_weighted_increment = ArchSimd::splat(y_increment * weight);
    let y_upper_increment = ArchSimd::splat(y_frac_start);
    let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);

    // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
    let mut base_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
    let mut base_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
    let mut y_offset_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
    let mut y_offset_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

    if config.has_block_head {
        grid_dotted_bilerp_helper::<INITIALIZE, false>(
            config,
            gradients,
            dimensions,
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

    // println!("Bilerp done in {:?}!", time.elapsed());
    // if config.has_block_tail {
    //     grid_dotted_bilerp_helper::<INITIALIZE, true>(
    //         config,
    //         gradients,
    //         dimensions,
    //         &mut base_dif,
    //         &mut base_top,
    //         &mut y_offset_dif,
    //         &mut y_offset_top,
    //         x_lerp_array,
    //         y_lerp_array,
    //         y_upper_increment,
    //         y_lower_increment,
    //         y_start_index,
    //         y_end_index,
    //         weight_vec,
    //         y_weighted_increment,
    //         result,
    //     );
    // }
}

#[inline(always)]
fn grid_dotted_bilerp_helper<const INITIALIZE: bool, const IS_TAIL: bool>(
    config: &BilerpConfig,
    gradients: &PerlinContainer2D,
    dimensions: &Vec2<usize>,
    base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
    base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
    y_offset_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
    y_offset_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
    x_lerp_array: &[MaybeUninit<f32>],
    y_lerp_array: &[MaybeUninit<f32>],
    y_upper_increment: ArchSimd<f32>,
    y_lower_increment: ArchSimd<f32>,
    y_start_index: usize,
    y_end_index: usize,
    weight_vec: ArchSimd<f32>,
    y_weighted_increment: ArchSimd<f32>,
    result: &mut [f32],
) {
    let range = if IS_TAIL {
        config.block_tail_start..dimensions.x
    } else {
        0..config.block_tail_start
    };

    let num_blocks = if IS_TAIL {
        config.block_tail_size
    } else {
        NUM_BLOCKS
    };

    for x_it in range.step_by(BLOCK_LANES) {
        // These blocked loops will get entirely unrolled by the compiler.
        for block in 0..num_blocks {
            // Load gradients into registers.

            let index = x_it + LANES * block;
            let (x_lerp, x_tl, x_tr, x_bl, x_br, y_tl, y_tr, y_bl, y_br) = unsafe {
                (
                    ArchSimd::from_slice_unchecked(
                        x_lerp_array.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.tl.x.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.tr.x.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.bl.x.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.br.x.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.tl.y.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.tr.y.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.bl.y.get_unchecked(index..).assume_init_ref(),
                    ),
                    ArchSimd::from_slice_unchecked(
                        gradients.br.y.get_unchecked(index..).assume_init_ref(),
                    ),
                )
            };

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
                process_lerp_block::<INITIALIZE, IS_TAIL>(
                    config,
                    dimensions,
                    base_dif,
                    base_top,
                    y_offset_dif,
                    y_offset_top,
                    x_it,
                    y_it,
                    y_lerp_array,
                    result,
                    0,
                );
                y_it += 1;
            } else {
                for i in 0..4 {
                    process_lerp_block::<INITIALIZE, IS_TAIL>(
                        config,
                        dimensions,
                        base_dif,
                        base_top,
                        y_offset_dif,
                        y_offset_top,
                        x_it,
                        y_it,
                        y_lerp_array,
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
    config: &BilerpConfig,
    dimensions: &Vec2<usize>,
    base_dif: &mut [ArchSimd<f32>; NUM_BLOCKS],
    base_top: &mut [ArchSimd<f32>; NUM_BLOCKS],
    y_offset_dif: &[ArchSimd<f32>; NUM_BLOCKS],
    y_offset_top: &[ArchSimd<f32>; NUM_BLOCKS],
    x_it: usize,
    y_it: usize,
    y_lerp_array: &[MaybeUninit<f32>],
    result: &mut [f32],
    y_idx: usize,
) {
    let y_lerp = ArchSimd::splat(unsafe { y_lerp_array.get_unchecked(y_it + y_idx).assume_init() });

    let range = if IS_TAIL {
        0..config.block_tail_size
    } else {
        0..NUM_BLOCKS
    };

    for block in range {
        let x_index = x_it + LANES * block;
        let index = x_index + y_it * dimensions.x + dimensions.x * y_idx;
        let output = y_lerp.mul_add(base_dif[block], base_top[block]);

        let val = if INITIALIZE {
            output
        } else {
            output + unsafe { ArchSimd::from_slice_unchecked(result.get_unchecked(index..)) }
        };

        if IS_TAIL && config.has_simd_tail && x_index >= config.simd_tail_start {
            val.copy_to_slice(&mut result[index..]);
        } else {
            unsafe { val.copy_to_slice_unchecked(result.get_unchecked_mut(index..)) };
        };

        base_dif[block] += y_offset_dif[block];
        base_top[block] += y_offset_top[block];
    }
}
