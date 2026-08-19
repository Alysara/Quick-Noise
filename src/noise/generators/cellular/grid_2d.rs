use std::mem::MaybeUninit;
use std::ops::Range;

use simply_simd::{ Arch, Simd, enable_targets };

use crate::api::grid::interface::GridNoiseParams;
use crate::noise::combiners::{ Combiner, CombinerState };
use crate::noise::util::grid_data::{ GridData, Lerp };
use crate::noise::util::grid_helpers::{
    Arena,
    ArenaBuffer,
    MaybeUninitSliceSimdExt,
    maybe_tail_load,
    maybe_tail_store,
    pad_grid_size,
    validate_grid_size,
    validate_state_size,
};
use crate::{ Cellular, GridGenerator };

const LERP: u8 = Lerp::Quintic as u8;

const RING: [(i32, i32); 8] = [
    (-1, 0), // ttl
    (0, -1), // tll
    (-1, 1), // ttr
    (0, 2), // trr
    (2, 0), // bbl
    (1, -1), // bll
    (2, 1), // bbr
    (1, 2), // brr
];

const BYTE_SHUFFLE: [u8; 64] = [
    3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12,
    14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8,
    10, 9, 15, 12, 14, 13,
];

struct RowWindow {
    top: *mut f32,
    bot: *mut f32,
    width: usize,
}

impl RowWindow {
    fn new(arena: &mut Arena, width: usize) -> Self {
        Self {
            top: arena
                .allocate::<f32>(width * 2)
                .as_mut_ptr()
                .cast(),
            bot: arena
                .allocate::<f32>(width * 2)
                .as_mut_ptr()
                .cast(),
            width,
        }
    }

    fn top(&self) -> &[(f32, f32)] {
        unsafe { std::slice::from_raw_parts(self.top.cast::<(f32, f32)>(), self.width) }
    }

    fn bot(&self) -> &[(f32, f32)] {
        unsafe { std::slice::from_raw_parts(self.bot.cast::<(f32, f32)>(), self.width) }
    }

    fn top_mut(&mut self) -> &mut [(f32, f32)] {
        unsafe { std::slice::from_raw_parts_mut(self.top.cast::<(f32, f32)>(), self.width) }
    }

    fn bot_mut(&mut self) -> &mut [(f32, f32)] {
        unsafe { std::slice::from_raw_parts_mut(self.bot.cast::<(f32, f32)>(), self.width) }
    }

    fn fill_row<A: Arch>(
        params: &GridNoiseParams<2>,
        grid_data: &GridData<2>,
        buff: &mut [(f32, f32)],
        cy: i32
    ) {
        let cy = grid_data.octave_tiling[1].map_or(cy, |t| cy.rem_euclid(t as i32));
        let y_shuf = hash_cell_y::<A>(cy as u32, params.seed);
        let y_shuf_v = Simd::<u32, A>::splat(y_shuf);
        let lanes = Simd::<f32, A>::LANES;
        let cx_start = grid_data.grid_start[0];

        let mut x_it = 0;
        while x_it + lanes <= buff.len() {
            let cx_base = cx_start.wrapping_add(x_it as i32);
            let cx_v = Simd::<i32, A>::splat(cx_base) + Simd::<i32, A>::iota(0);
            let cx_v = if let Some(t) = grid_data.octave_tiling[0] {
                simd_rem_euclid_i32::<A>(cx_v, t as i32)
            } else {
                cx_v
            };
            let hashes = hash_cells_row::<A>(cx_v.raw_cast(), y_shuf_v, params.seed);
            let (tx, ty) = split_hash_batch::<A>(hashes);
            let tx_arr = tx.to_array();
            let ty_arr = ty.to_array();
            for i in 0..lanes {
                buff[x_it + i] = (tx_arr[i], ty_arr[i]);
            }
            x_it += lanes;
        }
        for i in x_it..buff.len() {
            let cx = cx_start.wrapping_add(i as i32);
            let cx = grid_data.octave_tiling[0].map_or(cx, |t| cx.rem_euclid(t as i32));
            buff[i] = split_hash(hash_cell_with_y::<A>(cx as u32, y_shuf, params.seed));
        }
    }

    #[inline(always)]
    fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.top, &mut self.bot);
    }
}

#[inline(always)]
pub(super) fn hash_cell<A: Arch>(x: u32, y: u32, seed: u32) -> u32 {
    hash_cell_with_y::<A>(x, hash_cell_y::<A>(y, seed), seed)
}

