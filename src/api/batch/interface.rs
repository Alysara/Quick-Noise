use std::marker::PhantomData;

use crate::noise::combiners::Combiner;
use crate::simd::arch_simd::ArchSimd;

/// Zipped iterator type used for generically handling
/// `D` iterators.
pub trait DimTuple<const D: usize> {
    /// Converts the internal iterator representation
    /// into a generically sized array.
    fn into_array(self) -> [ArchSimd<f32>; D];
}
type S = ArchSimd<f32>;

impl DimTuple<1> for S {
    fn into_array(self) -> [ArchSimd<f32>; 1] {
        [self]
    }
}

impl DimTuple<2> for (S, S) {
    fn into_array(self) -> [ArchSimd<f32>; 2] {
        [self.0, self.1]
    }
}

impl DimTuple<3> for (S, S, S) {
    fn into_array(self) -> [ArchSimd<f32>; 3] {
        [self.0, self.1, self.2]
    }
}

impl DimTuple<4> for (S, S, S, S) {
    fn into_array(self) -> [ArchSimd<f32>; 4] {
        [self.0, self.1, self.2, self.3]
    }
}

impl DimTuple<5> for (S, S, S, S, S) {
    fn into_array(self) -> [ArchSimd<f32>; 5] {
        [self.0, self.1, self.2, self.3, self.4]
    }
}

pub trait DimIter<const D: usize>: Iterator<Item: DimTuple<D>> {}
impl<const D: usize, T: DimTuple<D>, I: Iterator<Item = T>> DimIter<D> for I {}

pub trait BatchGenerator<const D: usize> {
    /// Generates noise using simd registers.
    ///
    /// # Parameters
    /// - `seed`: Configures the deterministic randomness of the noise
    /// - `input`: Array of input values for each dimension
    /// - `freq`: Array of frequency values for each dimension
    fn sample_batch(
        seed: u32,
        input: [ArchSimd<f32>; D],
        freq: [ArchSimd<f32>; D],
    ) -> ArchSimd<f32>;
}

/// Static struct for sampling batch noise.
///
/// # Example
/// ```
/// use quick_noise::{Grid, BatchNoise, Fbm, Perlin};
///
/// let grid = Grid::<2>::new(32, 32);
///
/// let noise = BatchNoise::<2, Fbm, Perlin>::builder(grid.x_iter(), grid.y_iter())
///     .octaves(1)
///     .frequency(1.0 / 32.0)
///     .build();
/// ```
#[derive(Default)]
pub struct BatchNoise<const D: usize, F: Combiner, S: BatchGenerator<D>> {
    _fractal: PhantomData<F>,
    _sampler: PhantomData<S>,
}
