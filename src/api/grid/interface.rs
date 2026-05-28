use crate::EmptyIter;
use crate::api::configs::{BatchBuilder2DConfig, GridConfig2D, GridConfig3D, WarpBuilderConfig};
use crate::api::grid::custom::{CustomPerlinGrid2D, CustomPerlinGrid3D};
use crate::api::grid::fbm::{PerlinGrid2D, PerlinGrid3D};
use crate::api::grid::warp::FBMWarpBuilder2D;
use crate::api::methods::NoiseMethod;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3};

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
/// use quick_noise::noise::pipeline::Node2D;
///
/// // Subject to change.
/// let node = Node2D::<32, 32>::new(0, 0);
/// ```
pub struct Grid2D<const X: usize, const Y: usize, const N: usize> {
    pub(crate) config: GridConfig2D,
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
            config: GridConfig2D::default(),
        }
    }

    pub(crate) fn from_config(config: GridConfig2D) -> Self {
        Self { config }
    }

    pub fn perlin(&self) -> PerlinGrid2D<X, Y, N> {
        PerlinGrid2D::new(self.config)
    }

    pub fn custom_perlin(&self) -> CustomPerlinGrid2D<'static, X, Y, N> {
        let mut builder = CustomPerlinGrid2D::default();
        builder.grid_config = self.config;
        builder
    }

    pub fn perlin_warp(&self) -> FBMWarpBuilder2D<{ NoiseMethod::PERLIN_U8 }, X, Y, N, EmptyIter, EmptyIter> {
        FBMWarpBuilder2D::default()
    }

    pub fn value_warp(&self) -> FBMWarpBuilder2D<{ NoiseMethod::VALUE_U8 }, X, Y, N, EmptyIter, EmptyIter> {
        FBMWarpBuilder2D::default()
    }

    pub fn simplex_warp(&self) -> FBMWarpBuilder2D<{ NoiseMethod::SIMPLEX_U8 }, X, Y, N, EmptyIter, EmptyIter> {
        FBMWarpBuilder2D::default()
    }

    pub fn cellular_warp(&self) -> FBMWarpBuilder2D<{ NoiseMethod::CELLULAR_U8 }, X, Y, N, EmptyIter, EmptyIter> {
        FBMWarpBuilder2D::default()
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
/// use quick_noise::Grid3D;
///
/// // Subject to change.
/// let node = Grid3D::<32, 32>::new(0, 0);
/// ```
pub struct Grid3D<const X: usize, const Y: usize, const Z: usize, const N: usize> {
    pub(crate) config: GridConfig3D,
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
            config: GridConfig3D::default(),
        }
    }

    pub(crate) fn from_config(config: GridConfig3D) -> Self {
        Self { config }
    }

    pub fn perlin(&self) -> PerlinGrid3D<X, Y, Z, N> {
        PerlinGrid3D::new(self.config)
    }

    pub fn custom_perlin(&self) -> CustomPerlinGrid3D<'static, X, Y, Z, N> {
        let mut builder = CustomPerlinGrid3D::default();
        builder.grid_config = self.config;
        builder
    }
}