/// Hashes `LANES` consecutive lattice columns from a pre-built `cx_v` vector
/// at fixed y in one shot
#[inline(always)]
pub(super) fn hash_cells_row<A: Arch>(
    cx_v: Simd<u32, A>,
    y_shuf: Simd<u32, A>,
    seed: u32
) -> Simd<u32, A> {
    let shuffle_indices = unsafe { Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);
    let seed_v = Simd::<u32, A>::splat(seed);

    let x_shuf = (cx_v * seed_v).permute_8(shuffle_indices) ^ prime;
    (x_shuf * y_shuf) ^ x_shuf
}

pub(super) fn hash_cell_y<A: Arch>(y: u32, seed: u32) -> u32 {
    let shuffle_indices = unsafe { Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);
    (Simd::<u32, A>::splat(y.wrapping_mul(seed)).permute_8(shuffle_indices) ^ prime).to_array()[0]
}

#[inline(always)]
pub(super) fn hash_cell_with_y<A: Arch>(x: u32, y_shuf: u32, seed: u32) -> u32 {
    let shuffle_indices = unsafe { Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);
    let x_shuf = (
        Simd::<u32, A>::splat(x.wrapping_mul(seed)).permute_8(shuffle_indices) ^ prime
    ).to_array()[0];
    x_shuf.wrapping_mul(y_shuf) ^ x_shuf
}

/// Split a hash into the per-axis jitter offsets. The lower 23 bits
/// become the x-mantissa and the next 23 bits become the y-mantissa, giving
/// unform values in the range `[1, 2)`
#[inline(always)]
pub(super) fn split_hash(hash: u32) -> (f32, f32) {
    let exp_bits = 0x3f800000;
    let hash_mask = 0x007fffff;
    let tx = 1.5 - f32::from_bits((hash & hash_mask) | exp_bits);
    let ty = 1.5 - f32::from_bits((hash >> 9) | exp_bits);
    (tx, ty)
}

#[inline(always)]
pub(super) fn split_hash_batch<A: Arch>(hash: Simd<u32, A>) -> (Simd<f32, A>, Simd<f32, A>) {
    let exp_bits = Simd::<u32, A>::splat(0x3f800000);
    let hash_mask = Simd::<u32, A>::splat(0x007fffff);
    let one_halves = Simd::<f32, A>::splat(1.5);

    let tx = one_halves - ((hash & hash_mask) | exp_bits).raw_cast::<f32>();
    let ty = one_halves - ((hash >> Simd::<u32, A>::splat(9)) | exp_bits).raw_cast::<f32>();
    (tx, ty)
}

#[inline(always)]
fn simd_rem_euclid_i32<A: Arch>(x: Simd<i32, A>, t: i32) -> Simd<i32, A> {
    let t_f = Simd::<f32, A>::splat(t as f32);
    let x_f = x.cast_float();
    (x_f - (x_f / t_f).floor() * t_f).cast_int_trunc()
}

/// Jitter offsets for the 4 base candidates (corners of the gradient cell).
/// Arrays are arena-allocated and reused across cells.
struct BaseJitters {
    x_parts: *mut MaybeUninit<f32>,
    y_parts: *mut MaybeUninit<f32>,
}

impl BaseJitters {
    fn new(arena: &mut Arena) -> Self {
        Self {
            x_parts: arena.allocate(4).as_mut_ptr(),
            y_parts: arena.allocate(4).as_mut_ptr(),
        }
    }

    #[inline(always)]
    fn write(&mut self, top: &[(f32, f32)], bot: &[(f32, f32)], x_it: usize) {
        let (tx0, ty0) = top[x_it];
        let (tx1, ty1) = top[x_it + 1];
        let (tx2, ty2) = bot[x_it];
        let (tx3, ty3) = bot[x_it + 1];
        unsafe {
            self.x_parts.add(0).write(MaybeUninit::new(tx0));
            self.x_parts.add(1).write(MaybeUninit::new(tx1 + 1.0));
            self.x_parts.add(2).write(MaybeUninit::new(tx2));
            self.x_parts.add(3).write(MaybeUninit::new(tx3 + 1.0));
            self.y_parts.add(0).write(MaybeUninit::new(ty0));
            self.y_parts.add(1).write(MaybeUninit::new(ty1));
            self.y_parts.add(2).write(MaybeUninit::new(ty2 + 1.0));
            self.y_parts.add(3).write(MaybeUninit::new(ty3 + 1.0));
        }
    }
}

