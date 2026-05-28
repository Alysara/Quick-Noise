use itertools::izip;

use crate::api::batch;
use crate::api::batch::interface::{Batch2D, Batch3D};
use crate::api::configs::*;
use crate::api::methods::NoiseMethod;
use crate::api::parameters::*;
use crate::api::seed::OctaveSeed;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3, VecHorzMax};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::{EmptyIter, ZeroIter};

// ————————————————————————————————————————————————————————————————
// ————— 2D FBM Batch Builder —————————————————————————————————————
// ————————————————————————————————————————————————————————————————

const MAX_FBM_OCTAVES: usize = 32;

impl<const N: usize> Batch2D<N> {
    /// Helper static function for fbm noise.
    #[inline(always)]
    pub(crate) fn fbm_noise<const METHOD: u8, XIter, YIter>(
        general_config: GeneralBuilderConfig,
        fbm_config: FBMBuilderConfig2D,
        batch_config: BatchBuilder2DConfig<XIter, YIter>,
    ) -> impl Iterator<Item = ArchSimd<f32>>
    where
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
    {
        let noise_method = NoiseMethod::from_u8_const(METHOD);

        let octaves = 'outer: {
            let max_octaves = fbm_config.octaves.min(MAX_FBM_OCTAVES);
            let max_scaling = fbm_config.scaling.horizontal_max();
            let mut cur_freq = fbm_config.frequency * max_scaling;
            if cur_freq >= 1.0 || fbm_config.lacunarity >= 1.0 {
                for i in 0..max_octaves {
                    if cur_freq >= 1.0 {
                        break 'outer i;
                    }
                    cur_freq *= fbm_config.lacunarity;
                }
            }
            max_octaves
        };

        let frequency = fbm_config.scaling * fbm_config.frequency;
        let weight = if general_config.normalization && octaves > 0 {
            let mut sum = 0.0;
            let mut cur = 1.0;
            for _ in 0..octaves {
                sum += cur;
                cur *= fbm_config.persistence;
            }
            general_config.amplitude / sum
        } else {
            general_config.amplitude
        };

        let mut seeds = [0u32; 32];
        for i in 0..octaves {
            seeds[i as usize] =
                (fbm_config.frequency * fbm_config.scaling).octave_seed(general_config.seed);
        }

        let x_iter = batch_config.x_iter.expect("X Iterator not present!");
        let y_iter = batch_config.y_iter.expect("Y Iterator not present!");

        let lacunarity = ArchSimd::splat(fbm_config.lacunarity);
        let persistence = ArchSimd::splat(fbm_config.persistence);

        let iter = x_iter.zip(y_iter).map(move |(x, y)| {
            if octaves == 0 {
                return ArchSimd::zero();
            }

            let mut seed = seeds[0];
            let mut result = ArchSimd::zero();
            let mut x_freq = ArchSimd::splat(frequency.x);
            let mut y_freq = ArchSimd::splat(frequency.y);
            let mut weight = ArchSimd::splat(weight);
            result = noise_method
                .batch_2d(seed, x, y, x_freq, y_freq)
                .mul_add(weight, result);

            for i in 1..octaves {
                seed = seeds[i as usize];
                x_freq *= lacunarity;
                y_freq *= lacunarity;
                weight *= persistence;
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
pub struct FBMBatchBuilder2D<const METHOD: u8, const N: usize, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig2D,
    batch_config: BatchBuilder2DConfig<XIter, YIter>,
}

params_general_builder!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter]
);

params_fbm_builder!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter]
);

params_fbm_scaling_2d!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter]
);

params_batch_2d!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter],
    [METHOD, N,], self, x_iter, y_iter, {
        FBMBatchBuilder2D {
            general_config: self.general_config,
            fbm_config: self.fbm_config,
            batch_config: BatchBuilder2DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
            },
        }
    }
);

