use crate::api::configs::GridConfig;
use crate::api::grid::custom::CustomGridBuilder;
use crate::api::grid::fbm::FbmGridBuilder;
use crate::api::grid::warp::FbmGridWarpBuilder;
use crate::api::methods::{Dim2, Dim3, Octave};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::{BatchNoise, ZeroIter};

pub trait GridNoise: Default {
    fn grid_2d<const X: usize, const Y: usize, const N: usize, const INITIALIZE: bool>(
        seed: u32,
        result: &mut SimdArray<f32, N>,
        position: Vec2<i32>,
        frequency: Vec2<f32>,
        weight: f32,
        magnification: f32,
    );

    fn grid_3d<
        const X: usize,
        const Y: usize,
        const Z: usize,
        const N: usize,
        const INITIALIZE: bool,
    >(
        seed: u32,
        result: &mut SimdArray<f32, N>,
        position: Vec3<i32>,
        frequency: Vec3<f32>,
        weight: f32,
        magnification: f32,
    );
}

// ————————————————————————————————————————————————————————————————
// ————— 2D Grid ——————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// An interface struct for creating 2D noise.
///
/// # Type Parameters
/// * `X` - Length of the 2D Node in the X dimension
/// * `Y` - Length of the 2D Node in the Y dimension
///
/// # Example
/// ```
/// use quick_noise::Grid3D;
///
/// // Subject to change.
/// let grid = Grid3D::<32, 32, 32, 32768>::new()
///     .position(0, 0, 0)
///     .seed(1);
/// ```
#[derive(Default)]
pub struct Grid2D<const X: usize, const Y: usize, const N: usize> {
    pub(crate) config: GridConfig<Dim2>,
}

params_grid_2d!(Grid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);

impl<const X: usize, const Y: usize, const N: usize> Grid2D<X, Y, N> {
    pub fn new() -> Self {
        assert_eq!(
            N,
            X * Y,
            "Grid2D dimensions do not match SimdArray size! {X} * {Y} should be {}, not {N}!",
            X * Y
        );
        Self {
            config: GridConfig::<Dim2>::default(),
        }
    }

    pub(crate) fn from_config(config: GridConfig<Dim2>) -> Self {
        Self { config }
    }

    pub fn fbm<T: GridNoise>(&self) -> FbmGridBuilder<T, Dim2, X, Y, 0, N> {
        FbmGridBuilder::new(self.config)
    }

    pub fn custom<'a, T: GridNoise>(
        &self,
        octave_list: &'a [Octave<Dim2>],
    ) -> CustomGridBuilder<'a, T, Dim2, X, Y, 0, N> {
        CustomGridBuilder::new(self.config, octave_list)
    }

    pub fn warp<T: BatchNoise>(
        &self,
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> FbmGridWarpBuilder<
        T,
        Dim2,
        X,
        Y,
        0,
        N,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
    > {
        FbmGridWarpBuilder::new(self.config, x_iter, y_iter, ZeroIter::<N>::default())
    }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Grid ——————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// An interface struct for creating 3D noise.
///
/// # Type Parameters
/// * `X` - Length of the 3D grid region in the X dimension
/// * `Y` - Length of the 3D grid region in the Y dimension
/// * `Z` - Length of the 3D grid region in the Z dimension
///
/// # Example
/// ```
/// use quick_noise::Grid2D;
///
/// // Subject to change.
/// let grid = Grid2D::<32, 32, 1024>::new()
///     .position(0, 0)
///     .seed(1);
/// ```
#[derive(Default)]
pub struct Grid3D<const X: usize, const Y: usize, const Z: usize, const N: usize> {
    pub(crate) config: GridConfig<Dim3>,
}

params_grid_3d!(Grid3D, [const X: usize, const Y: usize, const Z: usize, const N: usize], [X, Y, Z, N]);

impl<const X: usize, const Y: usize, const Z: usize, const N: usize> Grid3D<X, Y, Z, N> {
    pub fn new() -> Self {
        assert_eq!(
            N,
            X * Y * Z,
            "Grid3D dimensions do not match SimdArray size! {X} * {Y} * {Z} should be {}, not {N}!",
            X * Y * Z
        );
        Self {
            config: GridConfig::<Dim3>::default(),
        }
    }

    pub(crate) fn from_config(config: GridConfig<Dim3>) -> Self {
        Self { config }
    }

    pub fn fbm<T: GridNoise>(&self) -> FbmGridBuilder<T, Dim3, X, Y, Z, N> {
        FbmGridBuilder::new(self.config)
    }

    pub fn custom<'a, T: GridNoise>(
        &self,
        octave_list: &'a [Octave<Dim3>],
    ) -> CustomGridBuilder<'a, T, Dim3, X, Y, Z, N> {
        CustomGridBuilder::new(self.config, octave_list)
    }

    pub fn warp<T: BatchNoise>(
        &self,
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
        z_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> FbmGridWarpBuilder<
        T,
        Dim3,
        X,
        Y,
        Z,
        N,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
    > {
        FbmGridWarpBuilder::new(self.config, x_iter, y_iter, z_iter)
    }
}
