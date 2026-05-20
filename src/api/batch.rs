use itertools::izip;

use crate::api::builders::*;
use crate::math::random::Random;
use crate::math::vec::Vec2;
use crate::perlin::Perlin;
use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::simplex::Simplex;

/// Static struct for calling independent batches of noise.
pub struct Batch2D<const N: usize> {}

/// Functions needed to be supported for each type of batched noise.
pub(crate) trait BatchNoise {
    fn batch_2d(
        seed: u64,
        x_input: ArchSimd<f32>,
        y_input: ArchSimd<f32>,
        freq: f32,
    ) -> ArchSimd<f32>;
}

impl<const N: usize> Batch2D<N> {
    /// Helper static function for fbm noise.
    #[inline(always)]
    pub(crate) fn fbm_noise<Noise: BatchNoise>(
        seed: u64,
        x_input: ArchSimd<f32>,
        y_input: ArchSimd<f32>,
        octaves: u32,
        frequency: f32,
        lacunarity: f32,
        persistence: f32,
    ) -> ArchSimd<f32> {
        let mut output = ArchSimd::<f32>::zero();
        let mut cur_freq = frequency;
        let persistence_vec = ArchSimd::splat(persistence);
        let mut weight_vec = ArchSimd::splat(1.0);
        for _ in 0..octaves {
            let octave_noise = Noise::batch_2d(seed, x_input, y_input, cur_freq);
            output = octave_noise.mul_add(weight_vec, output);
            weight_vec *= persistence_vec;
            cur_freq *= lacunarity;
        }
        output
    }

    /// Perlin FBM noise
    pub fn perlin() -> FBMBatchBuilder2D<{ NoiseMethod::PERLIN_U8 }, N, EmptyIter, EmptyIter> {
        FBMBatchBuilder2D::default()
    }

    /// Value FBM noise
    pub fn value() -> FBMBatchBuilder2D<{ NoiseMethod::VALUE_U8 }, N, EmptyIter, EmptyIter> {
        FBMBatchBuilder2D::default()
    }

    /// Simplex FBM noise
    pub fn simplex() -> FBMBatchBuilder2D<{ NoiseMethod::SIMPLEX_U8 }, N, EmptyIter, EmptyIter>
    {
        FBMBatchBuilder2D::default()
    }

    /// Cellular FBM noise
    pub fn cellular() -> FBMBatchBuilder2D<{ NoiseMethod::CELLULAR_U8 }, N, EmptyIter, EmptyIter>
    {
        FBMBatchBuilder2D::default()
    }
}

#[derive(Default)]
pub struct FBMBatchBuilder2D<const METHOD: u8, const N: usize, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig,
    dim_config: Builder2DConfig,
    batch_config: BatchBuilder2DConfig<XIter, YIter>,
}

impl<const METHOD: u8, const N: usize, XIter, YIter> FBMBatchBuilder2D<METHOD, N, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    const NOISE_TYPE: NoiseMethod = NoiseMethod::from_u8_const(METHOD);
}

apply_general_builder_params!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter]
);

apply_fbm_builder_params!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter]
);

apply_builder_2d_params!(
    FBMBatchBuilder2D,
    [const METHOD: u8, const N: usize, XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, N, XIter, YIter]
);

impl<const METHOD: u8, const N: usize, XIter, YIter> BatchBuilder2D
    for FBMBatchBuilder2D<METHOD, N, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    type OutputBuilder<NewXIter, NewYIter>
        = FBMBatchBuilder2D<METHOD, N, NewXIter, NewYIter>
    where
        NewXIter: Iterator<Item = ArchSimd<f32>>,
        NewYIter: Iterator<Item = ArchSimd<f32>>;

    fn input_iters<NewXIter, NewYIter>(
        self,
        x_iter: NewXIter,
        y_iter: NewYIter,
    ) -> Self::OutputBuilder<NewXIter, NewYIter>
    where
        NewXIter: Iterator<Item = ArchSimd<f32>>,
        NewYIter: Iterator<Item = ArchSimd<f32>>,
    {
        FBMBatchBuilder2D {
            general_config: self.general_config,
            fbm_config: self.fbm_config,
            dim_config: self.dim_config,
            batch_config: BatchBuilder2DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
            },
        }
    }
}

// apply_fbm_builder_parms!(FBMSimplexBatch2D, [const N: usize, XIter, YIter], [N, XIter, YIter]);
// apply_builder_2d_parms!(FBMSimplexBatch2D, [const N: usize, XIter, YIter], [N, XIter, YIter]);

impl<const N: usize, XIter, YIter> BuilderExecute<f32, N>
    for FBMBatchBuilder2D<{ NoiseMethod::SIMPLEX_U8 }, N, XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    #[inline(always)]
    fn fill(self, output: &mut SimdArray<f32, N>) {
        *output = self.build();
    }

    #[inline(always)]
    fn build(self) -> SimdArray<f32, N> {
        self.into_iter().collect()
    }

    #[inline(always)]
    fn into_iter(self) -> impl Iterator<Item = ArchSimd<f32>> {
        let x_iter = self.batch_config.x_iter.expect("X iterator not set!");
        let y_iter = self.batch_config.y_iter.expect("Y iterator not set!");

        izip!(x_iter, y_iter).map(move |(x, y)| {
            Batch2D::<N>::fbm_noise::<Simplex>(
                self.general_config.seed,
                x,
                y,
                self.fbm_config.octaves,
                self.general_config.frequency,
                self.fbm_config.lacunarity,
                self.fbm_config.persistence,
            )
        })
    }
}

// impl<const N: usize, XIter, YIter> BuilderExecute<f32, N>
//     for FBMBatchBuilder2D<{ NoiseMethod::PERLIN_U8 }, N, XIter, YIter>
// where
//     XIter: Iterator<Item = ArchSimd<f32>>,
//     YIter: Iterator<Item = ArchSimd<f32>>,
// {
//     #[inline(always)]
//     fn fill(self, output: &mut SimdArray<f32, N>) {
//         *output = self.build();
//     }

//     #[inline(always)]
//     fn build(self) -> SimdArray<f32, N> {
//         self.into_iter().collect()
//     }

//     #[inline(always)]
//     fn into_iter(self) -> impl Iterator<Item = ArchSimd<f32>> {
//         let x_iter = self.batch_config.x_iter.expect("X iterator not set!");
//         let y_iter = self.batch_config.y_iter.expect("Y iterator not set!");

//         izip!(x_iter, y_iter).map(move |(x, y)| {
//             Batch2D::<N>::fbm_noise::<Perlin>(
//                 self.general_config.seed,
//                 x,
//                 y,
//                 self.fbm_config.octaves,
//                 self.general_config.frequency,
//                 self.fbm_config.lacunarity,
//                 self.fbm_config.persistence,
//             )
//         })
//     }
// }
