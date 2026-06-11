use std::iter::zip;
use std::marker::PhantomData;

use either::Either;
use itertools::izip;

use crate::api::batch;
use crate::api::batch::fbm::{FbmBatchBuilder, batch_fbm_noise};
use crate::api::batch::interface::{Batch2D, Batch3D};
use crate::api::configs::*;
use crate::api::methods::{NoiseDim, NoiseDimension};
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
    pub(crate) fn new(
        grid_config: GridConfig<D>,
        x_iter: XIter,
        y_iter: YIter,
        z_iter: ZIter,
    ) -> Self {
        Self {
            grid_config,
            general_config: Default::default(),
            fbm_config: Default::default(),
            warp_config: Default::default(),
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
        let (x_grid_iter, y_grid_iter, z_grid_iter) = D::get_iters::<X, Y, Z, N>(self.grid_config);
        let strength = ArchSimd::splat(self.warp_config.strength);

        let x_iter = Some(
            zip(self.batch_config.x_iter.unwrap(), x_grid_iter)
                .map(move |(x, y)| x.mul_add(strength, y)),
        );
        let y_iter = Some(
            zip(self.batch_config.y_iter.unwrap(), y_grid_iter)
                .map(move |(x, y)| x.mul_add(strength, y)),
        );
        let z_iter = Some(
            zip(self.batch_config.z_iter.unwrap(), z_grid_iter)
                .map(move |(x, y)| x.mul_add(strength, y)),
        );

        FbmBatchBuilder::<T, D, N, _, _, _>::from_configs(
            self.general_config,
            self.fbm_config,
            BatchBuilderConfig {
                x_iter,
                y_iter,
                z_iter,
            },
        )
        .into_iter()
    });
}
