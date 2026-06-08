use std::iter::zip;
use std::marker::PhantomData;

use itertools::izip;

use crate::api::batch;
use crate::api::batch::fbm::batch_fbm_noise;
use crate::api::batch::interface::{Batch2D, Batch3D};
use crate::api::configs::*;
use crate::api::methods::NoiseDimension;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::{BatchNoise, EmptyIter, Grid2D, Grid3D, ZeroIter};

pub struct FbmGridWarpBuilder<
    T: BatchNoise,
    D: NoiseDimension,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
    XIter,
    YIter,
    ZIter,
> where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    grid_config: GridConfig<D>,
    general_config: GeneralBuilderConfig,
    fbm_config: FbmBuilderConfig<D>,
    batch_config: BatchBuilderConfig<XIter, YIter, ZIter>,
    warp_config: WarpBuilderConfig,
    _noise_type: PhantomData<T>,
}

params_general_builder!(
    FbmGridWarpBuilder,
    [T: BatchNoise, D: NoiseDimension, const X: usize, const Y: usize, const Z: usize, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, D, X, Y, Z, N, XIter, YIter, ZIter]
);

params_fbm_builder!(
    FbmGridWarpBuilder,
    [T: BatchNoise, D: NoiseDimension, const X: usize, const Y: usize, const Z: usize, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, D, X, Y, Z, N, XIter, YIter, ZIter]
);

params_warp_builder!(
    FbmGridWarpBuilder,
    [T: BatchNoise, D: NoiseDimension, const X: usize, const Y: usize, const Z: usize, const N: usize,
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>,
        ZIter: Iterator<Item = ArchSimd<f32>>
    ],
    [T, D, X, Y, Z, N, XIter, YIter, ZIter]
);

impl<
    T: BatchNoise,
    D: NoiseDimension,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
    XIter,
    YIter,
    ZIter,
> FbmGridWarpBuilder<T, D, X, Y, Z, N, XIter, YIter, ZIter>
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
        let grid = Grid2D::from_config(self.grid_config);
        let x_iter = zip(Grid2D::<X, Y, N>::x_iter(), self.batch_config.x_iter).map(|(x, y)| x + y);

        batch_fbm_noise::<T, D, N, XIter, YIter, ZIter>(
            self.general_config,
            self.fbm_config,
            self.batch_config,
        )
    });
}
