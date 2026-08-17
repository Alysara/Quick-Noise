use std::fmt;
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
    assume_init_slice,
    maybe_tail_load,
    maybe_tail_store,
    pad_grid_size,
    validate_grid_size,
    validate_state_size,
};
use crate::{ Cellular, GridGenerator };

struct CellularCandidates2D<'a> {
    xpart: [&'a mut [MaybeUninit<f32>]; 12],
    ypart: [&'a mut [MaybeUninit<f32>]; 12],
}

impl<'a> CellularCandidates2D<'a> {
    #[inline(always)]
    pub fn new(arena: &'a mut Arena, size: usize) -> Self {
        Self {
            xpart: std::array::from_fn(|_| arena.allocate(size)),
            ypart: std::array::from_fn(|_| arena.allocate(size)),
        }
    }
}

impl<'a> fmt::Debug for CellularCandidates2D<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            let mut dbg = f.debug_struct("CellularCandidates2D");
            for (i, part) in self.xpart.iter().enumerate() {
                dbg.field(&format!("xpart.{i}"), &assume_init_slice(part));
            }
            for (i, part) in self.ypart.iter().enumerate() {
                dbg.field(&format!("ypart.{i}"), &assume_init_slice(part));
            }
            dbg.finish()
        }
    }
}

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

        // Both the x and y candidate buffers must fit the larger axis.
        let candidate_size = padded_size[0].max(padded_size[1]);
        let required_cache =
            padded_size[1] * 3 + padded_size[0] * 3 + candidate_size * 24 + total_size;
        let mut cache = ArenaBuffer::<A>::with_capacity(required_cache);
        let mut arena = Arena::with_cache(&mut cache);

        let mut sub_arena = arena.allocate_arena(padded_size[0] * 3 + padded_size[1] * 3);

        let mut grid_data = GridData::new::<A, LERP>(&params, &mut sub_arena, &padded_size);

        // Scratch buffer for the raw cellular min and combiner pass
        let raw = unsafe { arena.allocate(total_size).assume_init_mut() };

        let mut candidates = CellularCandidates2D::new(&mut arena, candidate_size);

        // per-cell setup pass, driven by the cell x/y indices
        let mut y_idx = 0;
        for y_it in 0..grid_data.num_loops[1] {
            let y_next_idx = unsafe {
                grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize
            };
            let mut x_idx = 0;
            for x_it in 0..grid_data.num_loops[0] {
                let x_next_idx = unsafe {
                    grid_data.grid_indices[0].get_unchecked(x_it).assume_init() as usize
                };

                grid_cellular_hash::<A>(
                    &params,
                    &mut grid_data,
                    &mut candidates.xpart,
                    &mut candidates.ypart,
                    x_it,
                    x_idx,
                    y_it,
                    y_idx
                );

                let x_sample_range = x_idx..x_next_idx;
                let y_sample_range = y_idx..y_next_idx;

                let any_far = grid_cellular_fill::<A>(
                    &mut grid_data,
                    &mut candidates.xpart,
                    &mut candidates.ypart,
                    &x_sample_range,
                    &y_sample_range,
                    raw
                );

                // Only hash the 8-cell ring if any sample in this cell tripped the threshold
                if any_far {
                    grid_cellular_hash_ring::<A>(
                        &params,
                        &mut grid_data,
                        &mut candidates.xpart,
                        &mut candidates.ypart,
                        x_it,
                        x_idx,
                        y_it,
                        y_idx
                    );

                    grid_cellular_fill_ring::<A>(
                        &mut grid_data,
                        &mut candidates.xpart,
                        &mut candidates.ypart,
                        &x_sample_range,
                        &y_sample_range,
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
            y_idx = unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize };
        }
    }
}

