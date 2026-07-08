use crate::api::configs::GridConfig;
// use crate::api::grid::custom::CustomGridBuilder;
use crate::api::grid::fbm::FbmGridBuilder;
// use crate::api::grid::warp::FbmGridWarpBuilder;
use crate::api::methods::Octave;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;
// use crate::{BatchNoise, ZeroIter};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GridNoiseParams<const D: usize> {
    pub seed: u32,
    pub grid_size: [usize; D],
    pub position: [i32; D],
    pub frequency: [f32; D],
    pub weight: f32,
    pub magnification: f32,
    pub tiling: [Option<u32>; D],
}

pub trait GridNoiseImpl<const D: usize>: Default + Copy + Clone + PartialEq {
    fn sample<const INIT: bool>(params: GridNoiseParams<D>, dst: &mut [f32]);
}

// ————————————————————————————————————————————————————————————————
// ————— 2D Grid ——————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// An interface struct for creating 2D noise.
///
/// # Type Parameters
/// * `D: NoiseDimension` - Determines how many dimensions the grid has.
///
/// # Example
/// ```
/// use quick_noise::GridNoise;
///
/// // Subject to change.
/// let grid = GridNoise::<Dim3>::new(32, 32, 32)
///     .position(0, 0, 0)
///     .seed(1);
/// ```

#[derive(Default)]
pub struct GridNoise<const D: usize> {
    pub(crate) config: GridConfig<D>,
}

impl<const D: usize> GridNoise<D> {
    /// Determines the psuedo-random values used in noise generation called
    /// on this grid. Different seeds produce different noise.
    pub fn seed(mut self, seed: i64) -> Self {
        self.config.grid_seed = Random::static_mix_u64(seed as u64);
        self
    }
}

impl GridNoise<2> {
    /// Creates an anchor for a grid region that can be used for call noise.
    ///
    /// # Parameters
    /// -`x`: Length of the grid region along the x-axis
    /// -`y`: Length of the grid region along the y-axis
    pub fn new(x: usize, y: usize) -> Self {
        let mut config = GridConfig::default();
        config.dimensions = [x, y];
        Self { config }
    }

    /// Determines the position values provided to noise calls. This value represents
    /// the position of this grid region in grid units determiend by its dimension.
    /// A 32x32 grid at position { 1, 2 } covers samples in the range { [32-64), [64-96) }.
    ///
    /// # Default:
    /// `0`: x
    /// `0`: y
    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.config.position = [x, y];
        self
    }

    /// Determines the distance the sample space has until it starts repeating noise
    /// seamlessly. When values are left as None, noise does not repeat.
    ///
    /// # Default:
    /// - `x`: None
    /// - `y`: None
    pub fn tiling(mut self, x: Option<u32>, y: Option<u32>) -> Self {
        self.config.tiling = [x, y];
        self
    }
}

impl GridNoise<3> {
    /// Creates an anchor for a grid region that can be used for call noise.
    ///
    /// # Parameters
    /// -`x`: Length of the grid region along the x-axis
    /// -`y`: Length of the grid region along the y-axis
    /// -`z`: Length of the grid region along the z-axis
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        let mut config = GridConfig::default();
        config.dimensions = [x, y, z];
        Self { config }
    }

    /// Determines the position values provided to noise calls. This value represents
    /// the position of this grid region in grid units determiend by its dimension.
    /// A 32x32x32 grid at position { 1, 2, 3 } covers samples in the range
    /// { [32-64), [64-96), [96-128) }.
    ///
    /// # Default:
    /// `0`: x
    /// `0`: y
    /// `0`: z
    pub fn position(mut self, x: i32, y: i32, z: i32) -> Self {
        self.config.position = [x, y, z];
        self
    }

    /// Determines the distance the sample space has until it starts repeating noise
    /// seamlessly. When values are left as None, noise does not repeat.
    ///
    /// # Default:
    /// - `x`: None
    /// - `y`: None
    /// - `z`: None
    pub fn tiling(mut self, x: Option<u32>, y: Option<u32>, z: Option<u32>) -> Self {
        self.config.tiling = [x, y, z];
        self
    }
}

