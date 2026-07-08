use std::{array::from_fn, fmt, mem::MaybeUninit};

use crate::{
    api::grid::interface::GridNoiseParams, grid_helpers::{Arena, assume_init_slice, configure_tiling, grid_fill_indices_slice}, simd::arch_simd::ArchSimd,
};

const LANES: usize = ArchSimd::<f32>::LANES;

pub(crate) struct PerlinGridData<'a, const D: usize> {
    pub grid_start: [i32; D],
    pub increment: [f32; D],
    pub num_loops: [usize; D],
    pub octave_tiling: [Option<u32>; D],
    pub distances: [&'a mut [MaybeUninit<f32>]; D],
    pub fade_factors: [&'a mut [MaybeUninit<f32>]; D],
    pub grid_indices: [&'a mut [MaybeUninit<u32>]; D],
}

impl<'a, const D: usize> PerlinGridData<'a, D> {
    #[inline(always)]
    pub fn new(params: &GridNoiseParams<D>, arena: &mut Arena<'a>, padded_size: &[usize; D]) -> Self {
        let increment = from_fn(|i| params.frequency[i] * params.magnification);
        let block_pos: [i32; D] = from_fn(|i| params.position[i] * params.grid_size[i] as i32);

        // Get the starting gradient coordinates and how far the first sample is to the next one.
        let grid_start: [i32; D] = from_fn(|i| (block_pos[i] as f32 * increment[i].floor()) as i32);
        let frac_start: [f32; D] =
            from_fn(|i| (block_pos[i] as f32 * increment[i] - grid_start[i] as f32).max(0.0));

        // Quintic lerp the distances to get the fade factor.
        let distances = from_fn(|i| arena.allocate(padded_size[i]));
        let fade_factors = from_fn(|i| arena.allocate(padded_size[i]));

        // Get the distances from the gradient gridpoints.
        let mut cur_dist: [_; D] =
            from_fn(|i| ArchSimd::iota(frac_start[i]) * ArchSimd::splat(increment[i]));
        let chunk_increment: [_; D] = from_fn(|i| ArchSimd::splat(increment[i] * LANES as f32));

        for axis in 0..2 {
            for i in (0..params.grid_size[axis]).step_by(LANES) {
                let fract_dist = cur_dist[axis].fract();
                let cur_lerp = fract_dist.quintic_lerp();

                unsafe {
                    fract_dist.copy_to_slice_unchecked(
                        distances[axis].get_unchecked_mut(i..).assume_init_mut(),
                    );
                    cur_lerp.copy_to_slice_unchecked(
                        fade_factors[axis].get_unchecked_mut(i..).assume_init_mut(),
                    );
                }
                cur_dist[axis] += chunk_increment[axis];
            }
        }

        // Identify the cutoff points between frequency-based grid boundaries .
        let mut grid_indices = from_fn(|i| arena.allocate(padded_size[i]));
        let num_loops = grid_fill_indices_slice(&mut grid_indices, &distances, params.grid_size);

        // Adjust the tiling.
        let octave_tiling = configure_tiling(params);

        Self {
            grid_start,
            increment,
            num_loops,
            octave_tiling,
            distances,
            fade_factors,
            grid_indices,
        }
    }
}

impl<'a, const D: usize> fmt::Debug for PerlinGridData<'a, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe {
            f.debug_struct("PerlinGridData")
                .field("grid_start", &self.grid_start)
                .field("increment", &self.increment)
                .field("num_loops", &self.num_loops)
                .field("octave_tiling", &self.octave_tiling)
                .field("distances.x", &assume_init_slice(self.distances[0]))
                .field("distances.y", &assume_init_slice(self.distances[1]))
                .field("fade_factors.x", &assume_init_slice(self.fade_factors[0]))
                .field("fade_factors.y", &assume_init_slice(self.fade_factors[1]))
                .field("grid_indices.x", &assume_init_slice(self.grid_indices[0]))
                .field("grid_indices.y", &assume_init_slice(self.grid_indices[1]))
                .finish()
        }
    }
}
