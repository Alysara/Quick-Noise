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
    3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9,
    15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5, 11, 8, 10, 9, 15, 12, 14, 13, 3, 0, 2, 1, 7, 4, 6, 5,
    11, 8, 10, 9, 15, 12, 14, 13,
];

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

struct RowWindow {
    top: *mut f32,
    bot: *mut f32,
    width: usize,
}

impl RowWindow {
    fn new(arena: &mut Arena, width: usize) -> Self {
        Self {
            top: arena.allocate::<f32>(width * 2).as_mut_ptr().cast(),
            bot: arena.allocate::<f32>(width * 2).as_mut_ptr().cast(),
            width,
        }
    }

    fn top(&self) -> &[(f32, f32)] {
        unsafe { 
            std::slice::from_raw_parts(self.top.cast::<(f32, f32)>(), self.width) 
        }
    }

    fn bot(&self) -> &[(f32, f32)] {
        unsafe { 
            std::slice::from_raw_parts(self.bot.cast::<(f32, f32)>(), self.width) 
        }
    }

    fn top_mut(&mut self) -> &mut [(f32, f32)] {
        unsafe { 
            std::slice::from_raw_parts_mut(self.top.cast::<(f32, f32)>(), self.width) 
        }
    }

    fn bot_mut(&mut self) -> &mut [(f32, f32)] {
        unsafe { 
            std::slice::from_raw_parts_mut(self.bot.cast::<(f32, f32)>(), self.width) 
        }
    }

    fn fill_row<A: Arch>(
        params: &GridNoiseParams<2>,
        grid_data: &GridData<2>,
        buff: &mut [(f32, f32)],
        cy: i32,
    ) {
        let cy = 
            grid_data.octave_tiling[1]
            .map_or(cy, |t| cy.rem_euclid(t as i32));
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
            let cx = 
                grid_data.octave_tiling[0]
                .map_or(cx, |t| cx.rem_euclid(t as i32));
            buff[i] = split_hash(hash_cell_with_y::<A>(cx as u32, y_shuf, params.seed));
        }
    }

    #[inline(always)]
    fn swap_top_bottom(&mut self) {
        std::mem::swap(&mut self.top, &mut self.bot);
    }
}

#[inline(always)]
fn simd_rem_euclid_i32<A: Arch>(x: Simd<i32, A>, t: i32) -> Simd<i32, A> {
    let t_f = Simd::<f32, A>::splat(t as f32);
    let x_f = x.cast_float();
    (x_f - (x_f / t_f).floor() * t_f).cast_int_trunc()
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

        // Both the x and y candidate buffers must fit the larger axis.
        let candidate_size = padded_size[0].max(padded_size[1]);
        let required_cache =
            padded_size[1] * 3 + padded_size[0] * 3 + candidate_size * 24 + total_size
            + (padded_size[0] + 1) * 4;
        let mut cache = ArenaBuffer::<A>::with_capacity(required_cache);
        let mut arena = Arena::with_cache(&mut cache);
        let mut sub_arena = arena.allocate_arena(padded_size[0] * 3 + padded_size[1] * 3 );

        let mut grid_data = GridData::new::<A, LERP>(&params, &mut sub_arena, &padded_size);

        // Scratch buffer for the raw cellular min and combiner pass
        let raw = unsafe { arena.allocate(total_size).assume_init_mut() };

        // per-cell setup pass, driven by the cell x/y indices
        let mut window  = RowWindow::new(&mut arena, grid_data.num_loops[0] + 1);
        let mut candidates = CellularCandidates2D::new(&mut arena, candidate_size);
        // Hash top row and cache
        RowWindow::fill_row::<A>(&params, &grid_data, window.top_mut(), grid_data.grid_start[1]);

        let mut y_idx = 0;
        for y_it in 0..grid_data.num_loops[1] {
            let y_next_idx = unsafe {
                grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize
            };
            // Hash bottom row and cache
            RowWindow::fill_row::<A>(&params, &grid_data, window.bot_mut(), grid_data.grid_start[1] + y_it as i32 + 1);

            let mut x_idx = 0;
            for x_it in 0..grid_data.num_loops[0] {
                let x_next_idx = unsafe {
                    grid_data.grid_indices[0].get_unchecked(x_it).assume_init() as usize
                };

                let x_sample_range = x_idx..x_next_idx;
                let y_sample_range = y_idx..y_next_idx;

                let any_far = if x_next_idx - x_idx <= Simd::<f32, A>::LANES {
                    grid_cellular_hash_and_fill::<A>(
                        &mut grid_data,
                        x_it,
                        x_idx,
                        y_it,
                        y_idx,
                        window.top(),
                        window.bot(),
                        raw,
                    )
                } else {
                    grid_cellular_hash::<A>(
                        &mut grid_data,
                        &mut candidates.xpart,
                        &mut candidates.ypart,
                        x_it,
                        x_idx,
                        y_it,
                        y_idx,
                        window.top(),
                        window.bot(),
                    );
                    grid_cellular_fill::<A>(
                        &mut grid_data,
                        &mut candidates.xpart,
                        &mut candidates.ypart,
                        &x_sample_range,
                        &y_sample_range,
                        raw
                    )
                };

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
            // Reuse caches
            window.swap_top_bottom();

            y_idx = unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() as usize };
        }
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
    seed: u32,
) -> Simd<u32, A> {
    let shuffle_indices = unsafe { Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);
    let seed_v = Simd::<u32, A>::splat(seed);

    let x_shuf = (cx_v * seed_v).permute_8(shuffle_indices) ^ prime;
    x_shuf * y_shuf ^ x_shuf
}

