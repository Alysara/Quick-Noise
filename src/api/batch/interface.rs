use crate::api::batch::custom::CustomBatchBuilder;
use crate::api::batch::fbm::FbmBatchBuilder;
use crate::api::defaults::ZeroIter;
use crate::api::methods::Octave;
use crate::simd::arch_simd::ArchSimd;
use crate::{Dim2, Dim3};

/// Static struct for calling independent batches of noise.
pub struct Batch2D {}
pub struct Batch3D {}

/// Functions needed to be supported for each type of batched noise.
pub trait BatchNoise {
    fn batch_2d(
        seed: u32,
        x_input: ArchSimd<f32>,
        y_input: ArchSimd<f32>,
        x_freq: ArchSimd<f32>,
        y_freq: ArchSimd<f32>,
    ) -> ArchSimd<f32>;

    fn batch_3d(
        seed: u32,
        x_input: ArchSimd<f32>,
        y_input: ArchSimd<f32>,
        z_input: ArchSimd<f32>,
        x_freq: ArchSimd<f32>,
        y_freq: ArchSimd<f32>,
        z_freq: ArchSimd<f32>,
    ) -> ArchSimd<f32>;
}

impl Batch2D {
    pub fn fbm<T: BatchNoise, const N: usize>(
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> FbmBatchBuilder<
        T,
        Dim2,
        N,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
        ZeroIter<N>,
    > {
        FbmBatchBuilder::new(x_iter, y_iter, ZeroIter::default())
    }

    pub fn custom<'a, T: BatchNoise, const N: usize>(
        octave_list: &'a [Octave<Dim2>],
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> CustomBatchBuilder<
        'a,
        T,
        Dim2,
        N,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
        ZeroIter<N>,
    > {
        CustomBatchBuilder::new(octave_list, x_iter, y_iter, ZeroIter::default())
    }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Batch API —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl Batch3D {
    pub fn fbm<T: BatchNoise, const N: usize>(
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
        z_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> FbmBatchBuilder<
        T,
        Dim3,
        N,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
    > {
        FbmBatchBuilder::new(x_iter, y_iter, z_iter)
    }

    pub fn custom<'a, T: BatchNoise, const N: usize>(
        octave_list: &'a [Octave<Dim3>],
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
        z_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> CustomBatchBuilder<
        'a,
        T,
        Dim3,
        N,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
        impl Iterator<Item = ArchSimd<f32>>,
    > {
        CustomBatchBuilder::new(octave_list, x_iter, y_iter, z_iter)
    }
}
