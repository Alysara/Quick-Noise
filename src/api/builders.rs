use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::traits::SimdFloat;

pub enum NoiseMethod {
    Perlin = 0,
    Value = 1,
    Simplex = 2,
    Cellular = 3,
}

/// Necessary converter due to limitation of types for const generics.
impl NoiseMethod {
    pub const PERLIN_U8: u8 = NoiseMethod::Perlin as u8;
    pub const VALUE_U8: u8 = NoiseMethod::Value as u8;
    pub const SIMPLEX_U8: u8 = NoiseMethod::Simplex as u8;
    pub const CELLULAR_U8: u8 = NoiseMethod::Cellular as u8;

    pub const fn from_u8_const(val: u8) -> Self {
        match val {
            0 => NoiseMethod::Perlin,
            1 => NoiseMethod::Value,
            2 => NoiseMethod::Simplex,
            3 => NoiseMethod::Cellular,
            _ => panic!("Invalid NoiseMethod enum value!"),
        }
    }
}

pub(crate) struct GeneralBuilderConfig {
    pub(crate) seed: u64,
    pub(crate) frequency: f32,
    pub(crate) amplitude: f32,
    pub(crate) magnification: f32,
}

pub(crate) struct FBMBuilderConfig {
    pub(crate) octaves: u32,
    pub(crate) lacunarity: f32,
    pub(crate) persistence: f32,
}

pub(crate) struct Builder2DConfig {
    pub(crate) x_scaling: f32,
    pub(crate) y_scaling: f32,
}

#[derive(Default)]
pub(crate) struct Grid2DConfig {
    pub(crate) grid_seed: u64,
    pub(crate) x: i32,
    pub(crate) y: i32,
}

pub(crate) struct BatchBuilder2DConfig<XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    pub(crate) x_iter: Option<XIter>,
    pub(crate) y_iter: Option<YIter>,
}

impl Default for GeneralBuilderConfig {
    fn default() -> Self {
        Self {
            seed: 0xD5E7B3C94F8A1E6B,
            frequency: 0.03125,
            amplitude: 1.0,
            magnification: 1.0,
        }
    }
}

impl Default for FBMBuilderConfig {
    fn default() -> Self {
        Self {
            octaves: 1,
            lacunarity: 0.5,
            persistence: 0.5,
        }
    }
}

impl Default for Builder2DConfig {
    fn default() -> Self {
        Self {
            x_scaling: 1.0,
            y_scaling: 1.0,
        }
    }
}

impl<XIter, YIter> Default for BatchBuilder2DConfig<XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    fn default() -> Self {
        Self {
            x_iter: None,
            y_iter: None,
        }
    }
}

/// Empty iterator for default generics.
#[derive(Default)]
pub struct EmptyIter;

impl Iterator for EmptyIter {
    type Item = ArchSimd<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

/// Common interface for execucting all noise builders.
///
/// All builders support these three execution methods:
///  - `build()`: Create a new array
///  - `into_iter()`: Get a lazy iterator
///  - `fill()`: Reuse existing memory
pub trait BuilderExecute<T: SimdFloat, const N: usize> {
    /// Creates the noise and returns the result in an output array.
    ///
    /// Needs to know the length of the output SimdArray because
    /// const generic expr is not yet available in stable Rust when this was
    /// created.
    fn build(self) -> SimdArray<T, N>;

    /// Returns an iterator containing chunks of the noise output.
    /// Ideal for managing streams of noise without unnecessary read/writes.
    fn into_iter(self) -> impl Iterator<Item = ArchSimd<T>>;

    /// Creates the noise and puts the result in a given array.
    fn fill(self, output: &mut SimdArray<T, N>);
}

pub trait GeneralBuilder {
    /// Determines the psuedo-random values used in noise generation.
    /// Different seeds produce different noise.
    fn seed(self, seed: i64) -> Self;

    /// Controls how 'compressed' the noise is. Lower frequencies are smoother
    /// and change slower from pixel to pixel, while higher frequencies are sharper and
    /// change more quickly from pixel to pixel.
    ///
    /// # Default
    /// `0.03125` (1.0 / 32.0)
    ///
    /// # Note
    /// Frequencies higher than 0.5 are not properly supported by the uniform grid
    /// algorithm. For accurate noise at super-high frequencies, use perlin_batch().
    fn frequency(self, frequency: f32) -> Self;