/// Hash of a single cell corner
#[inline(always)]
pub(super) fn hash_cell<A: Arch>(x: u32, y: u32, seed: u32) -> u32 {
    const BYTE_SHUFFLE: [u8; 64] = [
        3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9,
        15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5,
        11, 8, 10, 9, 15, 12, 14, 13,
    ];
    let shuffle_indices = unsafe { Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);

    let x_shuf = (
        Simd::<u32, A>::splat(x.wrapping_mul(seed)).permute_8(shuffle_indices) ^ prime
    ).to_array()[0];
    let y_shuf = (
        Simd::<u32, A>::splat(y.wrapping_mul(seed)).permute_8(shuffle_indices) ^ prime
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

/// Per-cell setup pass. Hashes the cell's four corners once, splits each
/// hash into `(tx, ty)` jitter offsets (cell offset folded inside), and splats
/// `xpart[c][x] = (sx - tx)^2` across every x sample in the cell's run
#[inline(always)]
pub(super) fn grid_cellular_hash<'a, A: Arch>(
    params: &GridNoiseParams<2>,
    grid_data: &mut GridData<2>,
    xpart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    ypart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    x_it: usize,
    x_idx: usize,
    y_it: usize,
    y_idx: usize
) {
    let lanes = Simd::<f32, A>::LANES;

    // Candidate order: 0=(cx,cy), 1=(cx+1,cy), 2=(cx,cy+1), 3=(cx+1,cy+1).
    let cx = grid_data.grid_start[0] + (x_it as i32);
    let cy = grid_data.grid_start[1] + (y_it as i32);
    let cx0 = grid_data.octave_tiling[0].map_or(cx, |t| cx.rem_euclid(t as i32));
    let cx1 = grid_data.octave_tiling[0].map_or(cx + 1, |t| (cx + 1).rem_euclid(t as i32));
    let cy0 = grid_data.octave_tiling[1].map_or(cy, |t| cy.rem_euclid(t as i32));
    let cy1 = grid_data.octave_tiling[1].map_or(cy + 1, |t| (cy + 1).rem_euclid(t as i32));

    let hash_tl = hash_cell::<A>(cx0 as u32, cy0 as u32, params.seed);
    let hash_tr = hash_cell::<A>(cx1 as u32, cy0 as u32, params.seed);
    let hash_bl = hash_cell::<A>(cx0 as u32, cy1 as u32, params.seed);
    let hash_br = hash_cell::<A>(cx1 as u32, cy1 as u32, params.seed);

    // The +ox/+oy cell offsets are folded into the jitters so the same
    // dist calc covers every candidate
    let (tx0, ty0) = split_hash(hash_tl);
    let (mut tx1, ty1) = split_hash(hash_tr);
    tx1 += 1.0;
    let (tx2, mut ty2) = split_hash(hash_bl);
    ty2 += 1.0;
    let (mut tx3, mut ty3) = split_hash(hash_br);
    tx3 += 1.0;
    ty3 += 1.0;

    let x_next = (unsafe { grid_data.grid_indices[0].get_unchecked(x_it).assume_init() }) as usize;
    let y_next = (unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() }) as usize;
    let tx0v = Simd::<f32, A>::splat(tx0);
    let tx1v = Simd::<f32, A>::splat(tx1);
    let tx2v = Simd::<f32, A>::splat(tx2);
    let tx3v = Simd::<f32, A>::splat(tx3);
    let ty0v = Simd::<f32, A>::splat(ty0);
    let ty1v = Simd::<f32, A>::splat(ty1);
    let ty2v = Simd::<f32, A>::splat(ty2);
    let ty3v = Simd::<f32, A>::splat(ty3);

    let mut x_cur_idx = x_idx;
    let mut amount = (x_next - x_idx) as isize;
    while amount > 0 {
        let sx = unsafe { grid_data.distances[0].load_simd(x_cur_idx) };

        unsafe {
            let dx = sx - tx0v;
            xpart[0].write_simd(x_cur_idx, dx * dx);
            let dx = sx - tx1v;
            xpart[1].write_simd(x_cur_idx, dx * dx);
            let dx = sx - tx2v;
            xpart[2].write_simd(x_cur_idx, dx * dx);
            let dx = sx - tx3v;
            xpart[3].write_simd(x_cur_idx, dx * dx);
        }
        amount -= lanes as isize;
        x_cur_idx += lanes;
    }

    let mut y_cur_idx = y_idx;
    let mut amount = (y_next - y_idx) as isize;
    while amount > 0 {
        let sy = unsafe { grid_data.distances[1].load_simd(y_cur_idx) };

        unsafe {
            let dy = sy - ty0v;
            ypart[0].write_simd(y_cur_idx, dy * dy);
            let dy = sy - ty1v;
            ypart[1].write_simd(y_cur_idx, dy * dy);
            let dy = sy - ty2v;
            ypart[2].write_simd(y_cur_idx, dy * dy);
            let dy = sy - ty3v;
            ypart[3].write_simd(y_cur_idx, dy * dy);
        }
        amount -= lanes as isize;
        y_cur_idx += lanes;
    }
}

