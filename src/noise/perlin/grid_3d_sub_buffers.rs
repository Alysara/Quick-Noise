
use std::array::from_fn;
use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Range;

use paste::paste;

use crate::api::grid::interface::GridNoiseParams;
use crate::grid_helpers::{
    Arena, ArenaBuffer, MaybeUninitSliceSimdExt, assume_init_slice, pad_grid_size,
    validate_grid_size,
};
use crate::noise::perlin::constants::*;
use crate::perlin::grid_data::PerlinGridData;
use crate::simd::arch_simd::{ArchSimd, NUM_SIMD_REG};
use crate::{GridNoiseImpl, Perlin};

// ————————————————————————————————————————————————————————————————
// ————— 3D Perlin Grid ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

pub struct PerlinGradients3D<'a> {
    top: &'a mut [MaybeUninit<f32>],
    bottom: &'a mut [MaybeUninit<f32>],
    scratch: &'a mut [MaybeUninit<u32>],
    size: usize,
}

macro_rules! add_gradient_axis {
    ($corner:ident, $buffer:ident, $axis:ident, $offset:literal) => {
        paste! {
            #[inline(always)]
            pub unsafe fn [<load_ $corner _ $axis>](&self, index: usize, buf_size: usize) -> ArchSimd<f32> {
                unsafe { self.$buffer.load_simd_aligned(index + $offset * buf_size) }
            }

            #[inline(always)]
            pub unsafe fn [<write_ $corner _ $axis>](&mut self, index: usize, buf_size: usize, simd: ArchSimd<f32>) {
                unsafe { self.$buffer.write_simd_aligned(index + $offset * buf_size, simd) }
            }

            #[inline(always)]
            pub unsafe fn [<write_ $corner _ $axis _unaligned>](&mut self, index: usize, buf_size: usize, simd: ArchSimd<f32>) {
                unsafe { self.$buffer.write_simd(index + $offset * buf_size, simd) }
            }
        }
    };
}

macro_rules! add_gradient_corner {
    ($corner:ident, $buffer:ident, $x_off:literal, $y_off:literal, $z_off:literal) => {
        add_gradient_axis!($corner, $buffer, x, $x_off);
        add_gradient_axis!($corner, $buffer, y, $y_off);
        add_gradient_axis!($corner, $buffer, z, $z_off);
    };
}

impl<'a> PerlinGradients3D<'a> {
    #[inline(always)]
    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
        Self {
            top: arena.allocate(size * 12),
            bottom: arena.allocate(size * 12),
            scratch: arena.allocate(size * 2),
            size,
        }
    }

    #[inline(always)]
    pub fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.top, &mut self.bottom);
    }

    #[inline(always)]
    pub unsafe fn write_scratch0(&mut self, index: usize, simd: ArchSimd<u32>) {
        unsafe { self.scratch.write_simd_aligned(index, simd) }
    }

    #[inline(always)]
    pub unsafe fn write_scratch1(&mut self, index: usize, buf_size: usize, simd: ArchSimd<u32>) {
        unsafe { self.scratch.write_simd_aligned(index + buf_size, simd) }
    }

    add_gradient_corner!(tlf, top, 0, 1, 2);
    add_gradient_corner!(trf, top, 3, 4, 5);
    add_gradient_corner!(tlb, top, 6, 7, 8);
    add_gradient_corner!(trb, top, 9, 10, 11);

    add_gradient_corner!(blf, bottom, 0, 1, 2);
    add_gradient_corner!(brf, bottom, 3, 4, 5);
    add_gradient_corner!(blb, bottom, 6, 7, 8);
    add_gradient_corner!(brb, bottom, 9, 10, 11);
}

