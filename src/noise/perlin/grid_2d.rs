use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Range;

use crate::api::grid::interface::GridNoiseParams;
use crate::fractal::{Fractal, FractalState};
use crate::grid_data::{GridData, Lerp};
use crate::grid_helpers::{
    Arena, ArenaBuffer, InterpolationConfig, MaybeUninitSliceSimdExt, assume_init_slice, maybe_tail_load, maybe_tail_store, pad_grid_size, validate_grid_size, validate_state_size
};
use crate::noise::perlin::constants::*;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::{GridNoiseImpl, Perlin};

// ————————————————————————————————————————————————————————————————
// ————— 2D Perlin Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
pub const LANES: usize = ArchSimd::<f32>::LANES;
pub const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

pub struct PerlinGradients2D<'a> {
    pub tl: [&'a mut [MaybeUninit<f32>]; 2],
    pub tr: [&'a mut [MaybeUninit<f32>]; 2],
    pub bl: [&'a mut [MaybeUninit<f32>]; 2],
    pub br: [&'a mut [MaybeUninit<f32>]; 2],
}

impl<'a> PerlinGradients2D<'a> {
    #[inline(always)]
    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
        Self {
            tl: [arena.allocate(size), arena.allocate(size)],
            tr: [arena.allocate(size), arena.allocate(size)],
            bl: [arena.allocate(size), arena.allocate(size)],
            br: [arena.allocate(size), arena.allocate(size)],
        }
    }

    #[inline(always)]
    pub fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.tl, &mut self.bl);
        std::mem::swap(&mut self.tr, &mut self.br);
    }
}

impl<'a> fmt::Debug for PerlinGradients2D<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("GridData")
                .field("tl.x", &assume_init_slice(self.tl[0]))
                .field("tr.x", &assume_init_slice(self.tr[0]))
                .field("bl.x", &assume_init_slice(self.bl[0]))
                .field("br.x", &assume_init_slice(self.br[0]))
                .field("tl.y", &assume_init_slice(self.tl[1]))
                .field("tr.y", &assume_init_slice(self.tr[1]))
                .field("bl.y", &assume_init_slice(self.bl[1]))
                .field("br.y", &assume_init_slice(self.br[1]))
                .finish()
        }
    }
}

const LERP: u8 = Lerp::Quintic as u8;
impl GridNoiseImpl<2> for Perlin {
    fn sample<F: Fractal, const INIT: bool, const FINAL: bool>(
        params: GridNoiseParams<2>,
        fractal_config: F::Config,
        state: &mut [f32],
        dst: &mut [f32],
    ) {
        validate_grid_size(params.grid_size, dst.len());
        validate_state_size::<F, _>(params.grid_size, state.len());
        let padded_size = pad_grid_size(params.grid_size);

        let required_cache = padded_size[1] * 3 + padded_size[0] * 12;
        let mut cache = ArenaBuffer::with_capacity(required_cache);
        let mut arena = Arena::with_cache(&mut cache);

        // SIMD Slice constants.
        let bilerp_config = InterpolationConfig::new(params.grid_size[0]);

        let mut sub_arena = arena.allocate_arena(padded_size[0] * 3 + padded_size[1] * 3);

        let mut grid_data = GridData::new::<LERP>(&params, &mut sub_arena, &padded_size);

        // Allocate scratch buffer for gradients.
        let grad_scratch = arena.allocate(padded_size[0]);

        // Initialize gradient vectors.
        let mut gradients = PerlinGradients2D::new(&mut arena, padded_size[0]);

        // Set the top gradients.
        grid_gradients_2d(
            &params,
            &mut grid_data,
            grad_scratch,
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
                &params,
                &mut grid_data,
                grad_scratch,
                &mut gradients.bl,
                &mut gradients.br,
                y_it,
            );

            let y_range = y_cur_index..y_next_index;
            grid_dotted_bilerp::<F, INIT, FINAL>(
                &bilerp_config,
                &fractal_config,
                &grid_data,
                &gradients,
                y_range,
                (state, dst),
            );

            // Reuse the top and bottom gradients.
            gradients.swap_top_bottom();

            y_cur_index = y_next_index;
        }
    }
}

#[inline(always)]
pub(super) fn grid_gradients_2d<'a>(
    params: &GridNoiseParams<2>,
    grid_data: &mut GridData<2>,
    grad_buffer: &mut [MaybeUninit<u32>],
    left: &mut [&'a mut [MaybeUninit<f32>]; 2],
    right: &mut [&'a mut [MaybeUninit<f32>]; 2],
    y_it: usize,
) {
    let y_start = grid_data.grid_start[1] + y_it as i32;
    let y_rem = grid_data.octave_tiling[1].map_or(y_start, |t| y_start % t as i32);
    let y_vec = ArchSimd::splat((y_rem as u32).wrapping_mul(params.seed));

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
        let seed_vec = ArchSimd::splat(params.seed);

        let end_index = grid_data.num_loops[0] + 1;
        for i in (0..end_index).step_by(LANES) {
            let x_floats = x_vec.cast_float();
            let x_rem = x_floats - (x_floats / x_tiling).floor() * x_tiling;
            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;

            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe { grad_buffer.write_simd_aligned(i, indices) };
            x_vec += x_vec_stride;
        }
    } else {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(params.seed);
        let mut x_vec =
            ArchSimd::splat((grid_data.grid_start[0] as u32).wrapping_mul(params.seed)) + iota_vec;
        let x_vec_stride = ArchSimd::splat((LANES as u32).wrapping_mul(params.seed));

        // Main vectorized bit mixing loop.
        let end_index = grid_data.num_loops[0] + 1;
        for i in (0..end_index).step_by(LANES) {
            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
            let indices: ArchSimd<u32> = (y_shuf * x_shuf) >> 29;
            unsafe { grad_buffer.write_simd_aligned(i, indices) };
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
                left[1].write_simd(index, ly);
                left[0].write_simd(index, lx);
                right[1].write_simd(index, ry);
                right[0].write_simd(index, rx);

                amount -= LANES as isize;
                index += LANES;
            }
        }

        x_cur_index = x_next_index;
    }

    // Compute x dot products (Better to do here since these dot products get reused and operate per element).
    for i in (0..params.grid_size[0]).step_by(LANES) {
        unsafe {
            let cur_dist = grid_data.distances[0].load_simd_aligned(i);
            let cur_left = left[0].load_simd_aligned(i);
            let cur_right = right[0].load_simd_aligned(i);

            left[0].write_simd_aligned(i, cur_left * cur_dist);
            right[0].write_simd_aligned(i, cur_right.mul_sub(cur_dist, cur_right));
        }
    }
}