/// For each row in the cell's y-range, broadcasts the per-candidate
/// `ypart[c][y]` and adds it to the splatted `xpart[c][x]`, then
/// find min over the 4 candidates and writes `sqrt(min_dist)` to `raw`.
#[inline(always)]
pub(super) fn grid_cellular_fill<'a, A: Arch>(
    grid_data: &mut GridData<2>,
    xpart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    ypart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    x_range: &Range<usize>,
    y_range: &Range<usize>,
    raw: &mut [f32]
) -> bool {
    let lanes = Simd::<f32, A>::LANES;
    let row_width = grid_data.grid_size[0];
    let mut any_far = false;

    for y in y_range.start..y_range.end {
        let yp0 = Simd::<f32, A>::splat(unsafe { ypart[0].get_unchecked(y).assume_init() });
        let yp1 = Simd::<f32, A>::splat(unsafe { ypart[1].get_unchecked(y).assume_init() });
        let yp2 = Simd::<f32, A>::splat(unsafe { ypart[2].get_unchecked(y).assume_init() });
        let yp3 = Simd::<f32, A>::splat(unsafe { ypart[3].get_unchecked(y).assume_init() });

        let sy = unsafe { grid_data.distances[1].get_unchecked(y).assume_init() };
        let y_edge_lo = Simd::<f32, A>::splat(sy + 0.5);
        let y_edge_hi = Simd::<f32, A>::splat(1.5 - sy);

        let row_start = y * row_width;
        let row_end = row_start + row_width;
        let mut index = x_range.start;
        while index < x_range.end {
            let sx = unsafe { grid_data.distances[0].load_simd(index) };
            let xp0 = unsafe { xpart[0].load_simd(index) };
            let xp1 = unsafe { xpart[1].load_simd(index) };
            let xp2 = unsafe { xpart[2].load_simd(index) };
            let xp3 = unsafe { xpart[3].load_simd(index) };

            // Threshold
            let x_edge_lo = sx + Simd::<f32, A>::splat(0.5);
            let x_edge_hi = Simd::<f32, A>::splat(1.5) - sx;
            let closest_edge = x_edge_lo.min(y_edge_lo).min(x_edge_hi.min(y_edge_hi));
            let threshold = closest_edge * closest_edge;

            let dist_sq = (xp0 + yp0)
                .min(xp1 + yp1)
                .min(xp2 + yp2)
                .min(xp3 + yp3);

            let is_far = dist_sq.simd_gt(threshold).to_bits() != 0;
            any_far |= is_far;

            let min_dist = dist_sq.sqrt();
            // Writes are clamped to the row end so a SIMD block can overshoot the last
            // cell boundary without running past the end of the row.
            min_dist.copy_to_slice(
                &mut raw[row_start + index..(row_start + index + lanes).min(row_end)]
            );
            index += lanes;
        }
    }

    any_far
}