impl<'a> fmt::Debug for PerlinGradients3D<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("PerlinGradients3D")
                .field("tl.x", &assume_init_slice(&self.top[..self.size]))
                .field("tr.x", &assume_init_slice(&self.top[3 * self.size..4 * self.size]))
                .field("bl.x", &assume_init_slice(&self.bottom[..self.size]))
                .field("br.x", &assume_init_slice(&self.bottom[3 * self.size..4 * self.size]))
                .field("tl.y", &assume_init_slice(&self.top[self.size..2 * self.size]))
                .field("tr.y", &assume_init_slice(&self.top[4 * self.size..5 * self.size]))
                .field("bl.y", &assume_init_slice(&self.bottom[self.size..2 * self.size]))
                .field("br.y", &assume_init_slice(&self.bottom[4 * self.size..5 * self.size]))
                .finish()
        }
    }
}
// impl<'a> PerlinGradients3D<'a> {
//     #[inline(always)]
//     pub fn new(arena: &'a mut Arena, size: usize) -> Self {
//         Self {
//             tlf: from_fn(|_| arena.allocate(size)),
//             trf: from_fn(|_| arena.allocate(size)),
//             blf: from_fn(|_| arena.allocate(size)),
//             brf: from_fn(|_| arena.allocate(size)),
//             tlb: from_fn(|_| arena.allocate(size)),
//             trb: from_fn(|_| arena.allocate(size)),
//             blb: from_fn(|_| arena.allocate(size)),
//             brb: from_fn(|_| arena.allocate(size)),
//             scratch: from_fn(|_| arena.allocate(size)),
//         }
//     }
//
//     #[inline(always)]
//     pub fn swap_top_bottom(&mut self) {
//         std::mem::swap(&mut self.tlf, &mut self.blf);
//         std::mem::swap(&mut self.trf, &mut self.brf);
//         std::mem::swap(&mut self.tlb, &mut self.blb);
//         std::mem::swap(&mut self.trb, &mut self.brb);
//     }
// }

// impl<'a> fmt::Debug for PerlinGradients3D<'a> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         unsafe {
//             f.debug_struct("PerlinGradients3D")
//                 .field("tl.x", &assume_init_slice(self.tlf[0]))
//                 .field("tr.x", &assume_init_slice(self.trf[0]))
//                 .field("bl.x", &assume_init_slice(self.blf[0]))
//                 .field("br.x", &assume_init_slice(self.brf[0]))
//                 .field("tl.y", &assume_init_slice(self.tlf[1]))
//                 .field("tr.y", &assume_init_slice(self.trf[1]))
//                 .field("bl.y", &assume_init_slice(self.blf[1]))
//                 .field("br.y", &assume_init_slice(self.brf[1]))
//                 .finish()
//         }
//     }
// }

const NUM_BLOCKS: usize = NUM_SIMD_REG / 8;
const LANES: usize = ArchSimd::<f32>::LANES;
const BLOCK_LANES: usize = NUM_BLOCKS * LANES;

pub struct TrilerpConfig {
    pub has_block_head: bool,
    pub has_block_tail: bool,
    pub block_tail_size: usize,
    pub block_tail_start: usize,
}

impl TrilerpConfig {
    #[inline(always)]
    pub fn new(x_dim: usize) -> Self {
        Self {
            has_block_head: x_dim >= BLOCK_LANES,
            has_block_tail: !x_dim.is_multiple_of(BLOCK_LANES),
            block_tail_size: (x_dim % BLOCK_LANES).div_ceil(LANES),
            block_tail_start: (x_dim / BLOCK_LANES) * BLOCK_LANES,
        }
    }
}

impl GridNoiseImpl<3> for Perlin {
    #[inline(never)]
    fn sample<const INIT: bool>(params: GridNoiseParams<3>, dst: &mut [f32]) {
        // Validate and pad grid size.
        validate_grid_size(params.grid_size, dst.len());
        let padded_size = pad_grid_size(params.grid_size);

        // Arena setup.
        let required_cache = padded_size[0] * 41 + padded_size[1] * 3 + padded_size[2] * 3;
        let mut cache = ArenaBuffer::with_capacity(required_cache);
        let mut arena = Arena::with_cache(&mut cache);
        let mut data_arena = arena.allocate_arena(padded_size.iter().fold(0, |n, x| n + 3 * x));
        let mut trilerp_arena = arena.allocate_arena(padded_size[0] * 12);

        // Allocation setup.

        let bilerp_config = TrilerpConfig::new(params.grid_size[0]);
        let grid_data = PerlinGridData::new(&params, &mut data_arena, &padded_size);
        let mut trilerp_buffers = DottedTrilerpBuffers::new(&mut trilerp_arena, padded_size[0]);
        let mut gradients = PerlinGradients3D::new(&mut arena, padded_size[0]);

        // Iterate through single y chunks but full x chunks.
        let mut z_cur_index = 0;
        for z_it in 0..grid_data.num_loops[2] {
            let z_next_index =
                unsafe { grid_data.grid_indices[2].get_unchecked(z_it).assume_init() as usize };
            let z_range = z_cur_index..z_next_index;

            // Set the top gradients.
            grid_gradients_3d(&params, &grid_data, &mut gradients, 0, z_it);
            gradients.swap_top_bottom();

            let mut y_cur_index = 0;
            for y_it in 0..grid_data.num_loops[1] {
                let y_next_index =
                    unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize };
                let y_range = y_cur_index..y_next_index;

                // Set bottom gradients.
                grid_gradients_3d(&params, &grid_data, &mut gradients, y_it, z_it);

                grid_dotted_trilerp::<INIT>(
                    &mut trilerp_buffers,
                    &bilerp_config,
                    &params,
                    &grid_data,
                    &gradients,
                    (y_range, z_range.clone()),
                    dst,
                );

                // Reuse the top and bottom gradients.
                gradients.swap_top_bottom();

                y_cur_index = y_next_index;
            }
            z_cur_index = z_next_index;
        }
    }
}