    /// Controls the range of the noise output. All output is normalized
    /// to be in the range of `[-amplitude, amplitude]`.
    ///
    /// # Default
    /// `1.0`
    ///
    /// # Note
    /// As the number of octaves increases, the average noise value trends
    /// closer to zero due to more noise layers averaging eachother out.
    fn amplitude(self, amplitude: f32) -> Self;

    /// Controls the magnification of the noise output. For most use cases,
    /// this value can be ignored. Useful for LODs or multi-quality noise
    /// generation.
    ///
    /// # Default
    /// `1.0`
    fn magnification(self, magnification: f32) -> Self;
}

pub trait FBMBuilder {
    /// Determines the number of perlin noise passes layered ontop of one another.
    /// More octaves generally leads to more natural-appearing noise.
    ///
    /// # Default
    /// `1`
    fn octaves(self, octaves: u32) -> Self;

    /// Controls how the frequency changes after each subsequenct octave
    /// (noise layer). The next octave's frequency is the previous octave's
    /// frequency multiplied by the lacunarity.
    ///
    /// # Default
    /// `0.5`
    fn lacunarity(self, lacunarity: f32) -> Self;

    /// Controls how much each subsequenct octave (noise layer) impacts
    /// the final noise result. The next octave's weight is the previous octave's
    /// frequency multiplied by the persistence.
    ///
    /// # Default
    /// `0.5`
    fn persistence(self, lacunarity: f32) -> Self;
}

pub trait Builder2D {
    /// Controls how much each axis of the grid is 'stretched' in the noise
    /// sample space. Creates visible stretching in the noise output.
    /// The default values have no stretching.
    ///
    /// # Default
    ///  - `x_scaling`: 1.0
    ///  - `y_scaling`: 1.0
    fn scaling(self, x_scaling: f32, y_scaling: f32) -> Self;
}

pub trait BatchBuilder2D {
    type OutputBuilder<XIter, YIter>: BatchBuilder2D
    where
        XIter: Iterator<Item = ArchSimd<f32>>,
        YIter: Iterator<Item = ArchSimd<f32>>;
    fn input_iters<NewXIter, NewYIter>(
        self,
        x_iter: NewXIter,
        y_iter: NewYIter,
    ) -> Self::OutputBuilder<NewXIter, NewYIter>
    where
        NewXIter: Iterator<Item = ArchSimd<f32>>,
        NewYIter: Iterator<Item = ArchSimd<f32>>;
}

// ————————————————————————————————————————————————————————————————
// ————— Builder Macros ———————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

macro_rules! apply_general_builder_params {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > GeneralBuilder for $name< $($short_generics)* > {
            fn seed(mut self, seed: i64) -> Self {
                self.general_config.seed = Random::static_mix_u64(seed as u64 ^ 0xD5E7B3C94F8A1E6B);
                self
            }

            fn frequency(mut self, frequency: f32) -> Self {
                self.general_config.frequency = frequency;
                self
            }

            fn amplitude(mut self, amplitude: f32) -> Self {
                self.general_config.amplitude = amplitude;
                self
            }

            fn magnification(mut self, magnification: f32) -> Self {
                self.general_config.magnification = magnification;
                self
            }
        }
    };
}
pub(crate) use apply_general_builder_params;

macro_rules! apply_fbm_builder_params {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > FBMBuilder for $name< $($short_generics)* > {
            fn octaves(mut self, octaves: u32) -> Self {
                self.fbm_config.octaves = octaves;
                self
            }

            fn lacunarity(mut self, lacunarity: f32) -> Self {
                self.fbm_config.lacunarity = lacunarity;
                self
            }

            fn persistence(mut self, persistence: f32) -> Self {
                self.fbm_config.persistence = persistence;
                self
            }
        }
    };
}
pub(crate) use apply_fbm_builder_params;

macro_rules! apply_builder_2d_params {
    ($name:ident, [ $($full_generics:tt)* ], [ $($short_generics:tt)* ]) => {
        impl< $($full_generics)* > Builder2D for $name< $($short_generics)* > {
            fn scaling(mut self, x_scaling: f32, y_scaling: f32) -> Self {
                self.dim_config.x_scaling = x_scaling;
                self.dim_config.y_scaling = y_scaling;
                self
            }
        }
    };
}
pub(crate) use apply_builder_2d_params;