/// Jitter offsets for the 8 ring neighbors.
/// Arrays are arena-allocated and reused across cells.
struct RingJitters {
    x_parts: *mut MaybeUninit<f32>,
    y_parts: *mut MaybeUninit<f32>,
}

impl RingJitters {
    fn new(arena: &mut Arena) -> Self {
        Self {
            x_parts: arena.allocate(8).as_mut_ptr(),
            y_parts: arena.allocate(8).as_mut_ptr(),
        }
    }

    #[inline(always)]
    fn write<A: Arch>(
        &mut self,
        params: &GridNoiseParams<2>,
        grid_data: &GridData<2>,
        x_it: usize,
        y_it: usize
    ) {
        let cx = grid_data.grid_start[0] + (x_it as i32);
        let cy = grid_data.grid_start[1] + (y_it as i32);
        let tile_x = |x: i32| grid_data.octave_tiling[0].map_or(x, |t| x.rem_euclid(t as i32));
        let tile_y = |y: i32| grid_data.octave_tiling[1].map_or(y, |t| y.rem_euclid(t as i32));

        for (i, &(ox, oy)) in RING.iter().enumerate() {
            let (jx_i, jy_i) = split_hash(
                hash_cell::<A>(tile_x(cx + ox) as u32, tile_y(cy + oy) as u32, params.seed)
            );
            unsafe {
                self.x_parts.add(i).write(MaybeUninit::new(jx_i + (ox as f32)));
                self.y_parts.add(i).write(MaybeUninit::new(jy_i + (oy as f32)));
            }
        }
    }
}

#[enable_targets(A)]
impl GridGenerator<2> for Cellular {
    fn sample_grid<A: Arch, C: Combiner, const INIT: bool, const FINAL: bool>(
        params: GridNoiseParams<2>,
        combiner: C::Config,
        state: &mut [f32],
        dst: &mut [f32]
    ) {
        validate_grid_size(params.grid_size, dst.len());
        validate_state_size::<C, A, _>(params.grid_size, state.len());
        let padded_size = pad_grid_size::<A, _>(params.grid_size);
        let total_size = params.grid_size.iter().product::<usize>();

        let required_cache =
            padded_size[1] * 3 + padded_size[0] * 3 + total_size
             + (padded_size[0] + 1) * 4 + 24; // +24 for base/ring jitter scratch
        let mut cache = ArenaBuffer::<A>::with_capacity(required_cache);
        let mut arena = Arena::with_cache(&mut cache);
        let mut sub_arena = arena.allocate_arena(padded_size[0] * 3 + padded_size[1] * 3);

        let grid_data = GridData::new::<A, LERP>(&params, &mut sub_arena, &padded_size);

        // Scratch buffer for the raw cellular min and combiner pass
        let raw = unsafe { arena.allocate(total_size).assume_init_mut() };

        // per-cell setup pass, driven by the cell x/y indices
        let mut window = RowWindow::new(&mut arena, grid_data.num_loops[0] + 1);
        let mut base_jitters = BaseJitters::new(&mut arena);
        let mut ring_jitters = RingJitters::new(&mut arena);
        // Hash top row and cache
        RowWindow::fill_row::<A>(&params, &grid_data, window.top_mut(), grid_data.grid_start[1]);

        let mut y_idx = 0;
        for y_it in 0..grid_data.num_loops[1] {
            let y_next_idx = unsafe {
                grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize
            };
            let y_sample_range = y_idx..y_next_idx;
            // Hash bottom row and cache
            RowWindow::fill_row::<A>(
                &params,
                &grid_data,
                window.bot_mut(),
                grid_data.grid_start[1] + (y_it as i32) + 1
            );

            let mut x_idx = 0;
            for x_it in 0..grid_data.num_loops[0] {
                let x_next_idx = unsafe {
                    grid_data.grid_indices[0].get_unchecked(x_it).assume_init() as usize
                };
                let x_sample_range = x_idx..x_next_idx;

                base_jitters.write(window.top(), window.bot(), x_it);

                let any_far = grid_cellular_fill_base::<A>(
                    &grid_data,
                    &base_jitters,
                    x_idx,
                    x_next_idx,
                    y_idx,
                    y_next_idx,
                    raw
                );

                if any_far {
                    ring_jitters.write::<A>(&params, &grid_data, x_it, y_it);
                    grid_cellular_fill_ring::<A>(
                        &grid_data,
                        &ring_jitters,
                        x_idx,
                        x_next_idx,
                        y_idx,
                        y_next_idx,
                        raw
                    );
                }
                    
                // Apply the combiner
                grid_cellular_combine::<A, C, INIT, FINAL>(
                    &grid_data,
                    raw,
                    dst,
                    state,
                    &x_sample_range,
                    &y_sample_range,
                    &combiner
                );

                x_idx = unsafe {
                    grid_data.grid_indices[0].get_unchecked(x_it).assume_init() as usize
                };
            }
            // Reuse caches
            window.swap_top_bottom();

            y_idx = unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize };
        }
    }
}