/// Handles interpolation execution state and fills
/// the dst slice with interpolated values from gradient dot produtcts.
pub(crate) struct DottedBilerpExecuter<'a, F: Fractal, const INIT: bool, const FINAL: bool> {
    config: &'a InterpolationConfig<NUM_BLOCKS>,
    fractal_config: &'a F::Config,
    grid_data: &'a GridData<'a, 2>,
    gradients: &'a PerlinGradients2D<'a>,
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
#[inline(always)]
pub(super) fn grid_dotted_bilerp<F: Fractal, const INIT: bool, const FINAL: bool>(
    config: &InterpolationConfig<NUM_BLOCKS>,
    fractal_config: &F::Config,
    grid_data: &GridData<2>,
    gradients: &PerlinGradients2D,
    y_range: Range<usize>,
    output: (&mut [f32], &mut [f32]),
) {
    let y_frac_start = unsafe {
        grid_data.distances[1]
            .get_unchecked(y_range.start)
            .assume_init()
    };

    let mut executer = DottedBilerpExecuter::<F, INIT, FINAL> {
        config,
        fractal_config,
        grid_data,
        gradients,
        y_range,
        top: Default::default(),
        dif: Default::default(),
        d_top: Default::default(),
        d_dif: Default::default(),
        weight_vec: ArchSimd::splat(grid_data.weight),
        y_weighted_increment: ArchSimd::splat(grid_data.increment[1] * grid_data.weight),
        y_upper_increment: ArchSimd::splat(y_frac_start),
        y_lower_increment: ArchSimd::splat(y_frac_start - 1.0),
    };

    let (state, dst) = output;
    if config.has_block_head {
        executer.interpolate::<false>(state, dst);
    }

    if config.has_block_tail {
        executer.interpolate::<true>(state, dst);
        std::hint::cold_path();
    }
}

impl<'a, F: Fractal, const INIT: bool, const FINAL: bool> DottedBilerpExecuter<'a, F, INIT, FINAL> {
    #[inline(always)]
    pub fn interpolate<const IS_TAIL: bool>(
        &mut self,
        state: &mut [f32],
        dst: &mut [f32],
    ) {
        let range = if IS_TAIL {
            self.config.block_tail_start..self.grid_data.grid_size[0]
        } else {
            0..self.config.block_tail_start
        };

        for x in range.step_by(BLOCK_LANES) {
            self.initialize_factors::<IS_TAIL>(x);

            let mut y = self.y_range.start;
            while y < self.y_range.end {
                if y + 4 > self.y_range.end {
                    self.process_factors::<IS_TAIL>(x, y, state, dst);
                    y += 1;
                } else {
                    self.process_factors::<IS_TAIL>(x, y, state, dst);
                    self.process_factors::<IS_TAIL>(x, y + 1, state, dst);
                    self.process_factors::<IS_TAIL>(x, y + 2, state, dst);
                    self.process_factors::<IS_TAIL>(x, y + 3, state, dst);
                    y += 4;
                }
            }
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
    fn process_factors<const IS_TAIL: bool>(
        &mut self,
        x: usize,
        y: usize,
        state: &mut [f32],
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

        let index = y * self.grid_data.grid_size[0] + x;
        let tail_end = index + self.config.tail_size;
        for block in range {
            let index = index + block * LANES;
            let output = y_lerp.mul_add(self.dif[block], self.top[block]);

            let (cur_state, mut result) = if INIT {
                F::initialize(self.fractal_config, output)
            } else {
                let mut cur_state = F::State::default();
                for i in 0..F::State::STATE_SIZE {
                    let offset = i * self.grid_data.total_size;
                    let index = index + offset;
                    let tail_end = tail_end + offset;
                    cur_state[i] = unsafe { maybe_tail_load::<IS_TAIL>(index..tail_end, state) };
                }
                let cur_result = unsafe { maybe_tail_load::<IS_TAIL>(index..tail_end, dst) };
                F::sample(self.fractal_config, cur_state, cur_result, output)
            };

            // Save changes to state.
            if !FINAL {
                for i in 0..F::State::STATE_SIZE {
                    let offset = i * self.grid_data.total_size;
                    let index = index + offset;
                    let tail_end = tail_end + offset;
                    unsafe { maybe_tail_store::<IS_TAIL>(index..tail_end, cur_state[i], state) };
                }
            }

            if FINAL {
                result = F::finalize(self.fractal_config, cur_state, result);
            }

            unsafe { maybe_tail_store::<IS_TAIL>(index..tail_end, result, dst) };

            self.dif[block] += self.d_dif[block];
            self.top[block] += self.d_top[block];
        }
    }
}
