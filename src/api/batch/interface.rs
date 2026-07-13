use std::marker::PhantomData;

use crate::noise::combiners::Combiner;
use crate::simd::arch_simd::ArchSimd;

pub trait DimTuple<const D: usize> {
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
    fn sample_batch(seed: u32, input: [ArchSimd<f32>; D], freq: [ArchSimd<f32>; D])
    -> ArchSimd<f32>;
}

#[derive(Default)]
pub struct BatchNoise<const D: usize, F: Combiner, S: BatchGenerator<D>> {
    _fractal: PhantomData<F>,
    _sampler: PhantomData<S>,
}