/// Computes (sx - tx)^2 and (sy - ty)^2 inline.
/// Writes squared distances to `raw`. Returns true if any sample exceeded
/// the edge-distance threshold (meaning the 8-cell ring must be checked).
#[inline(always)]
fn grid_cellular_fill_base<A: Arch>(
    grid_data: &GridData<2>,
    jit: &BaseJitters,
    x_idx: usize,
    x_next: usize,
    y_idx: usize,
    y_next: usize,
    raw: &mut [f32]
) -> bool {
    let lanes = Simd::<f32, A>::LANES;
    let row_width = grid_data.grid_size[0];

    let tx0v = Simd::<f32, A>::splat(unsafe { (*jit.x_parts.add(0)).assume_init() });
    let tx1v = Simd::<f32, A>::splat(unsafe { (*jit.x_parts.add(1)).assume_init() });
    let tx2v = Simd::<f32, A>::splat(unsafe { (*jit.x_parts.add(2)).assume_init() });
    let tx3v = Simd::<f32, A>::splat(unsafe { (*jit.x_parts.add(3)).assume_init() });

    let mut any_far = false;

    for y in y_idx..y_next {
        let sy = unsafe { grid_data.distances[1].get_unchecked(y).assume_init() };
        let yp0 = Simd::<f32, A>::splat({
            let d = sy - (unsafe { (*jit.y_parts.add(0)).assume_init() });
            d * d
        });
        let yp1 = Simd::<f32, A>::splat({
            let d = sy - (unsafe { (*jit.y_parts.add(1)).assume_init() });
            d * d
        });
        let yp2 = Simd::<f32, A>::splat({
            let d = sy - (unsafe { (*jit.y_parts.add(2)).assume_init() });
            d * d
        });
        let yp3 = Simd::<f32, A>::splat({
            let d = sy - (unsafe { (*jit.y_parts.add(3)).assume_init() });
            d * d
        });

        // Threshold
        let y_edge_lo = Simd::<f32, A>::splat(sy + 0.5);
        let y_edge_hi = Simd::<f32, A>::splat(1.5 - sy);

        let row_start = y * row_width;
        let row_end = row_start + row_width;
        let mut index = x_idx;

        while index < x_next {
            let sx = unsafe { grid_data.distances[0].load_simd(index) };
            let xp0 = {
                let d = sx - tx0v;
                d * d
            };
            let xp1 = {
                let d = sx - tx1v;
                d * d
            };
            let xp2 = {
                let d = sx - tx2v;
                d * d
            };
            let xp3 = {
                let d = sx - tx3v;
                d * d
            };

            let x_edge_lo = sx + Simd::<f32, A>::splat(0.5);
            let x_edge_hi = Simd::<f32, A>::splat(1.5) - sx;
            let closest_edge = x_edge_lo.min(y_edge_lo).min(x_edge_hi.min(y_edge_hi));
            let threshold = closest_edge * closest_edge;

            // Squared distance to closest of 4 candidates
            let dist_sq = (xp0 + yp0)
                .min(xp1 + yp1)
                .min(xp2 + yp2)
                .min(xp3 + yp3);

            any_far |= dist_sq.simd_gt(threshold).to_bits() != 0;

            dist_sq.copy_to_slice(
                &mut raw[row_start + index..(row_start + index + lanes).min(row_end)]
            );
            index += lanes;
        }
    }

    any_far
}

