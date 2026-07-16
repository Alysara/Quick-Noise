use std::array::from_fn;

use crate::noise::combiners::Combiner;
use crate::api::batch::interface::{BatchNoise, BatchGenerator, DimIter, DimTuple};
use crate::api::configs::*;
use crate::api::seed::gen_octave_seed;
use crate::simd::arch_simd::ArchSimd;

const MAX_FBM_OCTAVES: usize = 32;

impl<const D: usize, C: Combiner, G: BatchGenerator<D>> BatchNoise<D, C, G> {
    #[inline(always)]
    pub fn sample<I: DimIter<D>>(
        noise_config: NoiseConfig<D>,
        combiner_config: C::Config,
        iters: I,
    ) -> impl Iterator<Item = ArchSimd<f32>> {
        let octaves = noise_config.octaves;

        let frequency: [_; D] = from_fn(|i| noise_config.scaling[i] * noise_config.frequency);
        let weight = if noise_config.normalization && octaves > 0 {
            noise_config.normalize_amplitude(noise_config.amplitude)
        } else {
            noise_config.amplitude
        };

        let mut seeds = [0u32; MAX_FBM_OCTAVES];
        let mut temp_freq = frequency;
        for seed in seeds.iter_mut().take(octaves) {
            *seed = gen_octave_seed(temp_freq, noise_config.seed);
            temp_freq
                .iter_mut()
                .for_each(|x| *x *= noise_config.lacunarity);
        }

        let lacunarity = ArchSimd::splat(noise_config.lacunarity);
        let persistence = ArchSimd::splat(noise_config.persistence);

        iters.map(move |x| {
            let inputs = x.into_array();
            if octaves == 0 {
                return ArchSimd::zero();
            }
            let (mut state, mut sample): (C::State, ArchSimd<f32>) = Default::default();

            let seed = seeds[0];
            let mut weight = ArchSimd::splat(weight);
            let mut freq = from_fn(|i| ArchSimd::splat(frequency[i]));
            let new_sample = G::sample_batch(seed, inputs, freq) * weight;
            
            if noise_config.initialize {
                (state, sample) = C::initialize_sample(&combiner_config, new_sample);
            } else {
                (state, sample) = C::apply_sample(&combiner_config, state, sample, new_sample);
            }

            for seed in seeds.iter().take(octaves).skip(1) {
                freq.iter_mut().for_each(|x| *x *= lacunarity);
                if C::WEIGHT_DECAY {
                    weight *= persistence;
                }

                let new_sample = G::sample_batch(*seed, inputs, freq) * weight;
                (state, sample) = C::apply_sample(&combiner_config, state, sample, new_sample);
            }

            if noise_config.finalize {
                C::finalize_sample(&combiner_config, state, sample)
            } else {
                sample
            }
        })
    }
}

