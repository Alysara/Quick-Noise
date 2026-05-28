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
use crate::{EmptyIter, Grid2D, Grid3D, ZeroIter};

#[derive(Default)]
pub struct FBMWarpBuilder2D<
    const METHOD: u8,
    const X: usize,
    const Y: usize,
    const N: usize,
    XIter,
    YIter,
> where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    grid_config: GridConfig2D,
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig2D,
    batch_config: BatchBuilder2DConfig<XIter, YIter>,
    warp_config: WarpBuilderConfig,
}

params_general_builder!(
    FBMWarpBuilder2D,
    [const METHOD: u8, const X: usize, const Y: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, N, XIter, YIter]
);

params_warp_builder!(
    FBMWarpBuilder2D,
    [const METHOD: u8, const X: usize, const Y: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, N, XIter, YIter]
);

params_fbm_builder!(
    FBMWarpBuilder2D,
    [const METHOD: u8, const X: usize, const Y: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, N, XIter, YIter]
);

params_fbm_scaling_2d!(
    FBMWarpBuilder2D,
    [const METHOD: u8, const X: usize, const Y: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, N, XIter, YIter]
);

params_batch_2d!(
    FBMWarpBuilder2D,
    [const METHOD: u8, const X: usize, const Y: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, N, XIter, YIter],
    [METHOD, X, Y, N,], self, x_iter, y_iter, {
        FBMWarpBuilder2D {
            grid_config: self.grid_config,
            general_config: self.general_config,
            fbm_config: self.fbm_config,
            warp_config: self.warp_config,
            batch_config: BatchBuilder2DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
            },
        }
    }
);

impl<const METHOD: u8, const X: usize, const Y: usize, const N: usize, XIter, YIter>
    FBMWarpBuilder2D<METHOD, X, Y, N, XIter, YIter>
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
        let grid = Grid2D::<X, Y, N>::from_config(self.grid_config);
        let x_iter = self.batch_config.x_iter.expect("X Iter not present!");
        let y_iter = self.batch_config.y_iter.expect("Y Iter not present!");

        let strength = ArchSimd::splat(self.warp_config.strength);
        let new_x_iter = x_iter
            .zip(grid.x_iter())
            .map(move |(a, b)| a.mul_add(strength, b));
        let new_y_iter = y_iter
            .zip(grid.y_iter())
            .map(move |(a, b)| a.mul_add(strength, b));

        let batch_config = BatchBuilder2DConfig {
            x_iter: Some(new_x_iter),
            y_iter: Some(new_y_iter),
        };

        Batch2D::<N>::fbm_noise::<METHOD, _, _>(self.general_config, self.fbm_config, batch_config)
    });
}

#[derive(Default)]
pub struct FBMWarpBuilder3D<
    const METHOD: u8,
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
    grid_config: GridConfig3D,
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig3D,
    batch_config: BatchBuilder3DConfig<XIter, YIter, ZIter>,
    warp_config: WarpBuilderConfig,
}

params_general_builder!(
    FBMWarpBuilder3D,
    [const METHOD: u8, const X: usize, const Y: usize, const Z: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>, ZIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, Z, N, XIter, YIter, ZIter]
);

params_warp_builder!(
    FBMWarpBuilder3D,
    [const METHOD: u8, const X: usize, const Y: usize, const Z: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>, ZIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, Z, N, XIter, YIter, ZIter]
);

params_fbm_builder!(
    FBMWarpBuilder3D,
    [const METHOD: u8, const X: usize, const Y: usize, const Z: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>, ZIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, Z, N, XIter, YIter, ZIter]
);

params_fbm_scaling_3d!(
    FBMWarpBuilder3D,
    [const METHOD: u8, const X: usize, const Y: usize, const Z: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>, ZIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, Z, N, XIter, YIter, ZIter]
);

params_batch_3d!(
    FBMWarpBuilder3D,
    [const METHOD: u8, const X: usize, const Y: usize, const Z: usize, const N: usize,
    XIter: Iterator<Item = ArchSimd<f32>>, YIter: Iterator<Item = ArchSimd<f32>>, ZIter: Iterator<Item = ArchSimd<f32>>],
    [METHOD, X, Y, Z, N, XIter, YIter, ZIter],
    [METHOD, X, Y, Z, N,], self, x_iter, y_iter, z_iter, {
        FBMWarpBuilder3D {
            grid_config: self.grid_config,
            general_config: self.general_config,
            fbm_config: self.fbm_config,
            warp_config: self.warp_config,
            batch_config: BatchBuilder3DConfig {
                x_iter: Some(x_iter),
                y_iter: Some(y_iter),
                z_iter: Some(z_iter),
            },
        }
    }
);

impl<
    const METHOD: u8,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
    XIter,
    YIter,
    ZIter,
> FBMWarpBuilder3D<METHOD, X, Y, Z, N, XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    // declare_fill!(self, output, {
    //     let mut i = 0;
    //     self.into_iter().for_each(|x| {
    //         output.store_simd(i, x);
    //         i += ArchSimd::<f32>::LANES;
    //     });
    // });

    // declare_fill_onto!(self, output, {
    //     let mut i = 0;
    //     self.into_iter().for_each(|x| {
    //         let cur = output.load_simd(i);
    //         output.store_simd(i, cur + x);
    //         i += ArchSimd::<f32>::LANES;
    //     });
    // });

    // declare_build!(self, { self.into_iter().collect() });

    // declare_into_iter!(self, {
    //     let grid = Grid3D::<X, Y, Z, N>::from_config(self.grid_config);
    //     let x_iter = self.batch_config.x_iter.expect("X Iter not present!");
    //     let y_iter = self.batch_config.y_iter.expect("Y Iter not present!");

    //     let strength = ArchSimd::splat(self.warp_config.strength);
    //     let new_x_iter = x_iter
    //         .zip(grid.x_iter())
    //         .map(move |(a, b)| b.mul_add(strength, a));
    //     let new_y_iter = y_iter
    //         .zip(grid.y_iter())
    //         .map(move |(a, b)| b.mul_add(strength, a));

    //     let batch_config = BatchBuilder3DConfig {
    //         x_iter: Some(new_x_iter),
    //         y_iter: Some(new_y_iter),
    //     };

    //     Batch3D::<N>::fbm_noise::<METHOD, _, _>(self.general_config, self.fbm_config, batch_config)
    // });
}
