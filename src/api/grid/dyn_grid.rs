use crate::api::builders::*;
use crate::math::random::Random;
use crate::math::vec::Vec2;
use crate::perlin::Perlin;
use crate::simd::arch_simd::{ArchMask, ArchSimd};
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;
use crate::simd::simd_vec::SimdVec;

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
    config: Grid2DConfig,
}

/// An interface struct for creating 2D noise.
///
/// # Parameters
/// * `x` - x coordinate of the node in a 2D grid
/// * `y` - y coordinate of the node in a 2D grid
///
/// # Example
/// ```
/// use quick_noise::noise::pipeline::Node2D;
///
/// // Subject to change.
/// let node = Node2D::<32, 32>::new()
/// ```
impl<const X: usize, const Y: usize, const N: usize> Grid2D<X, Y, N> {
    pub fn new() -> Self {
        assert_eq!(
            N,
            X * Y,
            "Grid2D dimensions do not match SimdArray size! {X} * {Y} should be {}, not {N}!",
            X * Y
        );
        Self {
            config: Grid2DConfig::default(),
        }
    }

    pub fn seed(mut self, seed: i64) -> Self {
        self.config.grid_seed = Random::static_mix_u64(seed as u64);
        self
    }

    pub fn dimensions(mut self, x: usize, y: usize) -> Self {
        self.config.dim_x = x;
        self.config.dim_y = y;
        self
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.config.x = x;
        self.config.y = y;
        self
    }

    pub fn perlin(&self) -> PerlinGrid2D<X, Y, N> {
        PerlinGrid2D::new(self.config)
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
    config: Grid3DConfig,
}

/// An interface struct for creating 3D noise.
///
/// # Parameters
/// * `x` - x coordinate of the node in a 3D grid
/// * `y` - y coordinate of the node in a 3D grid
/// * `z` - z coordinate of the node in a 3D grid
///
/// # Example
/// ```
/// use quick_noise::Grid3D;
///
/// // Subject to change.
/// let node = Grid3D::<32, 32>::new(0, 0);
/// ```
impl<const X: usize, const Y: usize, const Z: usize, const N: usize> Grid3D<X, Y, Z, N> {
    pub fn new() -> Self {
        assert_eq!(
            N,
            X * Y * Z,
            "Grid3D dimensions do not match SimdArray size! {X} * {Y} * {Z} should be {}, not {N}!",
            X * Y * Z
        );
        Self {
            config: Grid3DConfig::default(),
        }
    }

    pub fn seed(mut self, seed: i64) -> Self {
        self.config.grid_seed = Random::static_mix_u64(seed as u64);
        self
    }

    pub fn position(mut self, x: i32, y: i32, z: i32) -> Self {
        self.config.x = x;
        self.config.y = y;
        self.config.z = z;
        self
    }

    // pub fn perlin(&self) -> PerlinGrid3D<X, Y, Z, N> {
    //     PerlinGrid3D::new(self.seed, self.x, self.y)
    // }
}

// ————————————————————————————————————————————————————————————————
// ————— 2D Perlin Uniform Grid ———————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// A struct for creating perlin noise set on a uniform grid.
/// The most performant way to generate perline noise.
#[derive(Default)]
pub struct PerlinGrid2D<const X: usize, const Y: usize, const N: usize> {
    grid_config: Grid2DConfig,
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig,
    dim_config: Builder2DConfig,
}

impl<const X: usize, const Y: usize, const N: usize> PerlinGrid2D<X, Y, N> {
    fn new(grid_config: Grid2DConfig) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    #[inline(always)]
    pub(crate) fn octave_seed(seed: u64, frequency: Vec2<f32>) -> u32 {
        Random::static_mix_u64_pair(
            seed.wrapping_mul(frequency.x.to_bits() as u64),
            seed.wrapping_mul(frequency.y.to_bits() as u64),
        ) as u32
    }

