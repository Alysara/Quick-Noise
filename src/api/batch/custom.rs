use std::marker::PhantomData;

use itertools::izip;

use crate::api::batch::interface::BatchNoise;
use crate::api::configs::*;
use crate::api::methods::{NoiseDimension, Octave};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;

const MAX_CUSTOM_OCTAVES: usize = 32;

/// Helper static function for custom noise.
#[inline(always)]
pub(crate) fn custom_noise<
    'a,
    T: BatchNoise,
    D: NoiseDimension,
    const N: usize,
    XIter,
    YIter,
    ZIter,
>(
    general_config: GeneralBuilderConfig,
    batch_config: BatchBuilderConfig<XIter, YIter, ZIter>,
    custom_config: CustomBuilderConfig<'a, D>,
) -> impl Iterator<Item = ArchSimd<f32>>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    let octaves = custom_config.octave_list.len();

    let weight_coef = if general_config.normalization && octaves > 0 {
        custom_config.normalize_batch_amplitude(general_config.amplitude)
    } else {
        1.0
    };

    let mut seeds = [0u32; MAX_CUSTOM_OCTAVES];
    for (i, octave) in custom_config.octave_list.iter().enumerate() {
        seeds[i] = D::octave_seed(octave.frequency, general_config.seed);
    }

    let x_iter = batch_config.x_iter.expect("X Iterator not present!");
    let y_iter = batch_config.y_iter.expect("Y Iterator not present!");
    let z_iter = batch_config.z_iter.expect("Z Iterator not present!");

    izip!(x_iter, y_iter, z_iter).map(move |(x, y, z)| {
        if octaves == 0 {
            return ArchSimd::zero();
        }

        let mut result = ArchSimd::zero();

        for (i, octave) in custom_config.octave_list.iter().enumerate() {
            let seed = seeds[i];

            let freq_tuple: (ArchSimd<f32>, ArchSimd<f32>, ArchSimd<f32>) = octave.frequency.into();
            let weight = ArchSimd::splat(octave.weight * weight_coef);

            result = D::batch::<T, N>(seed, x, y, z, freq_tuple.0, freq_tuple.1, freq_tuple.2)
                .mul_add(weight, result);
        }

        result
    })
}

pub struct CustomBatchBuilder<
    'a,
    T: BatchNoise,
    D: NoiseDimension,
    const N: usize,
    XIter,
    YIter,
    ZIter,
> where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    batch_config: BatchBuilderConfig<XIter, YIter, ZIter>,
    custom_config: CustomBuilderConfig<'a, D>,
    _noise_type: PhantomData<T>,
}

params_general_builder!(
    CustomBatchBuilder,
    ['a, T: BatchNoise, D: NoiseDimension, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    ['a, T, D, N, XIter, YIter, ZIter]
);

params_grided_seed_builder!(
    CustomBatchBuilder,
    ['a, T: BatchNoise, D: NoiseDimension, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    ['a, T, D, N, XIter, YIter, ZIter]
);

impl<'a, T: BatchNoise, D: NoiseDimension, const N: usize, XIter, YIter, ZIter>
    CustomBatchBuilder<'a, T, D, N, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    pub fn new(octave_list: &'a [Octave<D>], x_iter: XIter, y_iter: YIter, z_iter: ZIter) -> Self {
        Self {
            general_config: Default::default(),
            custom_config: CustomBuilderConfig { octave_list },
            batch_config: BatchBuilderConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
                z_iter: Some(z_iter),
            },
            _noise_type: PhantomData::<T>,
        }
    }

    declare_fill!(self, output, {
        let mut i = 0;
        self.into_iter().for_each(|x| {
            output.store_simd(i, x);
            i += ArchSimd::<f32>::LANES;
        });
    });

    declare_fill_onto!(self, output, {
        let mut i = 0;
        self.into_iter().for_each(|x| {
            let cur = output.load_simd(i);
            output.store_simd(i, cur + x);
            i += ArchSimd::<f32>::LANES;
        });
    });

    declare_build!(self, { self.into_iter().collect() });

    declare_into_iter!(self, {
        custom_noise::<T, D, N, XIter, YIter, ZIter>(
            self.general_config,
            self.batch_config,
            self.custom_config,
        )
    });
}