impl<const METHOD: u8, const N: usize, XIter, YIter> FBMBatchBuilder2D<METHOD, N, XIter, YIter>
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
        Batch2D::<N>::fbm_noise::<METHOD, XIter, YIter>(
            self.general_config,
            self.fbm_config,
            self.batch_config,
        )
    });
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Batch FBM Builder —————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<const N: usize> Batch3D<N> {
    /// Helper static function for fbm noise.
    #[inline(always)]
    pub(crate) fn fbm_noise<const METHOD: u8, XIter, YIter, ZIter>(
        general_config: GeneralBuilderConfig,
        fbm_config: FBMBuilderConfig3D,
        batch_config: BatchBuilder3DConfig<XIter, YIter, ZIter>,
    ) -> impl Iterator<Item = ArchSimd<f32>>
    where
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>,
    {
        let noise_method = NoiseMethod::from_u8_const(METHOD);

        let octaves = 'outer: {
            let max_octaves = fbm_config.octaves.min(MAX_FBM_OCTAVES);
            let max_scaling = fbm_config.scaling.horizontal_max();
            let mut cur_freq = fbm_config.frequency * max_scaling;
            if cur_freq >= 1.0 || fbm_config.lacunarity >= 1.0 {
                for i in 0..max_octaves {
                    if cur_freq >= 1.0 {
                        break 'outer i;
                    }
                    cur_freq *= fbm_config.lacunarity;
                }
            }
            max_octaves
        };

        let frequency = fbm_config.scaling * fbm_config.frequency;
        let weight = if general_config.normalization && octaves == 0 {
            let mut sum = 0.0;
            let mut cur = 1.0;
            for _ in 0..octaves {
                sum += cur;
                cur *= fbm_config.persistence;
            }

            general_config.amplitude / sum
        } else {
            general_config.amplitude
        };

        let mut seeds = [0u32; 32];
        for i in 0..octaves {
            seeds[i as usize] =
                (fbm_config.frequency * fbm_config.scaling).octave_seed(general_config.seed);
        }

        let x_iter = batch_config.x_iter.expect("X Iterator not present!");
        let y_iter = batch_config.y_iter.expect("Y Iterator not present!");
        let z_iter = batch_config.z_iter.expect("Z Iterator not present!");

        let lacunarity = ArchSimd::splat(fbm_config.lacunarity);
        let persistence = ArchSimd::splat(fbm_config.persistence);

        let iter = izip!(x_iter, y_iter, z_iter).map(move |(x, y, z)| {
            if octaves == 0 {
                return ArchSimd::zero();
            }

            let mut seed = seeds[0];
            let mut result = ArchSimd::zero();
            let mut x_freq = ArchSimd::splat(frequency.x);
            let mut y_freq = ArchSimd::splat(frequency.y);
            let mut z_freq = ArchSimd::splat(frequency.z);
            let mut weight = ArchSimd::splat(weight);
            result = noise_method
                .batch_3d(seed, x, y, z, x_freq, y_freq, z_freq)
                .mul_add(weight, result);

            for i in 1..octaves {
                seed = seeds[i as usize];
                x_freq *= lacunarity;
                y_freq *= lacunarity;
                z_freq *= lacunarity;
                weight *= persistence;
                result = noise_method
                    .batch_3d(seed, x, y, z, x_freq, y_freq, z_freq)
                    .mul_add(weight, result);
            }

            result
        });

        iter
    }
}

#[derive(Default)]
pub struct FBMBatchBuilder3D<const METHOD: u8, const N: usize, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig3D,
    batch_config: BatchBuilder3DConfig<XIter, YIter, ZIter>,
}

params_general_builder!(
    FBMBatchBuilder3D,
    [
        const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [METHOD, N, XIter, YIter, ZIter]
);

params_fbm_builder!(
    FBMBatchBuilder3D,
    [
        const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [METHOD, N, XIter, YIter, ZIter]
);

params_fbm_scaling_3d!(
    FBMBatchBuilder3D,
    [
        const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [METHOD, N, XIter, YIter, ZIter]
);

params_batch_3d!(
    FBMBatchBuilder3D,
    [
        const METHOD: u8, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [METHOD, N, XIter, YIter, ZIter], [METHOD, N,], 
    self, x_iter, y_iter, z_iter, {
        FBMBatchBuilder3D {
            general_config: self.general_config,
            fbm_config: self.fbm_config,
            batch_config: BatchBuilder3DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
                z_iter: Some(z_iter),
            }
        }
    }
);

impl<const METHOD: u8, const N: usize, XIter, YIter, ZIter>
    FBMBatchBuilder3D<METHOD, N, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    declare_fill!(self, output, {
        *output = self.build();
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
        Batch3D::<N>::fbm_noise::<METHOD, XIter, YIter, ZIter>(
            self.general_config,
            self.fbm_config,
            self.batch_config,
        )
    });
}