#[inline(always)]
pub(super) fn grid_gradients_3d<'a>(
    params: &GridNoiseParams<3>,
    grid_data: &PerlinGridData<3>,
    gradients: &mut PerlinGradients3D<'a>,
    y_it: usize,
    z_it: usize,
) {
    let y_start = y_it as i32 + grid_data.grid_start[2];
    let z_start = z_it as i32 + grid_data.grid_start[2];
    let (z1, z2) = match grid_data.octave_tiling[2] {
        None => (
            (z_start as u32).wrapping_mul(params.seed),
            (z_start as u32)
                .wrapping_mul(params.seed)
                .wrapping_add(params.seed),
        ),
        Some(t) => (
            (z_start % t as i32) as u32,
            ((z_start + 1) % t as i32) as u32,
        ),
    };
    let z_vec = [ArchSimd::splat(z1), ArchSimd::splat(z2)];

    let y_rem = grid_data.octave_tiling[1].map_or(y_start, |t| y_start % t as i32);
    let y_vec = ArchSimd::splat((y_rem as u32).wrapping_mul(params.seed));

    const BYTE_SHUFFLE: [u8; 64] = [
        3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9,
        15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6,
        5, 11, 8, 10, 9, 15, 12, 14, 13,
    ];

    let shuffle_indices = ArchSimd::<u8>::from_slice(&BYTE_SHUFFLE[..]);

    let prime = ArchSimd::splat(0x85ebca6b_u32);
    let z_shuf: [_; 2] = from_fn(|i| z_vec[i].permute_8(shuffle_indices) ^ prime);
    let y_shuf = y_vec.permute_8(shuffle_indices) ^ prime;
    let zy_mix: [_; 2] = from_fn(|i| z_shuf[i] * y_shuf);

    // Main vectorized bit mixing loop.
    let end_index = grid_data.num_loops[0] + 1;

    if let Some(x_tiling) = grid_data.octave_tiling[0] {
        let x_tiling = ArchSimd::splat(x_tiling as f32);
        let mut x_vec = ArchSimd::splat(grid_data.grid_start[0]) + ArchSimd::iota(0);
        let x_vec_stride = ArchSimd::splat(ArchSimd::<f32>::LANES as i32);
        let seed_vec = ArchSimd::splat(params.seed);

        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let x_floats = x_vec.cast_float();
            let x_rem = x_floats - (x_floats / x_tiling).floor() * x_tiling;
            let x_seeded = x_rem.cast_int_round().raw_cast() * seed_vec;
            let x_shuf = x_seeded.permute_8(shuffle_indices) ^ prime;
            let grads: [_; 2] = from_fn(|i| (zy_mix[i] * x_shuf) >> 28);

            unsafe {
                gradients.write_scratch0(i, grads[0]);
                gradients.write_scratch1(i, grid_data.padded_size[0], grads[1]);
            };

            x_vec += x_vec_stride;
        }
    } else {
        let iota_vec = ArchSimd::iota(0) * ArchSimd::splat(params.seed);
        let x_start_seeded = (grid_data.grid_start[0] as u32).wrapping_mul(params.seed);
        let mut x_vec = ArchSimd::splat(x_start_seeded) + iota_vec;
        let x_vec_stride = ArchSimd::splat((LANES as u32).wrapping_mul(params.seed));

        for i in (0..end_index).step_by(ArchSimd::<f32>::LANES) {
            let x_shuf = x_vec.permute_8(shuffle_indices) ^ prime;
            let grads: [_; 2] = from_fn(|i| (zy_mix[i] * x_shuf) >> 28);

            unsafe {
                gradients.write_scratch0(i, grads[0]);
                gradients.write_scratch1(i, grid_data.padded_size[0], grads[1]);
            };
            x_vec += x_vec_stride;
        }
    }

    grid_gradients_3d_set_loop::<true>(grid_data, gradients);
    grid_gradients_3d_set_loop::<false>(grid_data, gradients);

    let buf_size = grid_data.padded_size[0];
    for i in (0..params.grid_size[0]).step_by(LANES) {
        unsafe {
            let cur_dist = grid_data.distances[0].load_simd_aligned(i);
            let lf = gradients.load_blf_x(i, buf_size);
            let rf = gradients.load_brf_x(i, buf_size);
            let lb = gradients.load_blb_x(i, buf_size);
            let rb = gradients.load_brb_x(i, buf_size);

            gradients.write_blf_x(i, buf_size, lf * cur_dist);
            gradients.write_brf_x(i, buf_size, rf.mul_sub(cur_dist, rf));
            gradients.write_blb_x(i, buf_size, lb * cur_dist);
            gradients.write_brb_x(i, buf_size, rb.mul_sub(cur_dist, rb));
        }
    }
}

