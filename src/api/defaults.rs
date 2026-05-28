use crate::api::configs::*;
use crate::math::vec::{Vec2, Vec3};
use crate::perlin::{Octave2D, Octave3D};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_traits::SimdZero;

const DEFAULT_OCTAVE_2D: Octave2D = Octave2D::splat(0.03125, 1.0);
const DEFAULT_OCTAVE_3D: Octave3D = Octave3D::splat(0.03125, 1.0);

const DEFAULT_OCTAVES_2D: &[Octave2D] = &[DEFAULT_OCTAVE_2D];
const DEFAULT_OCTAVES_3D: &[Octave3D] = &[DEFAULT_OCTAVE_3D];

impl Default for GeneralBuilderConfig {
    fn default() -> Self {
        Self {
            seed: 0xD5E7B3C94F8A1E6B,
            amplitude: 1.0,
            magnification: 1.0,
            normalization: true,
        }
    }
}

impl Default for FBMBuilderConfig2D {
    fn default() -> Self {
        Self {
            octaves: 1,
            frequency: 0.03125,
            lacunarity: 2.0,
            persistence: 0.5,
            scaling: Vec2::splat(1.0),
        }
    }
}

impl Default for FBMBuilderConfig3D {
    fn default() -> Self {
        Self {
            octaves: 1,
            frequency: 0.03125,
            lacunarity: 2.0,
            persistence: 0.5,
            scaling: Vec3::splat(1.0),
        }
    }
}

impl Default for GridConfig2D {
    fn default() -> Self {
        Self {
            grid_seed: 0xc4ceb9fe1a85ec53,
            position: Vec2::splat(0),
        }
    }
}

impl Default for GridConfig3D {
    fn default() -> Self {
        Self {
            grid_seed: 0xc4ceb9fe1a85ec53,
            position: Vec3::splat(0),
        }
    }
}

impl Default for CustomBuilderConfig<'static, Octave2D> {
    fn default() -> Self {
        Self {
            octave_list: DEFAULT_OCTAVES_2D,
        }
    }
}

impl Default for CustomBuilderConfig<'static, Octave3D> {
    fn default() -> Self {
        Self {
            octave_list: DEFAULT_OCTAVES_3D,
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

impl<XIter, YIter, ZIter> Default for BatchBuilder3DConfig<XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    fn default() -> Self {
        Self {
            x_iter: None,
            y_iter: None,
            z_iter: None,
        }
    }
}

impl Default for WarpBuilderConfig {
    fn default() -> Self {
        Self { strength: 1.0 }
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
}