/// Computes all 8 ring candidate distances inline,
/// takes min over ring, then min with the base min already in `raw`.
#[inline(always)]
fn grid_cellular_fill_ring<A: Arch>(
    grid_data: &GridData<2>,
    ring: &RingJitters,
    x_idx: usize,
    x_next: usize,
    y_idx: usize,
    y_next: usize,
    raw: &mut [f32]
) {
    let lanes = Simd::<f32, A>::LANES;
    let row_width = grid_data.grid_size[0];

    let ring_jx0 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(0)).assume_init() });
    let ring_jx1 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(1)).assume_init() });
    let ring_jx2 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(2)).assume_init() });
    let ring_jx3 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(3)).assume_init() });
    let ring_jx4 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(4)).assume_init() });
    let ring_jx5 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(5)).assume_init() });
    let ring_jx6 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(6)).assume_init() });
    let ring_jx7 = Simd::<f32, A>::splat(unsafe { (*ring.x_parts.add(7)).assume_init() });

    for y in y_idx..y_next {
        let sy = unsafe { grid_data.distances[1].get_unchecked(y).assume_init() };

        let d0 = sy - (unsafe { (*ring.y_parts.add(0)).assume_init() });
        let ring_yp0 = Simd::<f32, A>::splat(d0 * d0);
        let d1 = sy - (unsafe { (*ring.y_parts.add(1)).assume_init() });
        let ring_yp1 = Simd::<f32, A>::splat(d1 * d1);
        let d2 = sy - (unsafe { (*ring.y_parts.add(2)).assume_init() });
        let ring_yp2 = Simd::<f32, A>::splat(d2 * d2);
        let d3 = sy - (unsafe { (*ring.y_parts.add(3)).assume_init() });
        let ring_yp3 = Simd::<f32, A>::splat(d3 * d3);
        let d4 = sy - (unsafe { (*ring.y_parts.add(4)).assume_init() });
        let ring_yp4 = Simd::<f32, A>::splat(d4 * d4);
        let d5 = sy - (unsafe { (*ring.y_parts.add(5)).assume_init() });
        let ring_yp5 = Simd::<f32, A>::splat(d5 * d5);
        let d6 = sy - (unsafe { (*ring.y_parts.add(6)).assume_init() });
        let ring_yp6 = Simd::<f32, A>::splat(d6 * d6);
        let d7 = sy - (unsafe { (*ring.y_parts.add(7)).assume_init() });
        let ring_yp7 = Simd::<f32, A>::splat(d7 * d7);

        let row_start = y * row_width;
        let row_end = row_start + row_width;
        let mut index = x_idx;

        while index < x_next {
            let sx = unsafe { grid_data.distances[0].load_simd(index) };

            let d0 = sx - ring_jx0;
            let xp0 = d0 * d0;
            let d1 = sx - ring_jx1;
            let xp1 = d1 * d1;
            let d2 = sx - ring_jx2;
            let xp2 = d2 * d2;
            let d3 = sx - ring_jx3;
            let xp3 = d3 * d3;
            let d4 = sx - ring_jx4;
            let xp4 = d4 * d4;
            let d5 = sx - ring_jx5;
            let xp5 = d5 * d5;
            let d6 = sx - ring_jx6;
            let xp6 = d6 * d6;
            let d7 = sx - ring_jx7;
            let xp7 = d7 * d7;

            let ring_min_sq = (xp0 + ring_yp0)
                .min(xp1 + ring_yp1)
                .min(xp2 + ring_yp2)
                .min(xp3 + ring_yp3)
                .min(xp4 + ring_yp4)
                .min(xp5 + ring_yp5)
                .min(xp6 + ring_yp6)
                .min(xp7 + ring_yp7);

            let existing: Simd<f32, A> = Simd::<f32, A>::from_slice(
                &raw[row_start + index..(row_start + index + lanes).min(row_end)]
            );

            ring_min_sq
                .min(existing)
                .copy_to_slice(
                    &mut raw[row_start + index..(row_start + index + lanes).min(row_end)]
                );
            index += lanes;
        }
    }
}