#[inline(always)]
pub(super) fn grid_gradients_3d_set_loop<'a, const IS_FRONT: bool>(
    grid_data: &PerlinGridData<3>,
    gradients: &mut PerlinGradients3D<'a>,
) {
    let buf_size = grid_data.padded_size[0];

    let mut x_cur_index = 0;
    for x_it in 0..grid_data.num_loops[0] {
        // Find range of gradients to set.
        let x_next_index = unsafe { grid_data.grid_indices[0].get_unchecked(x_it).assume_init() };
        let mut amount = (x_next_index - x_cur_index) as isize;

        unsafe {
            let l = gradients.scratch.get_unchecked(x_it).assume_init() as usize;
            let r = gradients.scratch.get_unchecked(x_it + 1).assume_init() as usize;

            let l = GRADIENTS_3D.get_unchecked(l);
            let r = GRADIENTS_3D.get_unchecked(r);

            let lx = ArchSimd::splat(l[0]);
            let ly = ArchSimd::splat(l[1]);
            let lz = ArchSimd::splat(l[2]);
            let rx = ArchSimd::splat(r[0]);
            let ry = ArchSimd::splat(r[1]);
            let rz = ArchSimd::splat(r[2]);

            let mut index = x_cur_index as usize;
            while amount > 0 {
                if IS_FRONT {
                    gradients.write_blf_x_unaligned(index, buf_size, lx);
                    gradients.write_blf_y_unaligned(index, buf_size, ly);
                    gradients.write_blf_z_unaligned(index, buf_size, lz);
                    gradients.write_brf_x_unaligned(index, buf_size, rx);
                    gradients.write_brf_y_unaligned(index, buf_size, ry);
                    gradients.write_brf_z_unaligned(index, buf_size, rz);
                } else {
                    gradients.write_blb_x_unaligned(index, buf_size, lx);
                    gradients.write_blb_y_unaligned(index, buf_size, ly);
                    gradients.write_blb_z_unaligned(index, buf_size, lz);
                    gradients.write_brb_x_unaligned(index, buf_size, rx);
                    gradients.write_brb_y_unaligned(index, buf_size, ry);
                    gradients.write_brb_z_unaligned(index, buf_size, rz);
                }

                amount -= LANES as isize;
                index += LANES;
            }
        }

        x_cur_index = x_next_index;
    }
}

