use crate::math::vec::{Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;

pub(crate) struct GeneralBuilderConfig {
    pub(crate) seed: u64,
    pub(crate) amplitude: f32,
    pub(crate) magnification: f32,
    pub(crate) normalization: bool,
}

pub(crate) struct FBMBuilderConfig2D {
    pub(crate) octaves: usize,
    pub(crate) frequency: f32,
    pub(crate) lacunarity: f32,
    pub(crate) persistence: f32,
    pub(crate) scaling: Vec2<f32>,
}

pub(crate) struct FBMBuilderConfig3D {
    pub(crate) octaves: usize,
    pub(crate) frequency: f32,
    pub(crate) lacunarity: f32,
    pub(crate) persistence: f32,
    pub(crate) scaling: Vec3<f32>,
}

pub(crate) struct CustomBuilderConfig<'a, Octave> {
    pub(crate) octave_list: &'a [Octave],
}

pub type GridConfig2D = GridConfig<Vec2<i32>>;
pub type GridConfig3D = GridConfig<Vec3<i32>>;
#[derive(Copy, Clone)]
pub(crate) struct GridConfig<T> {
    pub(crate) grid_seed: u64,
    pub(crate) position: T,
}

pub(crate) struct BatchBuilder2DConfig<XIter, YIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
{
    pub(crate) x_iter: Option<XIter>,
    pub(crate) y_iter: Option<YIter>,
}

pub(crate) struct BatchBuilder3DConfig<XIter, YIter, ZIter>
where
    XIter: Iterator<Item = ArchSimd<f32>>,
    YIter: Iterator<Item = ArchSimd<f32>>,
    ZIter: Iterator<Item = ArchSimd<f32>>,
{
    pub(crate) x_iter: Option<XIter>,
    pub(crate) y_iter: Option<YIter>,
    pub(crate) z_iter: Option<ZIter>,
}

pub(crate) struct WarpBuilderConfig {
    pub(crate) strength: f32,
}