use std::array::from_fn;

use crate::BatchGenerator;
use crate::api::batch::interface::{BatchNoise, DimIter, DimTuple};
use crate::api::configs::*;
use crate::api::octave::Octave;
use crate::api::seed::gen_octave_seed;
use crate::noise::combiners::Combiner;
use crate::simd::arch_simd::ArchSimd;

const MAX_CUSTOM_OCTAVES: usize = 32;

fn get_max<const D: usize>(array: [f32; D]) -> f32 {
    array.iter().cloned().fold(f32::NEG_INFINITY, f32::max)
}

/// Helper static function for custom noise.
impl<const D: usize, F: Combiner, S: BatchGenerator<D>> BatchNoise<D, F, S> {
    #[inline(always)]
    pub fn sample_with_octaves<I: DimIter<D>>(
        noise_config: OctaveNoiseConfig<D>,
        fractal_config: F::Config,
        octave_list: &[Octave<D>],
        iters: I,
    ) -> impl Iterator<Item = ArchSimd<f32>> {
        let octaves = octave_list.len();

        let total_weight = octave_list
            .iter()
            .filter(|x| get_max(x.frequency) < 1.0)
            .fold(0.0, |acc, x| acc + x.weight);

        let weight_coef = match (noise_config.normalization, total_weight == 0.0) {
            (false, _) => noise_config.amplitude,
            (true, false) => noise_config.amplitude / total_weight,
            (true, true) => 0.0,
        };

        let mut seeds = [0u32; MAX_CUSTOM_OCTAVES];
        for (i, octave) in octave_list.iter().enumerate() {
            seeds[i] = gen_octave_seed(octave.frequency, noise_config.seed);
        }

        let weight_coef = ArchSimd::splat(weight_coef);
        iters.map(move |x| {
            let inputs = x.into_array();
            if octaves == 0 {
                return ArchSimd::zero();
            }

            let (mut state, mut sample): (F::State, ArchSimd<f32>) = Default::default();

            let freq = from_fn(|i| ArchSimd::splat(octave_list[0].frequency[i]));
            let seed = seeds[0];
            let weight = ArchSimd::splat(octave_list[0].weight) * weight_coef;
            let new_sample = S::sample_batch(seed, inputs, freq) * weight;
            if noise_config.initialization {
                (state, sample) = F::initialize_sample(&fractal_config, new_sample);
            } else {
                (state, sample) = F::apply_sample(&fractal_config, state, sample, new_sample);
            }

            for (i, octave) in octave_list
                .iter()
                .enumerate()
                .skip(1)
                .take(octaves.saturating_sub(2))
            {
                let freq = from_fn(|i| ArchSimd::splat(octave.frequency[i]));
                let seed = seeds[i];
                let weight = ArchSimd::splat(octave_list[0].weight) * weight_coef;
                let new_sample = S::sample_batch(seed, inputs, freq) * weight;
                (state, sample) = F::apply_sample(&fractal_config, state, sample, new_sample);
            }

            if noise_config.finalization {
                F::finalize_sample(&fractal_config, state, sample)
            } else {
                sample
            }
        })
    }
}