pub(crate) struct DottedTrilerpBuffers<'a> {
    // buf_size: usize,
    buffer: &'a mut [MaybeUninit<f32>],
    // y_tf_offset: &'a mut [MaybeUninit<f32>],
    // y_bf_offset: &'a mut [MaybeUninit<f32>],
    // y_top_offset_dif: &'a mut [MaybeUninit<f32>],
    // y_bottom_offset_dif: &'a mut [MaybeUninit<f32>],
    // z_tf_offset: &'a mut [MaybeUninit<f32>],
    // z_bf_offset: &'a mut [MaybeUninit<f32>],
    // z_top_offset_dif: &'a mut [MaybeUninit<f32>],
    // z_bottom_offset_dif: &'a mut [MaybeUninit<f32>],
    // tf_base: &'a mut [MaybeUninit<f32>],
    // bf_base: &'a mut [MaybeUninit<f32>],
    // top_base_dif: &'a mut [MaybeUninit<f32>],
    // bottom_base_dif: &'a mut [MaybeUninit<f32>],
}

macro_rules! add_sub_buffer {
    ($name:ident, $index:literal) => {
        paste! {
            #[inline(always)]
            pub unsafe fn [<load_ $name>](&self, index: usize, buf_size: usize) -> ArchSimd<f32> {
                unsafe { self.buffer.load_simd_aligned(index + $index * buf_size) }
            }

            #[inline(always)]
            pub unsafe fn [<write_ $name>](&mut self, index: usize, buf_size: usize, simd: ArchSimd<f32>) {
                unsafe { self.buffer.write_simd_aligned(index + $index * buf_size, simd) }
            }
        }
    };
}

impl<'a> DottedTrilerpBuffers<'a> {
    pub fn new(arena: &'a mut Arena, x_size: usize) -> Self {
        Self {
            // buf_size: x_size,
            buffer: arena.allocate(x_size * 12),
            // y_tf_offset: arena.allocate(x_size),
            // y_bf_offset: arena.allocate(x_size),
            // y_top_offset_dif: arena.allocate(x_size),
            // y_bottom_offset_dif: arena.allocate(x_size),
            // z_tf_offset: arena.allocate(x_size),
            // z_bf_offset: arena.allocate(x_size),
            // z_top_offset_dif: arena.allocate(x_size),
            // z_bottom_offset_dif: arena.allocate(x_size),
            // tf_base: arena.allocate(x_size),
            // bf_base: arena.allocate(x_size),
            // top_base_dif: arena.allocate(x_size),
            // bottom_base_dif: arena.allocate(x_size),
        }
    }

    add_sub_buffer!(y_tf_offset, 0);
    add_sub_buffer!(y_bf_offset, 1);
    add_sub_buffer!(y_top_offset_dif, 2);
    add_sub_buffer!(y_bottom_offset_dif, 3);
    add_sub_buffer!(z_tf_offset, 4);
    add_sub_buffer!(z_bf_offset, 5);
    add_sub_buffer!(z_top_offset_dif, 6);
    add_sub_buffer!(z_bottom_offset_dif, 7);
    add_sub_buffer!(tf_base, 8);
    add_sub_buffer!(bf_base, 9);
    add_sub_buffer!(top_base_dif, 10);
    add_sub_buffer!(bottom_base_dif, 11);
}

/// Handles interpolation execution state and fills
/// the dst slice with interpolated values from gradient dot produtcts.
pub(crate) struct DottedTrilerpExecuter<'a> {
    config: &'a TrilerpConfig,
    params: &'a GridNoiseParams<3>,
    grid_data: &'a PerlinGridData<'a, 3>,
    gradients: &'a PerlinGradients3D<'a>,
    y_range: Range<usize>,
    z_range: Range<usize>,
    top: [ArchSimd<f32>; NUM_BLOCKS],
    dif: [ArchSimd<f32>; NUM_BLOCKS],
    d_top: [ArchSimd<f32>; NUM_BLOCKS],
    d_dif: [ArchSimd<f32>; NUM_BLOCKS],
    weight: ArchSimd<f32>,
    y_inc_weighted: ArchSimd<f32>,
    y_inc_hi: ArchSimd<f32>,
    y_inc_lo: ArchSimd<f32>,
    z_inc_weighted: ArchSimd<f32>,
    z_inc_hi: ArchSimd<f32>,
    z_inc_lo: ArchSimd<f32>,
}

