use std::marker::PhantomData;

use itertools::izip;

use crate::api::batch::interface::BatchNoise;
use crate::api::configs::*;
use crate::api::defaults::ZeroIter;
use crate::api::methods::NoiseDimension;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{BasicVec, Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::{Dim2, Dim3};

const MAX_FBM_OCTAVES: usize = 32;

/// Helper static function for fbm noise.
#[inline(always)]
pub(crate) fn batch_fbm_noise<
    T: BatchNoise,
    D: NoiseDimension,
    const N: usize,
    XIter,
    YIter,
    ZIter,
>(
    general_config: GeneralBuilderConfig,
    fbm_config: FbmBuilderConfig<D>,
    batch_config: BatchBuilderConfig<XIter, YIter, ZIter>,
) -> impl Iterator<Item = ArchSimd<f32>>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    let octaves = fbm_config.octaves;

    let frequency = fbm_config.scaling * D::FVec::splat(fbm_config.frequency);
    let weight = if general_config.normalization && octaves > 0 {
        fbm_config.normalize_amplitude(general_config.amplitude)
    } else {
        general_config.amplitude
    };

    let mut seeds = [0u32; MAX_FBM_OCTAVES];
    let mut temp_freq = frequency;
    for seed in seeds.iter_mut().take(octaves) {
        *seed = D::octave_seed(temp_freq * fbm_config.scaling, general_config.seed);
        temp_freq *= D::FVec::splat(fbm_config.lacunarity);
    }

    let x_iter = batch_config.x_iter.expect("X Iterator not present!");
    let y_iter = batch_config.y_iter.expect("Y Iterator not present!");
    let z_iter = batch_config.z_iter.expect("Z Iterator not present!");

    let lacunarity = ArchSimd::splat(fbm_config.lacunarity);
    let persistence = ArchSimd::splat(fbm_config.persistence);

    izip!(x_iter, y_iter, z_iter).map(move |(x, y, z)| {
        if octaves == 0 {
            return ArchSimd::zero();
        }

        let seed = seeds[0];

        // I could not think of a better way to make this work for 2D and 3D at the same time.
        let freq_tuple: (ArchSimd<f32>, ArchSimd<f32>, ArchSimd<f32>) = frequency.into();
        let mut x_freq = freq_tuple.0;
        let mut y_freq = freq_tuple.1;
        let mut z_freq = freq_tuple.2;
        let mut weight = ArchSimd::splat(weight);
        let mut result = D::batch::<T, N>(seed, x, y, z, x_freq, y_freq, z_freq) * weight;

        for seed in seeds.iter().take(octaves).skip(1) {
            x_freq *= lacunarity;
            y_freq *= lacunarity;
            z_freq *= lacunarity;
            weight *= persistence;
            result =
                D::batch::<T, N>(*seed, x, y, z, x_freq, y_freq, z_freq).mul_add(weight, result);
        }

        result
    })
}

pub struct FbmBatchBuilder<T: BatchNoise, D: NoiseDimension, const N: usize, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    fbm_config: FbmBuilderConfig<D>,
    batch_config: BatchBuilderConfig<XIter, YIter, ZIter>,
    _noise_type: PhantomData<T>,
}

params_general_builder!(
    FbmBatchBuilder,
    [T: BatchNoise, D: NoiseDimension, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, D, N, XIter, YIter, ZIter]
);

params_grided_seed_builder!(
    FbmBatchBuilder,
    [T: BatchNoise, D: NoiseDimension, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, D, N, XIter, YIter, ZIter]
);

params_fbm_builder!(
    FbmBatchBuilder,
    [T: BatchNoise, D: NoiseDimension, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, D, N, XIter, YIter, ZIter]
);

params_fbm_scaling_2d!(
    FbmBatchBuilder,
    [T: BatchNoise, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [T, Dim2, N, XIter, YIter, ZeroIter<N>]
);

params_fbm_scaling_3d!(
    FbmBatchBuilder,
    [T: BatchNoise, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, Dim3, N, XIter, YIter, ZIter]
);

impl<T: BatchNoise, D: NoiseDimension, const N: usize, XIter, YIter, ZIter>
    FbmBatchBuilder<T, D, N, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    pub fn new(x_iter: XIter, y_iter: YIter, z_iter: ZIter) -> Self {
        Self {
            general_config: Default::default(),
            fbm_config: Default::default(),
            batch_config: BatchBuilderConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
                z_iter: Some(z_iter),
            },
            _noise_type: PhantomData::<T>,
        }
    }

    pub(crate) fn from_configs(
        general_config: GeneralBuilderConfig,
        fbm_config: FbmBuilderConfig<D>,
        batch_config: BatchBuilderConfig<XIter, YIter, ZIter>,
    ) -> Self {
        Self {
            general_config,
            fbm_config,
            batch_config,
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
        batch_fbm_noise::<T, D, N, XIter, YIter, ZIter>(
            self.general_config,
            self.fbm_config,
            self.batch_config,
        )
    });
}
