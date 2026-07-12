use std::marker::PhantomData;

use crate::Ridged;
use crate::api::configs::*;
use crate::api::grid::interface::{GridNoise, GridNoiseParams};
use crate::api::parameters::*;
use crate::api::seed::gen_octave_seed;
use crate::fractal::{Fractal, FractalState};
use crate::grid_helpers::{Arena, ArenaBuffer};
use crate::math::random::Random;
use crate::simd::simd_reg::iters::IntoSimdIterator;
use crate::simd::arch_simd::ArchSimd;

#[inline(always)]
pub fn sample_grid<const D: usize, F: Fractal, T: GridNoise<D>, const INIT: bool>(
    grid_config: &GridConfig<D>,
    noise_config: &NoiseConfig<D>,
    fractal_config: F::Config,
    result: &mut [f32],
) {
    let octaves = noise_config.num_grid_octaves();

    // Fill with zeroes if there are no octaves.
    if octaves == 0 {
        if INIT {
            result.fill(0.0)
        }
        return;
    }

    let base_seed = Random::static_mix_u64_pair(grid_config.grid_seed, noise_config.seed);

    // FBM algorithm:
    let frequency = std::array::from_fn(|i| noise_config.scaling[i] * noise_config.frequency);
    let weight = if noise_config.normalization && F::WEIGHT_DECAY {
        noise_config.normalize_amplitude(noise_config.amplitude)
    } else {
        noise_config.amplitude
    };

    // First octave:
    let mut params = GridNoiseParams {
        seed: gen_octave_seed(frequency, base_seed),
        grid_size: grid_config.grid_size,
        position: grid_config.position,
        magnification: noise_config.magnification,
        tiling: grid_config.tiling,
        frequency,
        weight,
    };

    let total_size: usize = grid_config.grid_size.iter().product();
    let needed_state_size = total_size * F::State::STATE_SIZE;
    let mut state_cache = ArenaBuffer::with_capacity(needed_state_size);
    let mut arena = Arena::with_cache(&mut state_cache);
    let state = arena.allocate(needed_state_size);
    let state = unsafe { state.assume_init_mut() };

    if octaves == 1 {
        T::sample::<F, INIT, true>(params, fractal_config, state, result);
        return;
    } else {
        T::sample::<F, INIT, false>(params, fractal_config, state, result);
    }

    // Subsequent octaves:
    for _ in 1..(octaves - 1) {
        if F::WEIGHT_DECAY {
            params.weight *= noise_config.persistence;
        }

        params.frequency = std::array::from_fn(|i| params.frequency[i] * noise_config.lacunarity);
        params.seed = gen_octave_seed(params.frequency, base_seed);

        T::sample::<F, false, false>(params, fractal_config, state, result);
    }

    params.weight *= noise_config.persistence;
    params.frequency = std::array::from_fn(|i| params.frequency[i] * noise_config.lacunarity);
    params.seed = gen_octave_seed(params.frequency, base_seed);

    T::sample::<F, false, true>(params, fractal_config, state, result);
}

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default, Copy, Clone)]
pub struct GridNoiseBuilder<const D: usize, F: Fractal, T: GridNoise<D>> {
    grid_config: GridConfig<D>,
    noise_config: NoiseConfig<D>,
    fractal_config: F::Config,
    _noise_type: PhantomData<T>,
}

params_noise_builder!(GridNoiseBuilder, [const D: usize, F: Fractal, T: GridNoise<D>], [D, F, T]);
params_noise_scaling_2d!(GridNoiseBuilder, [F: Fractal, T: GridNoise<2>], [2, F, T]);
params_noise_scaling_3d!(GridNoiseBuilder, [F: Fractal, T: GridNoise<3>], [3, F, T]);

impl<const D: usize, F: Fractal, T: GridNoise<D>> GridNoiseBuilder<D, F, T> {
    #[inline(always)]
    pub(crate) fn from_config(grid_config: GridConfig<D>) -> Self {
        Self {
            grid_config,
            ..Default::default()
        }
    }

    declare_build!(self, {
        let size = self.grid_config.grid_size.iter().product();
        let mut result = vec![0.0; size];
        sample_grid::<D, F, T, true>(
            &self.grid_config,
            &self.noise_config,
            self.fractal_config,
            result.as_mut_slice(),
        );
        result
    });

    declare_fill!(self, result, {
        sample_grid::<D, F, T, true>(
            &self.grid_config,
            &self.noise_config,
            self.fractal_config,
            result,
        );
    });

    declare_fill_onto!(self, result, {
        sample_grid::<D, F, T, false>(
            &self.grid_config,
            &self.noise_config,
            self.fractal_config,
            result,
        );
    });

    declare_into_iter!(self, { self.build().into_simd_iter() });
}

impl<const D: usize, T: GridNoise<D>> GridNoiseBuilder<D, Ridged, T> {
    pub fn gain(mut self, gain: f32) -> Self {
        self.fractal_config.gain = gain;
        self
    }

}