/// Fills the dst slice with interpolated dot products from gradients.
#[inline(always)]
pub(super) fn grid_dotted_trilerp<const INIT: bool>(
    buffers: &mut DottedTrilerpBuffers,
    config: &TrilerpConfig,
    params: &GridNoiseParams<3>,
    grid_data: &PerlinGridData<3>,
    gradients: &PerlinGradients3D,
    ranges: (Range<usize>, Range<usize>),
    dst: &mut [f32],
) {
    let y_frac_start = unsafe {
        grid_data.distances[1]
            .get_unchecked(ranges.0.start)
            .assume_init()
    };
    let z_frac_start = unsafe {
        grid_data.distances[2]
            .get_unchecked(ranges.1.start)
            .assume_init()
    };

    let mut executer = DottedTrilerpExecuter {
        config,
        params,
        grid_data,
        gradients,
        y_range: ranges.0,
        z_range: ranges.1,
        top: Default::default(),
        dif: Default::default(),
        d_top: Default::default(),
        d_dif: Default::default(),
        weight: ArchSimd::splat(params.weight),
        y_inc_weighted: ArchSimd::splat(grid_data.increment[1] * params.weight),
        y_inc_hi: ArchSimd::splat(y_frac_start),
        y_inc_lo: ArchSimd::splat(y_frac_start - 1.0),
        z_inc_weighted: ArchSimd::splat(grid_data.increment[2] * params.weight),
        z_inc_hi: ArchSimd::splat(z_frac_start),
        z_inc_lo: ArchSimd::splat(z_frac_start - 1.0),
    };

    executer.initialize_trilerp_buffers(grid_data.padded_size[0], buffers);

    if config.has_block_head {
        executer.interpolate::<INIT, false>(buffers, dst);
    }

    // if config.has_block_tail {
    //     executer.interpolate::<INIT, true>(buffers, dst);
    //     std::hint::cold_path();
    // }
}

impl<'a> DottedTrilerpExecuter<'a> {
    #[inline(always)]
    pub fn interpolate<const INIT: bool, const IS_TAIL: bool>(
        &mut self,
        buffers: &DottedTrilerpBuffers,
        dst: &mut [f32],
    ) {
        let range = if IS_TAIL {
            self.config.block_tail_start..self.params.grid_size[0]
        } else {
            0..self.config.block_tail_start
        };
        let buf_size = self.grid_data.padded_size[0];

        let mut z_cur = ArchSimd::splat(0.0);
        let z_hop = self.params.grid_size[0] * self.params.grid_size[1];
        let y_hop = self.params.grid_size[0];
        for z in self.z_range.start..self.z_range.end {
            let z_lerp = unsafe { self.grid_data.fade_factors[2].get_unchecked(z) };
            let z_lerp = unsafe { z_lerp.assume_init() };
            let z_lerp = ArchSimd::splat(z_lerp);

            for x in range.clone().step_by(BLOCK_LANES) {
                self.intialize_factors::<IS_TAIL>(buf_size, buffers, x, z_cur, z_lerp);

                let index = z * z_hop + x;
                let mut y = self.y_range.start;
                while y < self.y_range.end {
                    let index = index + y * y_hop;
                    if y + 4 > self.y_range.end {
                        self.process_factors::<INIT, IS_TAIL>(index, y, dst);
                        y += 1;
                    } else {
                        self.process_factors::<INIT, IS_TAIL>(index, y, dst);
                        self.process_factors::<INIT, IS_TAIL>(index + y_hop, y + 1, dst);
                        self.process_factors::<INIT, IS_TAIL>(index + 2 * y_hop, y + 2, dst);
                        self.process_factors::<INIT, IS_TAIL>(index + 3 * y_hop, y + 3, dst);
                        y += 4;
                    }
                }
            }
            z_cur += ArchSimd::splat(1.0);
        }
    }

