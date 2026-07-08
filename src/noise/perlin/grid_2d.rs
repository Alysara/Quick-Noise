use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Range;

use crate::api::grid::interface::GridNoiseParams;
use crate::grid_helpers::{
    AlignedBuffer, Arena, ArenaCache, CHUNK_SIZE, assume_init_slice, pad_grid_size,
    validate_grid_size,
};
use crate::noise::perlin::constants::*;
use crate::perlin::grid_data::PerlinGridData;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::{GridNoiseImpl, Perlin};

use std::array::from_fn;

// ————————————————————————————————————————————————————————————————
// ————— 2D Perlin Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

#[derive(std::fmt::Debug)]
pub struct PerlinGradients2D {
    pub tl: [AlignedBuffer<f32>; 2],
    pub tr: [AlignedBuffer<f32>; 2],
    pub bl: [AlignedBuffer<f32>; 2],
    pub br: [AlignedBuffer<f32>; 2],
}

impl PerlinGradients2D {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            tl: [AlignedBuffer::new(), AlignedBuffer::new()],
            tr: [AlignedBuffer::new(), AlignedBuffer::new()],
            bl: [AlignedBuffer::new(), AlignedBuffer::new()],
            br: [AlignedBuffer::new(), AlignedBuffer::new()],
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
        let block_tail_start: usize = ((x_dim / BLOCK_LANES) * BLOCK_LANES);
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

impl GridNoiseImpl<2> for Perlin {
    #[inline(never)]
    fn sample<const INIT: bool>(params: GridNoiseParams<2>, dst: &mut [f32]) {
        validate_grid_size(params.grid_size, dst.len());

        // SIMD Slice constants.
        let bilerp_config = BilerpConfig::new(params.grid_size[0]);
        let mut grid_data = PerlinGridData::new(&params);

        // Allocate scratch buffer for gradients.
        let mut grad_scratch = AlignedBuffer::<u32>::new();

        // Initialize gradient vectors.
        let mut gradients = PerlinGradients2D::new();

        grid_data.for_each_grid_chunk(&mut |grid_data| {
            // Set the top gradients.
            grid_gradients_2d(
                params.seed,
                grid_data,
                &mut grad_scratch,
                &mut gradients.tl,
                &mut gradients.tr,
                0,
            );

            // Iterate through single y chunks but full x chunks.
            let mut y_cur_index = 0;
            for y_it in 0..grid_data.num_loops[1] {
                let y_next_index =
                    unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize };

                // Set bottom gradients.
                grid_gradients_2d(
                    params.seed,
                    grid_data,
                    &mut grad_scratch,
                    &mut gradients.bl,
                    &mut gradients.br,
                    y_it,
                );

                let y_range = y_cur_index..y_next_index;
                grid_dotted_bilerp::<INIT>(
                    &bilerp_config,
                    grid_data,
                    &gradients,
                    params.weight,
                    y_range,
                    dst,
                );

                // Reuse the top and bottom gradients.
                gradients.swap_top_bottom();

                y_cur_index = y_next_index;
            }
        });
    }
}