/// Hash the 8-cell ring around the 2x2 base cells
/// once per cell, split each hash into `(jx, jy)` jitters (cell offset folded in),
/// and splat `xpart_ext`/`ypart_ext` into buffers 4..12 over the cell's runs
#[inline(always)]
pub(super) fn grid_cellular_hash_ring<'a, A: Arch>(
    params: &GridNoiseParams<2>,
    grid_data: &mut GridData<2>,
    xpart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    ypart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    x_it: usize,
    x_idx: usize,
    y_it: usize,
    y_idx: usize
) {
    let lanes = Simd::<f32, A>::LANES;

    let cx = grid_data.grid_start[0] + (x_it as i32);
    let cy = grid_data.grid_start[1] + (y_it as i32);
    let tile_x = |x: i32| grid_data.octave_tiling[0].map_or(x, |t| x.rem_euclid(t as i32));
    let tile_y = |y: i32| grid_data.octave_tiling[1].map_or(y, |t| y.rem_euclid(t as i32));

    let jitters = RING.map(|(ox, oy)| {
        let (mut jx, mut jy) = split_hash(
            hash_cell::<A>(tile_x(cx + ox) as u32, tile_y(cy + oy) as u32, params.seed)
        );
        jx += ox as f32;
        jy += oy as f32;
        (jx, jy)
    });

    let x_next = (unsafe { grid_data.grid_indices[0].get_unchecked(x_it).assume_init() }) as usize;
    let y_next = (unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() }) as usize;

    let mut x_cur_idx = x_idx;
    let mut amount = (x_next - x_idx) as isize;
    while amount > 0 {
        let sx = unsafe { grid_data.distances[0].load_simd(x_cur_idx) };

        unsafe {
            for (c, (jx, _)) in jitters.iter().enumerate() {
                let dx = sx - Simd::<f32, A>::splat(*jx);
                xpart[4 + c].write_simd(x_cur_idx, dx * dx);
            }
        }
        amount -= lanes as isize;
        x_cur_idx += lanes;
    }

    let mut y_cur_idx = y_idx;
    let mut amount = (y_next - y_idx) as isize;
    while amount > 0 {
        let sy = unsafe { grid_data.distances[1].load_simd(y_cur_idx) };

        unsafe {
            for (c, (_, jy)) in jitters.iter().enumerate() {
                let dy = sy - Simd::<f32, A>::splat(*jy);
                ypart[4 + c].write_simd(y_cur_idx, dy * dy);
            }
        }
        amount -= lanes as isize;
        y_cur_idx += lanes;
    }
}

/// For each row in the cell's y-range, broadcasts the per-candidate
/// `ypart[c][y]` and adds it to the splatted `xpart[c][x]`, then
/// narrows the minimum over the 8 ring candidates (first 4 already
/// compared by grid_cellular_fill) and writes the result to `raw`.
#[inline(always)]
pub(super) fn grid_cellular_fill_ring<'a, A: Arch>(
    grid_data: &mut GridData<2>,
    xpart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    ypart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    x_range: &Range<usize>,
    y_range: &Range<usize>,
    raw: &mut [f32]
) {
    let lanes = Simd::<f32, A>::LANES;
    let row_width = grid_data.grid_size[0];

    for y in y_range.clone() {
        let mut yp = [Simd::<f32, A>::splat(0.0); 8];
        for (i, c) in (4..12).enumerate() {
            yp[i] = Simd::<f32, A>::splat(unsafe { ypart[c].get_unchecked(y).assume_init() });
        }

        let row_start = y * row_width;
        let row_end = row_start + row_width;
        let mut index = x_range.start;
        while index < x_range.end {
            let mut outer_min = Simd::<f32, A>::splat(f32::MAX);
            for (i, c) in (4..12).enumerate() {
                let xp = unsafe { xpart[c].load_simd(index) };
                outer_min = outer_min.min(xp + yp[i]);
            }
            let outer_min = outer_min.sqrt();

            let existing: Simd<f32, A> = Simd::<f32, A>::from_slice(
                &raw[row_start + index..(row_start + index + lanes).min(row_end)]
            );

            let final_min = outer_min.min(existing);

            final_min.copy_to_slice(
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

    let raw_val: Simd<f32, A> = unsafe {
        maybe_tail_load::<A, IS_TAIL>(sample_start..sample_end, raw)
    } * Simd::<f32, A>::splat(grid_data.weight);

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
        use crate::{Billow, HybridMulti, Multi, PingPong, Ridged, Terrace};
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