pub(super) fn hash_cell_y<A: Arch>(y: u32, seed: u32) -> u32 {
    let shuffle_indices = unsafe { Simd::<u8, A>::from_slice_unchecked(&BYTE_SHUFFLE[..]) };
    let prime = Simd::<u32, A>::splat(0x85ebca6b_u32);
    (
        Simd::<u32, A>::splat(y.wrapping_mul(seed))
        .permute_8(shuffle_indices) ^ prime
    ).to_array()[0]
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

/// Per-cell setup pass. Hashes the cell's four corners once, splits each
/// hash into `(tx, ty)` jitter offsets (cell offset folded inside), and splats
/// `xpart[c][x] = (sx - tx)^2` across every x sample in the cell's run
#[inline(always)]
pub(super) fn grid_cellular_hash<'a, A: Arch>(
    grid_data: &mut GridData<2>,
    xpart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    ypart: &mut [&'a mut [MaybeUninit<f32>]; 12],
    x_it: usize,
    x_idx: usize,
    y_it: usize,
    y_idx: usize,
    top: &[(f32, f32)],
    bot: &[(f32, f32)],
) {
    let lanes = Simd::<f32, A>::LANES;

    // Candidate order: 0=(cx,cy), 1=(cx+1,cy), 2=(cx,cy+1), 3=(cx+1,cy+1).

    // The +ox/+oy cell offsets are folded into the jitters so the same
    // dist calc covers every candidate
    let (tx0, ty0) = top[x_it];
    let (tx1, ty1) = top[x_it + 1];
    let (tx2, ty2) = bot[x_it];
    let (tx3, ty3) = bot[x_it + 1];
    let tx1 = tx1 + 1.0;
    let ty2 = ty2 + 1.0;
    let tx3 = tx3 + 1.0;
    let ty3 = ty3 + 1.0;

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

/// Narrow-cell fast path: computes x-parts once into registers and folds
/// the fill pass inline, never touching the xpart/ypart buffers.
/// Only valid when the cell's x-range fits in a single SIMD block.
#[inline(always)]
pub(super) fn grid_cellular_hash_and_fill<'a, A: Arch>(
    grid_data: &mut GridData<2>,
    x_it: usize,
    x_idx: usize,
    y_it: usize,
    y_idx: usize,
    top: &[(f32, f32)],
    bot: &[(f32, f32)],
    raw: &mut [f32],
) -> bool {
    let lanes = Simd::<f32, A>::LANES;

    let (tx0, ty0) = top[x_it];
    let (tx1, ty1) = top[x_it + 1];
    let (tx2, ty2) = bot[x_it];
    let (tx3, ty3) = bot[x_it + 1];
    let tx1 = tx1 + 1.0;
    let ty2 = ty2 + 1.0;
    let tx3 = tx3 + 1.0;
    let ty3 = ty3 + 1.0;

    let x_next = (unsafe { grid_data.grid_indices[0].get_unchecked(x_it).assume_init() }) as usize;
    let y_next = (unsafe { grid_data.grid_indices[1].get_unchecked(y_it).assume_init() }) as usize;
    let tx0v = Simd::<f32, A>::splat(tx0);
    let tx1v = Simd::<f32, A>::splat(tx1);
    let tx2v = Simd::<f32, A>::splat(tx2);
    let tx3v = Simd::<f32, A>::splat(tx3);

    debug_assert!(
        x_next - x_idx <= lanes,
        "grid_cellular_hash_and_fill called on a wide cell"
    );

    // Single x-block: compute xp0..xp3 once, they stay in registers across
    // the entire y-loop.
    let sx = unsafe { grid_data.distances[0].load_simd(x_idx) };
    let xp0 = { let d = sx - tx0v; d * d };
    let xp1 = { let d = sx - tx1v; d * d };
    let xp2 = { let d = sx - tx2v; d * d };
    let xp3 = { let d = sx - tx3v; d * d };

    let row_width = grid_data.grid_size[0];
    let mut any_far = false;

    for y in y_idx..y_next {
        let sy = unsafe { grid_data.distances[1].get_unchecked(y).assume_init() };
        let yp0 = Simd::<f32, A>::splat({ let d = sy - ty0; d * d });
        let yp1 = Simd::<f32, A>::splat({ let d = sy - ty1; d * d });
        let yp2 = Simd::<f32, A>::splat({ let d = sy - ty2; d * d });
        let yp3 = Simd::<f32, A>::splat({ let d = sy - ty3; d * d });

        let x_edge_lo = sx + Simd::<f32, A>::splat(0.5);
        let x_edge_hi = Simd::<f32, A>::splat(1.5) - sx;
        let y_edge_lo = Simd::<f32, A>::splat(sy + 0.5);
        let y_edge_hi = Simd::<f32, A>::splat(1.5 - sy);
        let closest_edge = x_edge_lo.min(y_edge_lo).min(x_edge_hi.min(y_edge_hi));
        let threshold = closest_edge * closest_edge;

        let dist_sq = (xp0 + yp0).min(xp1 + yp1).min(xp2 + yp2).min(xp3 + yp3);
        let is_far = dist_sq.simd_gt(threshold).to_bits() != 0;
        any_far |= is_far;

        let min_dist = dist_sq.sqrt();
        let row_start = y * row_width;
        let row_end = row_start + row_width;
        min_dist.copy_to_slice(
            &mut raw[row_start + x_idx..(row_start + x_idx + lanes).min(row_end)]
        );
    }

    any_far
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
