use crate::api::batch::custom::{CustomBatchBuilder2D, CustomBatchBuilder3D};
use crate::api::batch::fbm::{FBMBatchBuilder2D, FBMBatchBuilder3D};
use crate::api::defaults::EmptyIter;
use crate::api::methods::NoiseMethod;
use crate::math::random::Random;
use crate::math::vec::{Vec2, VecHorzMax};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_traits::*;

/// Static struct for calling independent batches of noise.
pub struct Batch2D<const N: usize> {}
pub struct Batch3D<const N: usize> {}

/// Functions needed to be supported for each type of batched noise.
pub(crate) trait BatchNoise {
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

impl<const N: usize> Batch2D<N> {
    /// Perlin FBM noise
    pub fn perlin() -> FBMBatchBuilder2D<{ NoiseMethod::PERLIN_U8 }, N, EmptyIter, EmptyIter> {
        FBMBatchBuilder2D::default()
    }

    /// Value FBM noise
    pub fn value() -> FBMBatchBuilder2D<{ NoiseMethod::VALUE_U8 }, N, EmptyIter, EmptyIter> {
        FBMBatchBuilder2D::default()
    }

    /// Simplex FBM noise
    pub fn simplex() -> FBMBatchBuilder2D<{ NoiseMethod::SIMPLEX_U8 }, N, EmptyIter, EmptyIter> {
        FBMBatchBuilder2D::default()
    }

    /// Cellular FBM noise
    pub fn cellular() -> FBMBatchBuilder2D<{ NoiseMethod::CELLULAR_U8 }, N, EmptyIter, EmptyIter> {
        FBMBatchBuilder2D::default()
    }

    /// Custom octave Perlin noise
    pub fn perlin_custom()
    -> CustomBatchBuilder2D<'static, { NoiseMethod::PERLIN_U8 }, N, EmptyIter, EmptyIter> {
        CustomBatchBuilder2D::default()
    }

    /// Custom octave Value noise
    pub fn value_custom()
    -> CustomBatchBuilder2D<'static, { NoiseMethod::VALUE_U8 }, N, EmptyIter, EmptyIter> {
        CustomBatchBuilder2D::default()
    }

    /// Custom octave Simplex noise
    pub fn simplex_custom()
    -> CustomBatchBuilder2D<'static, { NoiseMethod::SIMPLEX_U8 }, N, EmptyIter, EmptyIter> {
        CustomBatchBuilder2D::default()
    }

    /// Custom octave Cellular noise
    pub fn cellular_custom()
    -> CustomBatchBuilder2D<'static, { NoiseMethod::CELLULAR_U8 }, N, EmptyIter, EmptyIter> {
        CustomBatchBuilder2D::default()
    }
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Batch API —————————————————————————————————————————————
// ————————————————————————————————————————————————————————————————

impl<const N: usize> Batch3D<N> {
    /// Perlin FBM noise
    pub fn perlin()
    -> FBMBatchBuilder3D<{ NoiseMethod::PERLIN_U8 }, N, EmptyIter, EmptyIter, EmptyIter> {
        FBMBatchBuilder3D::default()
    }

    /// Value FBM noise
    pub fn value()
    -> FBMBatchBuilder3D<{ NoiseMethod::VALUE_U8 }, N, EmptyIter, EmptyIter, EmptyIter> {
        FBMBatchBuilder3D::default()
    }

    /// Simplex FBM noise
    pub fn simplex()
    -> FBMBatchBuilder3D<{ NoiseMethod::SIMPLEX_U8 }, N, EmptyIter, EmptyIter, EmptyIter> {
        FBMBatchBuilder3D::default()
    }

    /// Cellular FBM noise
    pub fn cellular()
    -> FBMBatchBuilder3D<{ NoiseMethod::CELLULAR_U8 }, N, EmptyIter, EmptyIter, EmptyIter> {
        FBMBatchBuilder3D::default()
    }

    /// Custom octave Perlin noise
    pub fn perlin_custom()
    -> CustomBatchBuilder3D<'static, { NoiseMethod::PERLIN_U8 }, N, EmptyIter, EmptyIter, EmptyIter>
    {
        CustomBatchBuilder3D::default()
    }

    /// Custom octave Value noise
    pub fn value_custom()
    -> CustomBatchBuilder3D<'static, { NoiseMethod::VALUE_U8 }, N, EmptyIter, EmptyIter, EmptyIter>
    {
        CustomBatchBuilder3D::default()
    }

    /// Custom octave Simplex noise
    pub fn simplex_custom() -> CustomBatchBuilder3D<
        'static,
        { NoiseMethod::SIMPLEX_U8 },
        N,
        EmptyIter,
        EmptyIter,
        EmptyIter,
    > {
        CustomBatchBuilder3D::default()
    }

    /// Custom octave Cellular noise
    pub fn cellular_custom() -> CustomBatchBuilder3D<
        'static,
        { NoiseMethod::CELLULAR_U8 },
        N,
        EmptyIter,
        EmptyIter,
        EmptyIter,
    > {
        CustomBatchBuilder3D::default()
    }
}