/// Reads the finished raw cellular min from `raw`, combines
/// it with the accumulated value in `dst` (from previous octaves), and
/// writes the result back to `dst`
#[inline(always)]
pub(super) fn grid_cellular_combine<A: Arch, C: Combiner, const INIT: bool, const FINAL: bool>(
    grid_data: &GridData<2>,
    raw: &[f32],
    dst: &mut [f32],
    state: &mut [f32],
    x_range: &Range<usize>,
    y_range: &Range<usize>,
    combiner_config: &C::Config
) {
    let lanes = Simd::<f32, A>::LANES;
    let row_width = grid_data.grid_size[0];

    for y in y_range.clone() {
        let row_start = y * row_width;
        let mut index = x_range.start;

        while index + lanes <= x_range.end {
            let sample_start = row_start + index;
            grid_cellular_combine_block::<A, C, INIT, FINAL, false>(
                grid_data,
                raw,
                dst,
                state,
                sample_start,
                lanes,
                combiner_config
            );
            index += lanes;
        }
        if index < x_range.end {
            let sample_start = row_start + index;
            let tail_len = x_range.end - index;
            grid_cellular_combine_block::<A, C, INIT, FINAL, true>(
                grid_data,
                raw,
                dst,
                state,
                sample_start,
                tail_len,
                combiner_config
            );
        }
    }
}

#[inline(always)]
fn grid_cellular_combine_block<
    A: Arch,
    C: Combiner,
    const INIT: bool,
    const FINAL: bool,
    const IS_TAIL: bool
