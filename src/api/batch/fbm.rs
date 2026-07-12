use std::array::from_fn;
use std::marker::PhantomData;

use itertools::{Zip, multizip};

use crate::Fractal;
use crate::api::batch::interface::{BatchNoise, DimIter, DimTuple};
use crate::api::configs::*;
use crate::api::parameters::*;
use crate::api::seed::gen_octave_seed;
use crate::math::random::Random;
use crate::simd::arch_simd::ArchSimd;

const MAX_FBM_OCTAVES: usize = 32;

/// Helper static function for fbm noise.
#[inline(always)]
pub(crate) fn sample_batch<const D: usize, F: Fractal, T: BatchNoise<D>, I: DimIter<D>>(
    noise_config: NoiseConfig<D>,
    fractal_config: F::Config,
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

        let seed = seeds[0];

        // I could not think of a better way to make this work for 2D and 3D at the same time.
        let mut weight = ArchSimd::splat(weight);
        let mut freq = from_fn(|i| ArchSimd::splat(frequency[i]));

        let result = T::sample_batch(seed, inputs, freq) * weight;
        let (mut state, mut sample) = F::initialize(&fractal_config, result);

        for seed in seeds.iter().take(octaves).skip(1) {
            freq.iter_mut().for_each(|x| *x *= lacunarity);
            if F::WEIGHT_DECAY {
                weight *= persistence;
            }

            let new_sample = T::sample_batch(*seed, inputs, freq) * weight;
            (state, sample) = F::sample(&fractal_config, state, sample, new_sample);
        }

        F::finalize(&fractal_config, state, sample)
    })
}

pub struct BatchNoiseBuilder<const D: usize, F: Fractal, T: BatchNoise<D>, I: DimIter<D>> {
    noise_config: NoiseConfig<D>,
    fractal_config: F::Config,
    iters: I,
    _noise_type: PhantomData<T>,
}

params_noise_builder!(BatchNoiseBuilder, [const D: usize, F: Fractal, T: BatchNoise<D>, I: DimIter<D>], [D, F, T, I]);
params_grid_seed_builder!(BatchNoiseBuilder, [const D: usize, F: Fractal, T: BatchNoise<D>, I: DimIter<D>], [D, F, T, I]);
params_noise_scaling_2d!(BatchNoiseBuilder, [F: Fractal, T: BatchNoise<2>, I: DimIter<2>], [2, F, T, I]);
params_noise_scaling_3d!(BatchNoiseBuilder, [F: Fractal, T: BatchNoise<3>, I: DimIter<3>], [3, F, T, I]);

impl<T, F, X, Y> BatchNoiseBuilder<2, F, T, Zip<(X, Y)>>
where
    T: BatchNoise<2>,
    F: Fractal,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y)>: DimIter<2>,
{
    pub fn new(x_iter: X, y_iter: Y) -> Self {
        Self {
            noise_config: Default::default(),
            fractal_config: Default::default(),
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<T>,
        }
    }

    pub fn from_configs(
        noise_config: NoiseConfig<2>,
        fractal_config: F::Config,
        x_iter: X,
        y_iter: Y,
    ) -> Self {
        Self {
            noise_config,
            fractal_config,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<T>,
        }
    }
}

impl<T, F, X, Y, Z> BatchNoiseBuilder<3, F, T, Zip<(X, Y, Z)>>
where
    T: BatchNoise<3>,
    F: Fractal,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Z: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y, Z)>: DimIter<3>,
{
    pub fn new(x_iter: X, y_iter: Y, z_iter: Z) -> Self {
        Self {
            noise_config: Default::default(),
            fractal_config: Default::default(),
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<T>,
        }
    }

    pub fn from_configs(
        noise_config: NoiseConfig<3>,
        fractal_config: F::Config,
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> Self {
        Self {
            noise_config,
            fractal_config,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<T>,
        }
    }
}

impl<const D: usize, F: Fractal, T: BatchNoise<D>, I: DimIter<D>> BatchNoiseBuilder<D, F, T, I> {
    declare_fill!(self, output, {
        let mut i = 0;
        self.into_iter().for_each(|x| {
            x.copy_to_slice(&mut output[i..]);
            i += ArchSimd::<f32>::LANES;
        });
    });

    declare_fill_onto!(self, output, {
        let mut i = 0;
        self.into_iter().for_each(|x| {
            let cur = ArchSimd::from_slice(&output[i..]) + x;
            cur.copy_to_slice(&mut output[i..]);
            i += ArchSimd::<f32>::LANES;
        });
    });

    // declare_build!(self, { self.into_iter().collect() });

    declare_into_iter!(self, {
        sample_batch::<D, F, T, I>(
            self.noise_config,
            self.fractal_config,
            self.iters,
        )
    });
}