    #[inline(always)]
    pub(crate) fn fbm_noise(self, result: &mut SimdVec<f32>) {
        if self.fbm_config.octaves == 0 {
            return;
        }

        // Vec setup:
        let size = self.grid_config.dim_x * self.grid_config.dim_y;
        let needed = size.saturating_sub(result.len());
        result.reserve(needed);

        // println!("capacity: {}, size: {size}", result.capacity());
        unsafe { result.set_len(size) }; // Unsafe but skips initialization overhead.

        let seed =
            Random::static_mix_u64_pair(self.general_config.seed, self.grid_config.grid_seed);

        let dimensions = Vec2::new(self.grid_config.dim_x, self.grid_config.dim_y);
        let position = Vec2::new(self.grid_config.x, self.grid_config.y);
        let magnification = self.general_config.magnification;

        // FBM algorithm:
        let mut frequency = Vec2::new(
            self.dim_config.x_scaling * self.general_config.frequency,
            self.dim_config.y_scaling * self.general_config.frequency,
        );
        let mut weight = self.general_config.amplitude;

        // First octave:
        let first_seed = Self::octave_seed(seed, frequency);
        Perlin::grid_2d::<true>(
            dimensions,
            first_seed,
            result,
            position,
            frequency,
            weight,
            magnification,
        );

        // Subsequent octaves:
        for _ in 1..self.fbm_config.octaves {
            weight *= self.fbm_config.persistence;
            frequency *= self.fbm_config.lacunarity;

            let octave_seed = Self::octave_seed(seed, frequency);
            Perlin::grid_2d::<false>(
                dimensions,
                octave_seed,
                result,
                position,
                frequency,
                weight,
                magnification,
            );
        }
    }

    pub fn build(self) -> SimdVec<f32> {
        // TODO: Ensure result array will always be initialized.
        let mut result = SimdVec::new();
        self.fbm_noise(&mut result);
        result
    }

    pub fn into_iter(self) -> impl Iterator<Item = ArchSimd<f32>> {
        self.build().into_iter()
    }

    pub fn fill(self, result: &mut SimdVec<f32>) {
        self.fbm_noise(result);
    }
}

apply_general_builder_params!(PerlinGrid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);
apply_fbm_builder_params!(PerlinGrid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);
apply_builder_2d_params!(PerlinGrid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);

// impl<const N: usize, const X: usize, const Y: usize> BuilderExecute<f32, N>
//     for PerlinGrid2D<N, X, Y>
// {
//     fn build(self) -> SimdArray<f32, N> {
//         // TODO: Ensure result array will always be initialized.
//         let mut result = SimdVec::new();
//         self.fbm_noise(&mut result);
//         result
//     }

//     fn fill(self, result: &mut SimdArray<f32, N>) {
//         assert_eq!(
//             N,
//             X * Y,
//             "Grid2D dimensions do not match SimdArray size! {X} * {Y} should be {}, not {N}!",
//             X * Y
//         );
//         let pos = Vec2::new(self.x as i32, self.y as i32);
//         let mut random_gen = Random::new(self.core_seed);

//         Perlin::uniform_grid_2d(
//             &mut random_gen,
//             result,
//             pos,
//             self.octaves,
//             self.frequency,
//             self.amplitude,
//             self.lacunarity,
//             self.persistence,
//             self.channel_seed as i32,
//             self.magnification,
//         );
//     }

//     fn into_iter(self) -> impl Iterator<Item = ArchSimd<f32>> {
//         self.build().into_iter()
//     }
// }


// TODO: ADD VARIABLE DIMENSIONS FOR UNIFORM GRID!!!
// impl<const N: usize, const X: usize, const Y: usize> BuilderExecute<f32, N>
//     for PerlinGrid2D<N, X, Y>
// {
//     fn build(self) -> SimdArray<f32, N> {
//         assert_eq!(
//             N,
//             X * Y,
//             "Grid2D dimensions do not match SimdArray size! {X} * {Y} should be {}, not {N}!",
//             X * Y
//         );
//         // TODO: Ensure result array will always be initialized.
//         let mut result = unsafe { SimdArray::<f32, N>::new_uninit() };
//         self.fill(&mut result);
//         result
//     }

//     fn fill(self, result: &mut SimdArray<f32, N>) {
//         assert_eq!(
//             N,
//             X * Y,
//             "Grid2D dimensions do not match SimdArray size! {X} * {Y} should be {}, not {N}!",
//             X * Y
//         );
//         let pos = Vec2::new(self.x as i32, self.y as i32);
//         let mut random_gen = Random::new(self.core_seed);

//         Perlin::uniform_grid_2d(
//             &mut random_gen,
//             result,
//             pos,
//             self.octaves,
//             self.frequency,
//             self.amplitude,
//             self.lacunarity,
//             self.persistence,
//             self.channel_seed as i32,
//             self.magnification,
//         );
//     }

//     fn into_iter(self) -> impl Iterator<Item = ArchSimd<f32>> {
//         self.build().into_iter()
//     }
// }

// ————————————————————————————————————————————————————————————————
// ————— Iterators ————————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<const X: usize, const Y: usize, const N: usize> Grid2D<X, Y, N> {
    #[inline(always)]
    pub fn x_iter(&self) -> NodeIter2DX<X, Y> {
        NodeIter2DX {
            rows_left: X,
            columns_left: Y,
            cur_val: (X as i32 * self.config.x) as f32,
        }
    }