>(
    grid_data: &GridData<2>,
    raw: &[f32],
    dst: &mut [f32],
    state: &mut [f32],
    sample_start: usize,
    len: usize,
    combiner_config: &C::Config
) {
    let sample_end = sample_start + len;

    let raw_val: Simd<f32, A> = (unsafe {
        maybe_tail_load::<A, IS_TAIL>(sample_start..sample_end, raw).sqrt()
    }) * Simd::<f32, A>::splat(grid_data.weight);

    let (cur_state, mut result) = if INIT {
        C::initialize_sample(combiner_config, raw_val)
    } else {
        let mut cur_state = C::State::<A>::default();
        for i in 0..C::State::<A>::STATE_SIZE {
            let offset = i * grid_data.total_size;
            cur_state[i] = unsafe {
                maybe_tail_load::<A, IS_TAIL>(sample_start + offset..sample_end + offset, state)
            };
        }
        let cur_result = unsafe { maybe_tail_load::<A, IS_TAIL>(sample_start..sample_end, dst) };
        C::apply_sample(combiner_config, cur_state, cur_result, raw_val)
    };

    if !FINAL {
        for i in 0..C::State::<A>::STATE_SIZE {
            let offset = i * grid_data.total_size;
            unsafe {
                maybe_tail_store::<A, IS_TAIL>(
                    sample_start + offset..sample_end + offset,
                    cur_state[i],
                    state
                );
            }
        }
    }

    if FINAL {
        result = C::finalize_sample(combiner_config, cur_state, result);
    }

    unsafe {
        maybe_tail_store::<A, IS_TAIL>(sample_start..sample_end, result, dst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::seed::gen_octave_seed;
    use crate::math::random::Random;
    use crate::simd::StaticArch;
    use crate::{ Cellular, Fbm, Grid };

    #[test]
    fn ring_covers_all_neighbors() {
        let expected = [
            (-1, 0),
            (0, -1),
            (-1, 1),
            (0, 2),
            (2, 0),
            (1, -1),
            (2, 1),
            (1, 2),
        ];
        assert_eq!(RING, expected, "RING must stay in sync with the batch ring");
    }

    #[test]
    fn cellular_grid_2d_sanity() {
        let grid = Grid::<2>::new(32, 32);
        let mut result = [0.0; 1024];
        grid.builder::<Fbm, Cellular>().fill(result.as_mut_slice());
        verify_slice(result.as_slice());
    }

    #[test]
    fn cellular_grid_2d_reference() {
        const W: usize = 64;
        const H: usize = 64;
        const FREQ: f32 = 1.0 / 32.0;
        let seed = 123456789i64;

        let grid = Grid::<2>::new(W, H).seed(seed).sample_position(-5, 3);
        let grid_seed = Random::mix_u64(seed as u64);
        let base_seed = Random::mix_u64_pair(grid_seed, 0xd5e7b3c94f8a1e6b);
        let octave_seed = gen_octave_seed([FREQ, FREQ], base_seed);

        let mut result = [0.0; W * H];
        grid.builder::<Fbm, Cellular>().frequency(FREQ).fill(result.as_mut_slice());

        let mut max_diff = 0.0f32;
        for y in 0..H {
            for x in 0..W {
                let px = (-5.0 + (x as f32)) * FREQ;
                let py = (3.0 + (y as f32)) * FREQ;
                let reference = reference_cellular(octave_seed, px, py);
                let actual = result[y * W + x];
                max_diff = max_diff.max((actual - reference).abs());
            }
        }
        assert!(
            max_diff < 1e-4,
            "Grid cellular diverges from the brute-force Cellular by {max_diff}"
        );

        // Non-square (tall) grid, which must size its candidate buffers for the
        // larger y axis.
        let grid = Grid::<2>::new(32, 96).seed(seed).sample_position(-5, 3);
        let mut result = [0.0; 32 * 96];
        grid.builder::<Fbm, Cellular>().frequency(FREQ).fill(result.as_mut_slice());

        let mut max_diff = 0.0f32;
        for y in 0..96 {
            for x in 0..32 {
                let px = (-5.0 + (x as f32)) * FREQ;
                let py = (3.0 + (y as f32)) * FREQ;
                let reference = reference_cellular(octave_seed, px, py);
                let actual = result[y * 32 + x];
                max_diff = max_diff.max((actual - reference).abs());
            }
        }
        assert!(
            max_diff < 1e-4,
            "Tall grid cellular diverges from the brute-force Cellular by {max_diff}"
        );
    }

    fn reference_cellular(seed: u32, px: f32, py: f32) -> f32 {
        let cell_x = px.floor() as i32;
        let cell_y = py.floor() as i32;
        let sx = px - px.floor();
        let sy = py - py.floor();

        let mut min_dist = f32::MAX;
        for ox in -3..=3 {
            for oy in -3..=3 {
                let (jx, jy) = split_hash(
                    hash_cell::<StaticArch>((cell_x + ox) as u32, (cell_y + oy) as u32, seed)
                );
                let dx = sx - ((ox as f32) + jx);
                let dy = sy - ((oy as f32) + jy);
                min_dist = min_dist.min(dx * dx + dy * dy);
            }
        }
        min_dist.sqrt()
    }

    #[test]
    #[cfg(feature = "image")]
    fn cellular_grid_2d_image() {
        use crate::{ Billow, HybridMulti, Multi, PingPong, Ridged, Terrace };
        use crate::emit::NoiseImageExt;
        use crate::simd::StaticSimd;

        let grid = Grid::<2>::new(256, 256).seed(42).sample_position(-128, -128);

        grid.builder::<Fbm, Cellular>()
            .frequency(1.0 / 32.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d.png");

        grid.builder::<Fbm, Cellular>()
            .octaves(2)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_fbm.png");

        grid.builder::<PingPong, Cellular>()
            .octaves(2)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_ping_pong.png");

        grid.builder::<Ridged, Cellular>()
            .octaves(2)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_ridged.png");

        grid.builder::<Billow, Cellular>()
            .octaves(2)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_billow.png");

        grid.builder::<HybridMulti, Cellular>()
            .octaves(1)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_hybrid_multi.png");

        grid.builder::<Multi, Cellular>()
            .octaves(1)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_multi.png");

        grid.builder::<Terrace, Cellular>()
            .octaves(1)
            .frequency(1.0 / 64.0)
            .into_iter()
            .map(|x| x * StaticSimd::splat(1.4) - StaticSimd::splat(1.0))
            .to_grayscale_image(256, 256, "test_images/cellular_grid_2d_terrace.png");

        let grid_2d_tiled = Grid::<2>::new(1024, 1024).tiling(Some(128), Some(256));

        grid_2d_tiled
            .builder::<Fbm, Cellular>()
            .octaves(4)
            .frequency(1.0 / 64.0)
            .into_iter()
            .to_grayscale_image(1024, 1024, "test_images/grid_2d_cellular_tiled.png");
    }

    fn verify_slice(slice: &[f32]) {
        let mut min = f32::MAX;
        let mut max = f32::NEG_INFINITY;
        let mut dif_total = 0.0;
        let mut prev = slice[0];

        for val in slice.iter() {
            min = val.min(min);
            max = val.max(max);

            let dif = (*val - prev).abs();
            dif_total += dif;
            prev = *val;
        }

        assert!(min >= 0.0, "Cellular distance of {min} was negative!");
        assert!(max < 10.0, "Maximum value of {max} was above 10!");
        assert!(dif_total > 0.0, "Output is constant of {}!", slice[0]);
    }
}
