use crate::api::configs::*;
use crate::api::methods::NoiseDimension;
use crate::math::vec::{BasicVec, Vec2, Vec3};
use crate::perlin::{Octave2D, Octave3D};
use crate::simd::arch_simd::ArchSimd;

const DEFAULT_OCTAVE_2D: Octave2D = Octave2D::splat(0.03125, 1.0);
const DEFAULT_OCTAVE_3D: Octave3D = Octave3D::splat(0.03125, 1.0);

const DEFAULT_OCTAVES_2D: &[Octave2D] = &[DEFAULT_OCTAVE_2D];
const DEFAULT_OCTAVES_3D: &[Octave3D] = &[DEFAULT_OCTAVE_3D];

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            seed: 0xD5E7B3C94F8A1E6B,
            amplitude: 1.0,
            magnification: 1.0,
            normalization: true,
        }
    }
}

impl<D: NoiseDimension> Default for FbmConfig<D> {
    fn default() -> Self {
        Self {
            octaves: 1,
            frequency: 0.03125,
            lacunarity: 2.0,
            persistence: 0.5,
            scaling: D::FVec::splat(1.0),
        }
    }
}

impl<D: NoiseDimension> Default for GridConfig<D> {
    fn default() -> Self {
        Self {
            dimensions: D::USizeVec::splat(32),
            grid_seed: 0xc4ceb9fe1a85ec53,
            position: D::IVec::splat(0),
            tiling: D::Vec::splat(None),
        }
    }
}

impl Default for WarpBuilderConfig {
    fn default() -> Self {
        Self {
            strength: 100.0,
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

/// Zero Iter for blank noise output.
#[derive(Default)]
pub struct ZeroIter<const N: usize> {
    index: usize,
}

impl<const N: usize> Iterator for ZeroIter<N> {
    type Item = ArchSimd<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < N {
            self.index += ArchSimd::<f32>::LANES;
            Some(ArchSimd::zero())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        const LANES: usize = ArchSimd::<f32>::LANES;
        let left = (N - self.index + LANES - 1) / ArchSimd::<f32>::LANES;
        (left, Some(left))
    }
}