    #[inline(always)]
    fn initialize_trilerp_buffers(&mut self, buf_size: usize, buffers: &mut DottedTrilerpBuffers) {
        for x in (0..self.params.grid_size[0]).step_by(LANES) {
            unsafe {
                let x_lerp = self.grid_data.fade_factors[0].load_simd_aligned(x);

                let x_tlf = self.gradients.load_tlf_x(x, buf_size);
                let x_trf = self.gradients.load_trf_x(x, buf_size);
                let x_blf = self.gradients.load_blf_x(x, buf_size);
                let x_brf = self.gradients.load_brf_x(x, buf_size);
                let x_tlb = self.gradients.load_tlb_x(x, buf_size);
                let x_trb = self.gradients.load_trb_x(x, buf_size);
                let x_blb = self.gradients.load_blb_x(x, buf_size);
                let x_brb = self.gradients.load_brb_x(x, buf_size);

                let y_tlf = self.gradients.load_tlf_y(x, buf_size);
                let y_trf = self.gradients.load_trf_y(x, buf_size);
                let y_blf = self.gradients.load_blf_y(x, buf_size);
                let y_brf = self.gradients.load_brf_y(x, buf_size);
                let y_tlb = self.gradients.load_tlb_y(x, buf_size);
                let y_trb = self.gradients.load_trb_y(x, buf_size);
                let y_blb = self.gradients.load_blb_y(x, buf_size);
                let y_brb = self.gradients.load_brb_y(x, buf_size);

                let z_tlf = self.gradients.load_tlf_z(x, buf_size);
                let z_trf = self.gradients.load_trf_z(x, buf_size);
                let z_blf = self.gradients.load_blf_z(x, buf_size);
                let z_brf = self.gradients.load_brf_z(x, buf_size);
                let z_tlb = self.gradients.load_tlb_z(x, buf_size);
                let z_trb = self.gradients.load_trb_z(x, buf_size);
                let z_blb = self.gradients.load_blb_z(x, buf_size);
                let z_brb = self.gradients.load_brb_z(x, buf_size);

                let calc_prod_sum = |z_inc: ArchSimd<f32>, y_inc: ArchSimd<f32>, z, y, x| {
                    z_inc.mul_add(z, y_inc.mul_add(y, x))
                };

                let sum_prod_tlf = calc_prod_sum(self.z_inc_hi, self.y_inc_hi, z_tlf, y_tlf, x_tlf);
                let sum_prod_trf = calc_prod_sum(self.z_inc_hi, self.y_inc_hi, z_trf, y_trf, x_trf);
                let sum_prod_blf = calc_prod_sum(self.z_inc_hi, self.y_inc_lo, z_blf, y_blf, x_blf);
                let sum_prod_brf = calc_prod_sum(self.z_inc_hi, self.y_inc_lo, z_brf, y_brf, x_brf);
                let sum_prod_tlb = calc_prod_sum(self.z_inc_lo, self.y_inc_hi, z_tlb, y_tlb, x_tlb);
                let sum_prod_trb = calc_prod_sum(self.z_inc_lo, self.y_inc_hi, z_trb, y_trb, x_trb);
                let sum_prod_blb = calc_prod_sum(self.z_inc_lo, self.y_inc_lo, z_blb, y_blb, x_blb);
                let sum_prod_brb = calc_prod_sum(self.z_inc_lo, self.y_inc_lo, z_brb, y_brb, x_brb);

                let z_tf_offset = x_lerp.mul_add(z_trf - z_tlf, z_tlf) * self.z_inc_weighted;
                let z_bf_offset = x_lerp.mul_add(z_brf - z_blf, z_blf) * self.z_inc_weighted;
                let z_tb_offset = x_lerp.mul_add(z_trb - z_tlb, z_tlb) * self.z_inc_weighted;
                let z_bb_offset = x_lerp.mul_add(z_brb - z_blb, z_blb) * self.z_inc_weighted;

                let y_tf_offset = x_lerp.mul_add(y_trf - y_tlf, y_tlf) * self.y_inc_weighted;
                let y_bf_offset = x_lerp.mul_add(y_brf - y_blf, y_blf) * self.y_inc_weighted;
                let y_hi_offset_dif = x_lerp
                    .mul_add(y_trb - y_tlb, y_tlb)
                    .mul_sub(self.y_inc_weighted, y_tf_offset);
                let y_lo_offset_dif = x_lerp
                    .mul_add(y_brb - y_blb, y_blb)
                    .mul_sub(self.y_inc_weighted, y_bf_offset);

                let tf_base =
                    x_lerp.mul_add(sum_prod_trf - sum_prod_tlf, sum_prod_tlf) * self.weight;
                let bf_base =
                    x_lerp.mul_add(sum_prod_brf - sum_prod_blf, sum_prod_blf) * self.weight;
                let hi_base_dif = x_lerp
                    .mul_add(sum_prod_trb - sum_prod_tlb, sum_prod_tlb)
                    .mul_sub(self.weight, tf_base);
                let lo_base_dif = x_lerp
                    .mul_add(sum_prod_brb - sum_prod_blb, sum_prod_blb)
                    .mul_sub(self.weight, bf_base);

                buffers.write_z_tf_offset(x, buf_size, z_tf_offset);
                buffers.write_z_bf_offset(x, buf_size, z_bf_offset);
                buffers.write_z_top_offset_dif(x, buf_size, z_tb_offset - z_tf_offset);
                buffers.write_z_bottom_offset_dif(x, buf_size, z_bb_offset - z_bf_offset);

                buffers.write_y_tf_offset(x, buf_size, y_tf_offset);
                buffers.write_y_bf_offset(x, buf_size, y_bf_offset);
                buffers.write_y_top_offset_dif(x, buf_size, y_hi_offset_dif);
                buffers.write_y_bottom_offset_dif(x, buf_size, y_lo_offset_dif);

                buffers.write_tf_base(x, buf_size, tf_base);
                buffers.write_bf_base(x, buf_size, bf_base);
                buffers.write_top_base_dif(x, buf_size, hi_base_dif);
                buffers.write_bottom_base_dif(x, buf_size, lo_base_dif);
            }
        }
    }

