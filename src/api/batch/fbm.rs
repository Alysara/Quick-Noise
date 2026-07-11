use std::array::from_fn;
use std::marker::PhantomData;

use itertools::{Zip, izip, multizip};

use crate::api::batch::interface::{BatchNoise, BatchNoiseImpl, DimIter, DimTuple};
use crate::api::configs::*;
use crate::api::defaults::ZeroIter;
use crate::api::parameters::*;
use crate::api::seed::gen_octave_seed;
use crate::math::random::Random;
use crate::math::vec::{BasicVec, Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;

const MAX_FBM_OCTAVES: usize = 32;

/// Helper static function for fbm noise.
#[inline(always)]
pub(crate) fn batch_fbm_noise<const D: usize, T: BatchNoiseImpl<D>, I: DimIter<D>>(
    general_config: GeneralConfig,
    fbm_config: FbmConfig<D>,
    iters: I,
) -> impl Iterator<Item = ArchSimd<f32>> {
    let octaves = fbm_config.octaves;

    let frequency: [_; D] = from_fn(|i| fbm_config.scaling[i] * fbm_config.frequency);
    let weight = if general_config.normalization && octaves > 0 {
        fbm_config.normalize_amplitude(general_config.amplitude)
    } else {
        general_config.amplitude
    };

    let mut seeds = [0u32; MAX_FBM_OCTAVES];
    let mut temp_freq = frequency;
    for seed in seeds.iter_mut().take(octaves) {
        *seed = gen_octave_seed(temp_freq, general_config.seed);
        temp_freq
            .iter_mut()
            .for_each(|x| *x *= fbm_config.lacunarity);
    }

    let lacunarity = ArchSimd::splat(fbm_config.lacunarity);
    let persistence = ArchSimd::splat(fbm_config.persistence);

    iters.map(move |x| {
        let inputs = x.into_array();
        if octaves == 0 {
            return ArchSimd::zero();
        }

        let seed = seeds[0];

        // I could not think of a better way to make this work for 2D and 3D at the same time.
        let mut weight = ArchSimd::splat(weight);
        let mut freq = from_fn(|i| ArchSimd::splat(frequency[i]));
        let mut result = T::sample_batch(seed, inputs, freq) * weight;

        for seed in seeds.iter().take(octaves).skip(1) {
            freq.iter_mut().for_each(|x| *x *= lacunarity);
            weight *= persistence;
            result += T::sample_batch(*seed, inputs, freq) * weight;
        }

        result
    })
}

pub struct BatchBuilder<const D: usize, T: BatchNoiseImpl<D>, I: DimIter<D>> {
    general_config: GeneralConfig,
    fbm_config: FbmConfig<D>,
    iters: I,
    _noise_type: PhantomData<T>,
}

params_general_builder!(BatchBuilder, [const D: usize, T: BatchNoiseImpl<D>, I: DimIter<D>], [D, T, I]);
params_grided_seed_builder!(BatchBuilder, [const D: usize, T: BatchNoiseImpl<D>, I: DimIter<D>], [D, T, I]);
params_fbm_builder!(BatchBuilder, [const D: usize, T: BatchNoiseImpl<D>, I: DimIter<D>], [D, T, I]);
params_fbm_scaling_2d!(BatchBuilder, [T: BatchNoiseImpl<2>, I: DimIter<2>], [2, T, I]);
params_fbm_scaling_3d!(BatchBuilder, [T: BatchNoiseImpl<3>, I: DimIter<3>], [3, T, I]);

impl<T, X, Y> BatchBuilder<2, T, Zip<(X, Y)>>
where
    T: BatchNoiseImpl<2>,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y)>: DimIter<2>,
{
    pub fn new(x_iter: X, y_iter: Y) -> Self {
        Self {
            general_config: Default::default(),
            fbm_config: Default::default(),
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<T>,
        }
    }

    pub fn from_configs(
        general_config: GeneralConfig,
        fbm_config: FbmConfig<2>,
        x_iter: X,
        y_iter: Y,
    ) -> Self {
        Self {
            general_config,
            fbm_config,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<T>,
        }
    }
}

impl<T, X, Y, Z> BatchBuilder<3, T, Zip<(X, Y, Z)>>
where
    T: BatchNoiseImpl<3>,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Z: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y, Z)>: DimIter<3>,
{
    pub fn new(x_iter: X, y_iter: Y, z_iter: Z) -> Self {
        Self {
            general_config: Default::default(),
            fbm_config: Default::default(),
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<T>,
        }
    }

    pub fn from_configs(
        general_config: GeneralConfig,
        fbm_config: FbmConfig<3>,
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> Self {
        Self {
            general_config,
            fbm_config,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<T>,
        }
    }
}

impl<const D: usize, T: BatchNoiseImpl<D>, I: DimIter<D>> BatchBuilder<D, T, I> {
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
        batch_fbm_noise::<D, T, I>(self.general_config, self.fbm_config, self.iters)
    });
}
