use std::fmt::Debug;
use std::{array::from_fn, fmt, mem::MaybeUninit};

use crate::grid_helpers::CHUNK_SIZE;
use crate::{
    api::grid::interface::GridNoiseParams,
    grid_helpers::{AlignedBuffer, Arena, assume_init_slice, configure_tiling, grid_fill_indices},
    simd::arch_simd::ArchSimd,
};

const LANES: usize = ArchSimd::<f32>::LANES;

#[derive(Debug)]
pub(crate) struct PerlinGridData<const D: usize> {
    first_pos: [i32; D],
    cur_pos: [i32; D],
    pub remaining: [usize; D],
    pub completed: [usize; D],
    pub grid_size: [usize; D],
    pub cur_size: [usize; D],
    pub grid_start: [i32; D],
    pub increment: [f32; D],
    pub num_loops: [usize; D],
    pub octave_tiling: [Option<u32>; D],
    pub distances: [AlignedBuffer<f32>; D],
    pub fade_factors: [AlignedBuffer<f32>; D],
    pub grid_indices: [AlignedBuffer<u32>; D],
}

impl<const D: usize> PerlinGridData<D> {
    #[inline(always)]
    pub fn new(params: &GridNoiseParams<D>) -> Self {
        Self {
            first_pos: [0; D],
            cur_pos: [0; D],
            grid_size: params.grid_size,
            remaining: params.grid_size,
            completed: [0; D],
            cur_size: [0; D],
            grid_start: [0; D],
            increment: from_fn(|i| params.frequency[i] * params.magnification),
            num_loops: [0; D],
            octave_tiling: configure_tiling(params),
            distances: from_fn(|_| AlignedBuffer::new()),
            fade_factors: from_fn(|_| AlignedBuffer::new()),
            grid_indices: from_fn(|_| AlignedBuffer::new()),
        }
    }

    fn fill_dist_and_lerps<const AXIS: usize>(&mut self) {
        let cur_pos = self.cur_pos[AXIS] as f32;
        let increment = self.increment[AXIS];
        let grid_start = self.grid_start[AXIS] as f32;
        let frac_start = (cur_pos * increment - grid_start).max(0.0);

        let mut cur_dist = ArchSimd::iota(frac_start) * ArchSimd::splat(increment);
        let chunk_increment = ArchSimd::splat(increment * LANES as f32);

        for i in (0..self.cur_size[AXIS]).step_by(LANES) {
            let fract_dist = cur_dist.fract();
            let cur_lerp = fract_dist.quintic_lerp();

            unsafe {
                self.distances[AXIS].store_simd_aligned(i, fract_dist);
                self.fade_factors[AXIS].store_simd_aligned(i, cur_lerp);
            }
            cur_dist += chunk_increment;
        }
    }

    pub fn reset_axis<const AXIS: usize>(&mut self) {
        self.remaining[AXIS] = self.grid_size[AXIS];
        self.cur_pos[AXIS] = self.first_pos[AXIS];
        self.completed[AXIS] = 0;
    }

    pub fn advance_axis<const AXIS: usize>(&mut self) {
        self.cur_size[AXIS] = self.remaining[AXIS].min(CHUNK_SIZE);
        self.remaining[AXIS] -= self.cur_size[AXIS];

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        self.grid_start[AXIS] = (self.cur_pos[AXIS] as f32 * self.increment[AXIS]).floor() as i32;

        // Get the distances from the gradient gridpoints.
        // Identify the cutoff points between frequency-based grid boundaries .
        self.num_loops[AXIS] = grid_fill_indices(
            &mut self.grid_indices[AXIS],
            &self.distances[AXIS],
            self.cur_size[AXIS],
        );
        self.fill_dist_and_lerps::<AXIS>();

        self.cur_pos[AXIS] += self.cur_size[AXIS] as i32;
    }

    pub fn axis_has_full_remaining<const AXIS: usize>(&self) -> bool {
        self.remaining[AXIS] >= CHUNK_SIZE
    }

    pub fn axis_has_remaining<const AXIS: usize>(&self) -> bool {
        self.remaining[AXIS] > 0
    }

    pub fn for_each_grid_chunk<F: FnMut(&Self)>(&mut self, f: &mut F) {
        while self.axis_has_remaining::<1>() {
            self.reset_axis::<0>();
            self.advance_axis::<1>();
            while self.axis_has_remaining::<0>() {
                self.advance_axis::<0>();
                f(self);

                self.completed[0] += CHUNK_SIZE;
            }
            self.completed[1] += CHUNK_SIZE;
        }
    }
}
