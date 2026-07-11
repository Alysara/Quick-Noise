// use crate::api::batch::custom::CustomBatchBuilder;
use crate::api::batch::fbm::BatchBuilder;
use crate::api::defaults::ZeroIter;
use crate::api::methods::Octave;
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

pub trait BatchNoiseImpl<const D: usize> {
    fn sample_batch(seed: u32, input: [ArchSimd<f32>; D], freq: [ArchSimd<f32>; D])
    -> ArchSimd<f32>;
}

#[derive(Default)]
pub struct BatchNoise<const D: usize> {}

// Static struct for calling independent batches of noise.
// pub struct Batch2D {}
// pub struct Batch3D {}

// impl<const D: usize> Batch2D<D> {
//     pub fn fbm<T: BatchNoiseImpl>(
//         x_iter: impl Iterator<Item = ArchSimd<f32>>,
//         y_iter: impl Iterator<Item = ArchSimd<f32>>,
//     ) -> FbmBatchBuilder<
//         T,
//         Dim2,
//         N,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         ZeroIter<N>,
//     > {
//         FbmBatchBuilder::new(x_iter, y_iter, ZeroIter::default())
//     }

// pub fn custom<'a, T: BatchNoise, const N: usize>(
//     octave_list: &'a [Octave<Dim2>],
//     x_iter: impl Iterator<Item = ArchSimd<f32>>,
//     y_iter: impl Iterator<Item = ArchSimd<f32>>,
// ) -> CustomBatchBuilder<
//     'a,
//     T,
//     Dim2,
//     N,
//     impl Iterator<Item = ArchSimd<f32>>,
//     impl Iterator<Item = ArchSimd<f32>>,
//     ZeroIter<N>,
// > {
//     CustomBatchBuilder::new(octave_list, x_iter, y_iter, ZeroIter::default())
// }
// }

// ————————————————————————————————————————————————————————————————
// ————— 3D Batch API —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

// impl Batch3D {
//     pub fn fbm<T: BatchNoise, const N: usize>(
//         x_iter: impl Iterator<Item = ArchSimd<f32>>,
//         y_iter: impl Iterator<Item = ArchSimd<f32>>,
//         z_iter: impl Iterator<Item = ArchSimd<f32>>,
//     ) -> FbmBatchBuilder<
//         T,
//         Dim3,
//         N,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//     > {
//         FbmBatchBuilder::new(x_iter, y_iter, z_iter)
//     }
//
//     pub fn custom<'a, T: BatchNoise, const N: usize>(
//         octave_list: &'a [Octave<Dim3>],
//         x_iter: impl Iterator<Item = ArchSimd<f32>>,
//         y_iter: impl Iterator<Item = ArchSimd<f32>>,
//         z_iter: impl Iterator<Item = ArchSimd<f32>>,
//     ) -> CustomBatchBuilder<
//         'a,
//         T,
//         Dim3,
//         N,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//         impl Iterator<Item = ArchSimd<f32>>,
//     > {
//         CustomBatchBuilder::new(octave_list, x_iter, y_iter, z_iter)
//     }
// }
