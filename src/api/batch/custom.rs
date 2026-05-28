use itertools::izip;

use crate::api::batch;
use crate::api::batch::interface::{Batch2D, Batch3D};
use crate::api::configs::*;
use crate::api::methods::NoiseMethod;
use crate::api::parameters::*;
use crate::api::seed::OctaveSeed;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3, VecHorzMax};
use crate::perlin::{Octave2D, Octave3D};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::{EmptyIter, ZeroIter};

// ————————————————————————————————————————————————————————————————
// ————— 2D Custom Batch Builder ——————————————————————————————————
// ————————————————————————————————————————————————————————————————

const MAX_CUSTOM_OCTAVES: usize = 32;

impl<const N: usize> Batch2D<N> {
    /// Helper static function for fbm noise.
    #[inline(always)]
    pub(crate) fn custom_noise<'a, const METHOD: u8, XIter, YIter>(
        general_config: GeneralBuilderConfig,
        batch_config: BatchBuilder2DConfig<XIter, YIter>,
        custom_config: CustomBuilderConfig<'a, Octave2D>,
    ) -> impl Iterator<Item = ArchSimd<f32>>
    where
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
    {
        let noise_method = NoiseMethod::from_u8_const(METHOD);
        let octaves = custom_config.octave_list.len().min(MAX_CUSTOM_OCTAVES);

        let weight_ceof = if general_config.normalization {
            let mut sum = 0.0;
            for i in 0..octaves {
                sum += custom_config.octave_list[i].weight;
            }
            if sum == 0.0 {
                0.0
            } else {
                general_config.amplitude / sum
            }
        } else {
            general_config.amplitude
        };

        let mut seeds = [0u32; MAX_CUSTOM_OCTAVES];
        for i in 0..octaves {
            let cur = custom_config.octave_list[i];
            seeds[i] = cur.frequency.octave_seed(general_config.seed);
        }

        let x_iter = batch_config.x_iter.expect("X Iterator not present!");
        let y_iter = batch_config.y_iter.expect("Y Iterator not present!");

        let weight_coef_vec = ArchSimd::splat(weight_ceof);
        let iter = x_iter.zip(y_iter).map(move |(x, y)| {
            let mut result = ArchSimd::zero();
            for i in 0..octaves {
                let cur_octave: Octave2D = custom_config.octave_list[i];
                let seed = seeds[0];
                let x_freq = ArchSimd::splat(cur_octave.frequency.x);
                let y_freq = ArchSimd::splat(cur_octave.frequency.y);
                let weight = ArchSimd::splat(cur_octave.weight) * weight_coef_vec;
                result = noise_method
                    .batch_2d(seed, x, y, x_freq, y_freq)
                    .mul_add(weight, result);
            }

            result
        });

        iter
    }
}

#[derive(Default)]
pub struct CustomBatchBuilder2D<'a, const METHOD: u8, const N: usize, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    custom_config: CustomBuilderConfig<'a, Octave2D>,
    batch_config: BatchBuilder2DConfig<XIter, YIter>,
}

params_general_builder!(
    CustomBatchBuilder2D,
    ['a, const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    ['a, METHOD, N, XIter, YIter]
);

params_custom_builder_2d!(
    CustomBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter],
    self, octave_list, {
        CustomBatchBuilder2D {
            general_config: self.general_config,
            batch_config: self.batch_config,
            custom_config: CustomBuilderConfig {
                octave_list: octave_list,
            },
        }
    }
);

params_batch_2d!(
    CustomBatchBuilder2D,
    ['a, const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    ['a, METHOD, N, XIter, YIter],
    ['a, METHOD, N,], self, x_iter, y_iter, {
        CustomBatchBuilder2D {
            general_config: self.general_config,
            custom_config: self.custom_config,
            batch_config: BatchBuilder2DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
            },
        }
    }
);

impl<'a, const METHOD: u8, const N: usize, XIter, YIter>
    CustomBatchBuilder2D<'a, METHOD, N, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
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
        Batch2D::<N>::custom_noise::<METHOD, XIter, YIter>(
            self.general_config,
            self.batch_config,
            self.custom_config,
        )
    });
}