    #[inline(always)]
    fn intialize_factors<const IS_TAIL: bool>(
        &mut self,
        buf_size: usize,
        buffers: &DottedTrilerpBuffers,
        x: usize,
        z_vec: ArchSimd<f32>,
        z_lerp: ArchSimd<f32>,
    ) {
        let num_blocks = if IS_TAIL {
            self.config.block_tail_size
        } else {
            NUM_BLOCKS
        };

        // These blocked loops will get entirely unrolled by the compiler.
        for block in 0..num_blocks {
            // Load gradients into registers.
            unsafe {
                let index = x + LANES * block;

                let z_tf_offset = buffers.load_z_tf_offset(index, buf_size);
                let z_bf_offset = buffers.load_z_bf_offset(index, buf_size);
                let z_top_offset_dif = buffers.load_z_top_offset_dif(index, buf_size);
                let z_bottom_offset_dif = buffers.load_z_bottom_offset_dif(index, buf_size);

                let y_tf_offset = buffers.load_y_tf_offset(index, buf_size);
                let y_bf_offset = buffers.load_y_bf_offset(index, buf_size);
                let y_top_offset_dif = buffers.load_y_top_offset_dif(index, buf_size);
                let y_bottom_offset_dif = buffers.load_y_bottom_offset_dif(index, buf_size);

                let tf_base_vec = buffers.load_tf_base(index, buf_size);
                let bf_base_vec = buffers.load_bf_base(index, buf_size);
                let top_base_dif_vec = buffers.load_top_base_dif(index, buf_size);
                let bottom_base_dif_vec = buffers.load_bottom_base_dif(index, buf_size);

                let z_top_offset = z_lerp.mul_add(z_top_offset_dif, z_tf_offset);
                let z_bottom_offset = z_lerp.mul_add(z_bottom_offset_dif, z_bf_offset);

                self.top[block] =
                    z_vec.mul_add(z_top_offset, z_lerp.mul_add(top_base_dif_vec, tf_base_vec));
                let bottom_base = z_vec.mul_add(
                    z_bottom_offset,
                    z_lerp.mul_add(bottom_base_dif_vec, bf_base_vec),
                );
                self.dif[block] = bottom_base - self.top[block];

                self.d_top[block] = z_lerp.mul_add(y_top_offset_dif, y_tf_offset);
                let y_bottom_offset = z_lerp.mul_add(y_bottom_offset_dif, y_bf_offset);
                self.d_dif[block] = y_bottom_offset - self.d_top[block];
            }
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

            let slice = unsafe { dst.get_unchecked_mut(index..) };
            if IS_TAIL {
                val.copy_to_slice(slice);
            } else {
                unsafe { val.copy_to_slice_unchecked(slice) };
            };

            self.dif[block] += self.d_dif[block];
            self.top[block] += self.d_top[block];
        }
    }
}
