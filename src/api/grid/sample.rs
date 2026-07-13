use crate::api::configs::*;
use crate::api::grid::interface::{GridNoise, GridNoiseParams, GridGenerator};
use crate::api::seed::gen_octave_seed;
use crate::math::random::Random;
use crate::noise::util::grid_helpers::{Arena, ArenaBuffer};
use crate::{Combiner, CombinerState};

impl<const D: usize, F: Combiner, S: GridGenerator<D>> GridNoise<D, F, S> {
    #[inline(always)]
    pub fn sample(
        grid_config: &GridConfig<D>,
        noise_config: &NoiseConfig<D>,
        fractal_config: &F::Config,
        result: &mut [f32],
    ) {
        let octaves = noise_config.num_grid_octaves();

        // Fill with zeroes if there are no octaves.
        if octaves == 0 {
            if noise_config.initialization {
                result.fill(0.0)
            }
            return;
        }

        let base_seed = Random::mix_u64_pair(grid_config.grid_seed, noise_config.seed);

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
        let f_config = *fractal_config;

        match (
            noise_config.initialization,
            noise_config.finalization && octaves == 1,
        ) {
            (false, false) => S::sample_grid::<F, false, false>(params, f_config, state, result),
            (true, false) => S::sample_grid::<F, true, false>(params, f_config, state, result),
            (false, true) => S::sample_grid::<F, false, true>(params, f_config, state, result),
            (true, true) => S::sample_grid::<F, true, true>(params, f_config, state, result),
        }

        // Subsequent octaves:
        for _ in 1..(octaves - 1) {
            if F::WEIGHT_DECAY {
                params.weight *= noise_config.persistence;
            }
            params.frequency =
                std::array::from_fn(|i| params.frequency[i] * noise_config.lacunarity);
            params.seed = gen_octave_seed(params.frequency, base_seed);
            S::sample_grid::<F, false, false>(params, f_config, state, result);
        }

        params.weight *= noise_config.persistence;
        params.frequency = std::array::from_fn(|i| params.frequency[i] * noise_config.lacunarity);
        params.seed = gen_octave_seed(params.frequency, base_seed);
        match noise_config.finalization {
            true => S::sample_grid::<F, false, true>(params, f_config, state, result),
            false => S::sample_grid::<F, false, false>(params, f_config, state, result),
        }
    }
}