impl<const D: usize> GridNoise<D> {
    // pub fn new -> Self {
    //     assert_eq!(
    //         N,
    //         X * Y,
    //         "Grid2D dimensions do not match SimdArray size! {X} * {Y} should be {}, not {N}!",
    //         X * Y
    //     );
    //     Self {
    //         config: GridConfig::<Dim2>::default(),
    //     }
    // }

    pub(crate) fn from_config(config: GridConfig<D>) -> Self {
        Self { config }
    }

    pub fn fbm<T: GridNoiseImpl<D>>(&self) -> FbmGridBuilder<D, T> {
        FbmGridBuilder::from_config(self.config)
    }

    // pub fn custom<'a, T: GridNoiseImpl>(
    //     &self,
    //     octave_list: &'a [Octave<Dim2>],
    // ) -> CustomGridBuilder<'a, T, Dim2, X, Y, 0, N> {
    //     CustomGridBuilder::new(self.config, octave_list)
    // }

    // pub fn warp<T: BatchNoise>(
    //     &self,
    //     x_iter: impl Iterator<Item = ArchSimd<f32>>,
    //     y_iter: impl Iterator<Item = ArchSimd<f32>>,
    // ) -> FbmGridWarpBuilder<
    //     T,
    //     Dim2,
    //     X,
    //     Y,
    //     0,
    //     N,
    //     impl Iterator<Item = ArchSimd<f32>>,
    //     impl Iterator<Item = ArchSimd<f32>>,
    //     impl Iterator<Item = ArchSimd<f32>>,
    // > {
    //     FbmGridWarpBuilder::new(self.config, x_iter, y_iter, ZeroIter::<N>::default())
    // }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Grid ——————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

// /// An interface struct for creating 3D noise.
// ///
// /// # Type Parameters
// /// * `X` - Length of the 3D grid region in the X dimension
// /// * `Y` - Length of the 3D grid region in the Y dimension
// /// * `Z` - Length of the 3D grid region in the Z dimension
// ///
// /// # Example
// /// ```
// /// use quick_noise::Grid2D;
// ///
// /// // Subject to change.
// /// let grid = Grid2D::<32, 32, 1024>::new()
// ///     .position(0, 0)
// ///     .seed(1);
// /// ```
// #[derive(Default)]
// pub struct Grid3D<const X: usize, const Y: usize, const Z: usize, const N: usize> {
//     pub(crate) config: GridConfig<Dim3>,
// }

// params_grid_3d!(Grid3D, [const X: usize, const Y: usize, const Z: usize, const N: usize], [X, Y, Z, N]);

// impl<const X: usize, const Y: usize, const Z: usize, const N: usize> Grid3D<X, Y, Z, N> {
//     pub fn new() -> Self {
//         assert_eq!(
//             N,
//             X * Y * Z,
//             "Grid3D dimensions do not match SimdArray size! {X} * {Y} * {Z} should be {}, not {N}!",
//             X * Y * Z
//         );
//         Self {
//             config: GridConfig::<Dim3>::default(),
//         }
//     }

//     pub(crate) fn from_config(config: GridConfig<Dim3>) -> Self {
//         Self { config }
//     }

//     pub fn fbm<T: GridNoiseImpl>(&self) -> FbmGridBuilder<T, Dim3, X, Y, Z, N> {
//         FbmGridBuilder::new(self.config)
//     }

//     pub fn custom<'a, T: GridNoiseImpl>(
//         &self,
//         octave_list: &'a [Octave<Dim3>],
//     ) -> CustomGridBuilder<'a, T, Dim3, X, Y, Z, N> {
//         CustomGridBuilder::new(self.config, octave_list)
//     }

//     pub fn warp<T: BatchNoise>(
//         &self,
//         x_iter: impl Iterator<Item = ArchSimd<f32>>,
//         y_iter: impl Iterator<Item = ArchSimd<f32>>,
//         z_iter: impl Iterator<Item = ArchSimd<f32>>,
//     ) -> FbmGridWarpBuilder<
//         T,
//         Dim3,
//         X,
//         Y,
//         Z,
//         N,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//     > {
//         FbmGridWarpBuilder::new(self.config, x_iter, y_iter, z_iter)
//     }
// }