#[inline(always)]
pub(super) fn grid_gradients_2d(
    seed: u32,
    grid_data: &PerlinGridData<2>,
    grad_buffer: &mut AlignedBuffer<u32>,
    left: &mut [AlignedBuffer<f32>; 2],
    right: &mut [AlignedBuffer<f32>; 2],
    y_it: usize,
) {
    let y_start = grid_data.grid_start[1] + y_it as i32;
    let y_rem = grid_data.octave_tiling[1].map_or(y_start, |t| y_start % t as i32);
    let y_vec = ArchSimd::splat((y_rem as u32).wrapping_mul(seed));

    let prime = ArchSimd::splat(0x85ebca6b_u32);
    const BYTE_SHUFFLE: [u8; 64] = [
        3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9,
        15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6,
        5, 11, 8, 10, 9, 15, 12, 14, 13,
    ];
    let shuffle_indices = unsafe { ArchSimd::<u8>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;

    if let Some(x_tiling) = grid_data.octave_tiling[0] {
        let x_tiling = ArchSimd::splat(x_tiling as f32);
        let mut x_vec = ArchSimd::splat(grid_data.grid_start[0]) + ArchSimd::iota(0);
        let x_vec_stride = ArchSimd::splat(LANES as i32);
        let seed_vec = ArchSimd::splat(seed);

        let end_index = grid_data.num_loops[0] + 1;
        for i in (0..end_index).step_by(LANES) {
            let x_floats = x_vec.cast_float();
            let x_rem = x_floats - (x_floats / x_tiling).floor() * x_tiling;
            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;

            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe { grad_buffer.store_simd_aligned(i, indices) };
            x_vec += x_vec_stride;
        }
    } else {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(seed);
        let mut x_vec =
            ArchSimd::splat((grid_data.grid_start[0] as u32).wrapping_mul(seed)) + iota_vec;
        let x_vec_stride = ArchSimd::splat((LANES as u32).wrapping_mul(seed));

        // Main vectorized bit mixing loop.
        let end_index = grid_data.num_loops[0] + 1;
        for i in (0..end_index).step_by(LANES) {
            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe { grad_buffer.store_simd_aligned(i, indices) };
            x_vec += x_vec_stride;
        }
    }

    // Loop through the x chunks.
    let mut x_cur_index = 0;
    for x_it in 0..grid_data.num_loops[0] {
        // Find range of gradients to set.
        let x_next_index = unsafe { grid_data.grid_indices[0].get_unchecked(x_it).assume_init() };
        let mut amount = (x_next_index - x_cur_index) as isize;

        unsafe {
            let l = grad_buffer.get_unchecked(x_it).assume_init() as usize;
            let r = grad_buffer.get_unchecked(x_it + 1).assume_init() as usize;

            let ly = ArchSimd::splat(GRADIENTS_2D.get_unchecked(l)[1]);
            let lx = ArchSimd::splat(GRADIENTS_2D.get_unchecked(l)[0]);
            let ry = ArchSimd::splat(GRADIENTS_2D.get_unchecked(r)[1]);
            let rx = ArchSimd::splat(GRADIENTS_2D.get_unchecked(r)[0]);

            let mut index = x_cur_index as usize;
            while amount > 0 {
                left[1].store_simd(index, ly);
                left[0].store_simd(index, lx);
                right[1].store_simd(index, ry);
                right[0].store_simd(index, rx);
                amount -= LANES as isize;
                index += LANES;
            }
        }

        x_cur_index = x_next_index;
    }

    // Compute x dot products (Better to do here since these dot products get reused and operate per element).
    for i in (0..grid_data.cur_size[0]).step_by(LANES) {
        unsafe {
            let cur_dist = grid_data.distances[0].load_simd_aligned(i);
            let cur_left = left[0].load_simd_aligned(i);
            let cur_right = right[0].load_simd_aligned(i);

            left[0].store_simd_aligned(i, cur_dist * cur_left);
            right[0].store_simd_aligned(i, cur_right.mul_sub(cur_dist, cur_right));
        }
    }
}

/// Handles interpolation execution state and fills
/// the dst slice with interpolated values from gradient dot produtcts.
pub(crate) struct DottedBilerpExecuter<'a> {
    config: &'a BilerpConfig,
    grid_data: &'a PerlinGridData<2>,
    gradients: &'a PerlinGradients2D,
    base_index: usize,
    y_range: Range<usize>,
    top: [ArchSimd<f32>; NUM_BLOCKS],
    dif: [ArchSimd<f32>; NUM_BLOCKS],
    d_top: [ArchSimd<f32>; NUM_BLOCKS],
    d_dif: [ArchSimd<f32>; NUM_BLOCKS],
    weight_vec: ArchSimd<f32>,
    y_weighted_increment: ArchSimd<f32>,
    y_upper_increment: ArchSimd<f32>,
    y_lower_increment: ArchSimd<f32>,
}

/// Fills the dst slice with interpolated dot products from gradients.
#[inline(never)]
pub(super) fn grid_dotted_bilerp<const INIT: bool>(
    config: &BilerpConfig,
    grid_data: &PerlinGridData<2>,
    gradients: &PerlinGradients2D,
    weight: f32,
    y_range: Range<usize>,
    dst: &mut [f32],
) {
    let y_frac_start = unsafe {
        grid_data.distances[1]
            .get_unchecked(y_range.start)
            .assume_init()
    };

    let weight_vec = ArchSimd::splat(weight);
    let y_weighted_increment = ArchSimd::splat(grid_data.increment[1] * weight);
    let y_upper_increment = ArchSimd::splat(y_frac_start);
    let y_lower_increment = ArchSimd::splat(y_frac_start - 1.0);

    let grid_size = grid_data.grid_size;
    let completed = grid_data.completed;
    let base_index = grid_size[0] * completed[1] + completed[0];

    // Set up registers per block. Initialization is just to keep Rust happy. Compiler will optimize away.
    let top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
    let dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
    let d_top: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();
    let d_dif: [ArchSimd<f32>; NUM_BLOCKS] = Default::default();

    let mut executer = DottedBilerpExecuter {
        config,
        grid_data,
        gradients,
        base_index,
        y_range,
        top,
        dif,
        d_top,
        d_dif,
        weight_vec,
        y_weighted_increment,
        y_upper_increment,
        y_lower_increment,
    };

    if config.has_block_head {
        executer.interpolate::<INIT, false>(dst);
    }

    // if config.has_block_tail {
    //     executer.interpolate::<INIT, true>(dst);
    //     std::hint::cold_path();
    // }
}

