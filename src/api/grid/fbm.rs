use std::marker::PhantomData;

use crate::Ridged;
use crate::api::configs::*;
use crate::api::grid::interface::{GridNoiseImpl, GridNoiseParams};
use crate::api::parameters::*;
use crate::api::seed::gen_octave_seed;
use crate::fractal::{Fractal, FractalState};
use crate::grid_helpers::{Arena, ArenaBuffer};
use crate::math::random::Random;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_reg::iters::IntoSimdIterator;

#[inline(always)]
pub fn fbm_noise<const D: usize, F: Fractal, T: GridNoiseImpl<D>, const INIT: bool>(
    grid_config: &GridConfig<D>,
    general_config: &GeneralConfig,
    fbm_config: &FbmConfig<D>,
    fractal_config: F::Config,
    result: &mut [f32],
) {
    let octaves = fbm_config.num_grid_octaves();

    // Fill with zeroes if there are no octaves.
    if octaves == 0 {
        if INIT {
            result.fill(0.0)
        }
        return;
    }

    let base_seed = Random::static_mix_u64_pair(grid_config.grid_seed, general_config.seed);

    // FBM algorithm:
    let frequency = std::array::from_fn(|i| fbm_config.scaling[i] * fbm_config.frequency);
    let weight = if general_config.normalization && F::WEIGHT_DECAY {
        fbm_config.normalize_amplitude(general_config.amplitude)
    } else {
        general_config.amplitude
    };

    // First octave:
    let mut params = GridNoiseParams {
        seed: gen_octave_seed(frequency, base_seed),
        grid_size: grid_config.dimensions,
        position: grid_config.position,
        magnification: general_config.magnification,
        tiling: grid_config.tiling,
        frequency,
        weight,
    };

    let total_size: usize = grid_config.dimensions.iter().product();
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
            params.weight *= fbm_config.persistence;
        }

        params.frequency = std::array::from_fn(|i| params.frequency[i] * fbm_config.lacunarity);
        params.seed = gen_octave_seed(params.frequency, base_seed);

        T::sample::<F, false, false>(params, fractal_config, state, result);
    }

    params.weight *= fbm_config.persistence;
    params.frequency = std::array::from_fn(|i| params.frequency[i] * fbm_config.lacunarity);
    params.seed = gen_octave_seed(params.frequency, base_seed);

    T::sample::<F, false, true>(params, fractal_config, state, result);
}

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default, Copy, Clone)]
pub struct FbmGridBuilder<const D: usize, F: Fractal, T: GridNoiseImpl<D>> {
    grid_config: GridConfig<D>,
    general_config: GeneralConfig,
    fbm_config: FbmConfig<D>,
    fractal_config: F::Config,
    _noise_type: PhantomData<T>,
}

params_general_builder!(FbmGridBuilder, [const D: usize, F: Fractal, T: GridNoiseImpl<D>], [D, F, T]);
params_fbm_builder!(FbmGridBuilder, [const D: usize, F: Fractal, T: GridNoiseImpl<D>], [D, F, T]);
params_fbm_scaling_2d!(FbmGridBuilder, [F: Fractal, T: GridNoiseImpl<2>], [2, F, T]);
params_fbm_scaling_3d!(FbmGridBuilder, [F: Fractal, T: GridNoiseImpl<3>], [3, F, T]);

impl<const D: usize, F: Fractal, T: GridNoiseImpl<D>> FbmGridBuilder<D, F, T> {
    #[inline(always)]
    pub(crate) fn from_config(grid_config: GridConfig<D>) -> Self {
        Self {
            grid_config,
            ..Default::default()
        }
    }

    declare_build!(self, {
        let size = self.grid_config.dimensions.iter().product();
        let mut result = vec![0.0; size];
        fbm_noise::<D, F, T, true>(
            &self.grid_config,
            &self.general_config,
            &self.fbm_config,
            self.fractal_config,
            result.as_mut_slice(),
        );
        result
    });

    declare_fill!(self, result, {
        fbm_noise::<D, F, T, true>(
            &self.grid_config,
            &self.general_config,
            &self.fbm_config,
            self.fractal_config,
            result,
        );
    });

    declare_fill_onto!(self, result, {
        fbm_noise::<D, F, T, false>(
            &self.grid_config,
            &self.general_config,
            &self.fbm_config,
            self.fractal_config,
            result,
        );
    });

    declare_into_iter!(self, { self.build().into_simd_iter() });
}

impl<const D: usize, T: GridNoiseImpl<D>> FbmGridBuilder<D, Ridged, T> {
    pub fn gain(mut self, gain: f32) -> Self {
        self.fractal_config.gain = gain;
        self
    }

}