impl<const N: usize> Batch3D<N> {
    /// Helper static function for fbm noise.
    #[inline(always)]
    pub(crate) fn custom_noise<'a, const METHOD: u8, XIter, YIter, ZIter>(
        general_config: GeneralBuilderConfig,
        batch_config: BatchBuilder3DConfig<XIter, YIter, ZIter>,
        custom_config: CustomBuilderConfig<'a, Octave3D>,
    ) -> impl Iterator<Item = ArchSimd<f32>>
    where
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>,
    {
        let noise_method = NoiseMethod::from_u8_const(METHOD);
        let octaves = custom_config.octave_list.len().min(MAX_CUSTOM_OCTAVES);

        let weight_ceof = if general_config.normalization {
            let mut sum = 0.0;
            for i in 0..octaves {
                sum += custom_config.octave_list[i].weight;
            }
            if sum == 0.0 {
                0.0
            } else {
                general_config.amplitude / sum
            }
        } else {
            general_config.amplitude
        };

        let mut seeds = [0u32; MAX_CUSTOM_OCTAVES];
        for i in 0..octaves {
            let cur = custom_config.octave_list[i];
            seeds[i] = cur.frequency.octave_seed(general_config.seed);
        }

        let x_iter = batch_config.x_iter.expect("X Iterator not present!");
        let y_iter = batch_config.y_iter.expect("Y Iterator not present!");
        let z_iter = batch_config.z_iter.expect("Y Iterator not present!");

        let weight_coef_vec = ArchSimd::splat(weight_ceof);
        let iter = izip!(x_iter, y_iter, z_iter).map(move |(x, y, z)| {
            let mut result = ArchSimd::zero();
            for i in 0..octaves {
                let cur_octave: Octave3D = custom_config.octave_list[i];
                let seed = seeds[0];
                let x_freq = ArchSimd::splat(cur_octave.frequency.x);
                let y_freq = ArchSimd::splat(cur_octave.frequency.y);
                let z_freq = ArchSimd::splat(cur_octave.frequency.z);
                let weight = ArchSimd::splat(cur_octave.weight) * weight_coef_vec;
                result = noise_method
                    .batch_3d(seed, x, y, z, x_freq, y_freq, z_freq)
                    .mul_add(weight, result);
            }

            result
        });

        iter
    }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Custom Batch Builder ——————————————————————————————————
// ————————————————————————————————————————————————————————————————

#[derive(Default)]
pub struct CustomBatchBuilder3D<'a, const METHOD: u8, const N: usize, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    custom_config: CustomBuilderConfig<'a, Octave3D>,
    batch_config: BatchBuilder3DConfig<XIter, YIter, ZIter>,
}

params_general_builder!(
    CustomBatchBuilder3D,
    [
        'a, const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    ['a, METHOD, N, XIter, YIter, ZIter]
);

params_batch_3d!(
    CustomBatchBuilder3D,
    [
        'a, const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    ['a, METHOD, N, XIter, YIter, ZIter], ['a, METHOD, N,],
    self, x_iter, y_iter, z_iter, {
        CustomBatchBuilder3D {
            general_config: self.general_config,
            custom_config: self.custom_config,
            batch_config: BatchBuilder3DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
                z_iter: Some(z_iter),
            }
        }
    }
);

params_custom_builder_3d!(
    CustomBatchBuilder3D,
    [
        const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [METHOD, N, XIter, YIter, ZIter],
    self, octave_list, {
        CustomBatchBuilder3D {
            general_config: self.general_config,
            batch_config: self.batch_config,
            custom_config: CustomBuilderConfig {
                octave_list: octave_list,
            },
        }
    }
);

impl<'a, const METHOD: u8, const N: usize, XIter, YIter, ZIter>
    CustomBatchBuilder3D<'a, METHOD, N, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
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
        Batch3D::<N>::custom_noise::<METHOD, XIter, YIter, ZIter>(
            self.general_config,
            self.batch_config,
            self.custom_config,
        )
    });
}