impl<'a> DottedBilerpExecuter<'a> {
    #[inline(always)]
    pub fn interpolate<const INIT: bool, const IS_TAIL: bool>(&mut self, dst: &mut [f32]) {
        // println!("cur_size: {:?}", self.grid_data.cur_size);
        let range = if IS_TAIL {
            self.config.block_tail_start..self.grid_data.cur_size[0]
        } else {
            0..self.grid_data.cur_size[0]
        };

        let y_hop = self.grid_data.grid_size[0];
        let mut x_index = self.base_index;
        for x in range.step_by(BLOCK_LANES) {
            self.initialize_factors::<IS_TAIL>(x);

            let mut i = x_index;
            let mut y = self.y_range.start;
            while y < self.y_range.end {
                if y + 4 > self.y_range.end {
                    self.process_factors::<INIT, IS_TAIL>(i, y, dst);
                    i += y_hop;
                    y += 1;
                    std::hint::cold_path();
                } else {
                    self.process_factors::<INIT, IS_TAIL>(i, y, dst);
                    self.process_factors::<INIT, IS_TAIL>(i + y_hop, y + 1, dst);
                    self.process_factors::<INIT, IS_TAIL>(i + 2 * y_hop, y + 2, dst);
                    self.process_factors::<INIT, IS_TAIL>(i + 3 * y_hop, y + 3, dst);
                    i += 4 * y_hop;
                    y += 4;
                }
            }
            x_index += BLOCK_LANES;
        }
    }

    #[inline(always)]
    fn initialize_factors<const IS_TAIL: bool>(&mut self, x: usize) {
        let num_blocks = if IS_TAIL {
            self.config.block_tail_size
        } else {
            NUM_BLOCKS
        };

        // These blocked loops will get entirely unrolled by the compiler.
        for block in 0..num_blocks {
            // Load gradients into registers.
            let index = x + LANES * block;

            // println!("index: {index}");

            let x_lerp = unsafe { self.grid_data.fade_factors[0].load_simd_aligned(index) };
            let x_tl = unsafe { self.gradients.tl[0].load_simd_aligned(index) };
            let x_tr = unsafe { self.gradients.tr[0].load_simd_aligned(index) };
            let x_bl = unsafe { self.gradients.bl[0].load_simd_aligned(index) };
            let x_br = unsafe { self.gradients.br[0].load_simd_aligned(index) };
            let y_tl = unsafe { self.gradients.tl[1].load_simd_aligned(index) };
            let y_tr = unsafe { self.gradients.tr[1].load_simd_aligned(index) };
            let y_bl = unsafe { self.gradients.bl[1].load_simd_aligned(index) };
            let y_br = unsafe { self.gradients.br[1].load_simd_aligned(index) };

            // Compute base dot products.
            let prod_sum_tl = y_tl.mul_add(self.y_upper_increment, x_tl);
            let prod_sum_tr = y_tr.mul_add(self.y_upper_increment, x_tr);
            let prod_sum_bl = y_bl.mul_add(self.y_lower_increment, x_bl);
            let prod_sum_br = y_br.mul_add(self.y_lower_increment, x_br);

            // Base interpolation.
            let prod_sum_top_dif = prod_sum_tr - prod_sum_tl;
            let prod_sum_low_dif = prod_sum_br - prod_sum_bl;
            self.top[block] = x_lerp.mul_add(prod_sum_top_dif, prod_sum_tl) * self.weight_vec;
            let base_lerp_bottom = x_lerp.mul_add(prod_sum_low_dif, prod_sum_bl) * self.weight_vec;
            self.dif[block] = base_lerp_bottom - self.top[block];

            // Offset interpolation.
            self.d_top[block] = x_lerp.mul_add(y_tr - y_tl, y_tl) * self.y_weighted_increment;
            let y_offset_lerp_bottom =
                x_lerp.mul_add(y_br - y_bl, y_bl) * self.y_weighted_increment;
            self.d_dif[block] = y_offset_lerp_bottom - self.d_top[block];
        }
    }

    #[inline(always)]
    fn process_factors<const INIT: bool, const IS_TAIL: bool>(
        &mut self,
        index: usize,
        y: usize,
        dst: &mut [f32],
    ) {
        let y_lerp = ArchSimd::splat(unsafe {
            self.grid_data.fade_factors[1]
                .get_unchecked(y)
                .assume_init()
        });

        let range = if IS_TAIL {
            0..self.config.block_tail_size
        } else {
            0..NUM_BLOCKS
        };
        

        for block in range {
            let index = index + block * LANES;
            let output = y_lerp.mul_add(self.dif[block], self.top[block]);

            let val = match (INIT, IS_TAIL) {
                (true, _) => output,
                (false, true) => unsafe {
                    output + ArchSimd::from_slice(dst.get_unchecked(index..))
                },
                (false, false) => unsafe {
                    output + ArchSimd::from_slice_unchecked(dst.get_unchecked(index..))
                },
            };

            // if IS_TAIL && self.config.has_simd_tail && x_index >= self.config.simd_tail_start {
            //     val.copy_to_slice(&mut dst[index..]);
            // } else {
                unsafe { val.copy_to_slice_unchecked(dst.get_unchecked_mut(index..)) };
            // };

            self.dif[block] += self.d_dif[block];
            self.top[block] += self.d_top[block];
        }
    }
}