    #[inline(always)]
    pub fn y_iter(&self) -> NodeIter2DY<X, Y> {
        let start_vec = ArchSimd::iota((Y as i32 * self.config.y) as f32);
        let cur_vec = start_vec - ArchSimd::splat(ArchSimd::<f32>::LANES as f32);
        NodeIter2DY {
            rows_left: X,
            columns_left: Y,
            start_vec,
            cur_vec,
        }
    }
}
pub struct NodeIter2DX<const X: usize, const Y: usize> {
    rows_left: usize,
    columns_left: usize,
    cur_val: f32,
}

impl<const X: usize, const Y: usize> NodeIter2DX<X, Y> {
    const LANES: usize = ArchSimd::<f32>::LANES;
}

impl<const X: usize, const Y: usize> Iterator for NodeIter2DX<X, Y> {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Full rows.
        if self.rows_left > Self::LANES {
            self.rows_left -= Self::LANES;
            return Some(ArchSimd::splat(self.cur_val));
        }

        // Partial/Transition rows.
        if self.columns_left > 0 {
            let new_val = self.cur_val + 1.0;
            let mask = ArchMask::first_n_true(self.rows_left as u32);
            let old = ArchSimd::splat(self.cur_val);
            let new = ArchSimd::splat(new_val);
            self.cur_val = new_val;
            self.rows_left = (X - Self::LANES) + self.rows_left;
            self.columns_left -= 1;
            return Some(mask.select(old, new));
        }

        // Tail.
        if self.rows_left > 0 {
            let mask = ArchMask::first_n_true(self.rows_left as u32);
            let vec = ArchSimd::splat(self.cur_val);
            self.rows_left = 0;
            return Some(mask.select(vec, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }
}

pub struct NodeIter2DY<const X: usize, const Y: usize> {
    rows_left: usize,
    columns_left: usize,
    cur_vec: ArchSimd<f32>,
    start_vec: ArchSimd<f32>,
}

impl<const X: usize, const Y: usize> NodeIter2DY<X, Y> {
    const LANES: usize = ArchSimd::<f32>::LANES;
}

impl<const X: usize, const Y: usize> Iterator for NodeIter2DY<X, Y> {
    type Item = ArchSimd<f32>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        // Full columns.
        if self.columns_left >= Self::LANES {
            self.columns_left -= Self::LANES;
            self.cur_vec += ArchSimd::splat(Self::LANES as f32);
            return Some(self.cur_vec);
        }

        // Partial/Transition columns.
        if self.rows_left > 0 {
            let mask = ArchMask::first_n_true(self.columns_left as u32);
            let old = self.cur_vec + ArchSimd::splat(Self::LANES as f32);
            let new = self.start_vec - ArchSimd::splat(self.columns_left as f32);
            self.cur_vec = new;
            self.columns_left = (Y - Self::LANES) + self.columns_left;
            self.rows_left -= 1;
            return Some(mask.select(old, new));
        }

        // Tail.
        if self.columns_left > 0 {
            let mask = ArchMask::first_n_true(self.columns_left as u32);
            let vec = self.cur_vec + ArchSimd::splat(Self::LANES as f32);
            self.columns_left = 0;
            return Some(mask.select(vec, ArchSimd::zero()));
        }

        // Finished iter.
        None
    }
}
