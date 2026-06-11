use crate::api::methods::{NoiseDimension, Octave};
use crate::math::vec::{Vec2, Vec3, VecHorzMax};
use crate::simd::arch_simd::ArchSimd;

#[derive(Copy, Clone)]
pub(crate) struct GeneralBuilderConfig {
    pub(crate) seed: u64,
    pub(crate) amplitude: f32,
    pub(crate) magnification: f32,
    pub(crate) normalization: bool,
}

#[derive(Copy, Clone)]
pub(crate) struct FbmBuilderConfig<D: NoiseDimension> {
    pub(crate) octaves: usize,
    pub(crate) frequency: f32,
    pub(crate) lacunarity: f32,
    pub(crate) persistence: f32,
    pub(crate) scaling: D::FVec,
}

impl<D: NoiseDimension> FbmBuilderConfig<D> {
    pub(crate) fn num_grid_octaves(&self) -> usize {
        let max_scaling = self.scaling.horizontal_max();

        let mut cur_freq = self.frequency * max_scaling;
        if cur_freq >= 1.0 || self.lacunarity >= 1.0 {
            for i in 0..self.octaves {
                if cur_freq >= 1.0 {
                    return i;
                }
                cur_freq *= self.lacunarity;
            }
        }

        self.octaves
    }

    // pub(crate) fn num_batch_octaves(&self) -> usize {
    //     let max_octaves = fbm_config.octaves.min(MAX_FBM_OCTAVES);
    //     let max_scaling = fbm_config.scaling.horizontal_max();
    //     let mut cur_freq = fbm_config.frequency * max_scaling;
    //     if fbm_config.lacunarity >= 1.0 {
    //         for i in 0..max_octaves {
    //             if cur_freq >= 1.0 {
    //                 break 'outer i;
    //             }
    //             cur_freq *= fbm_config.lacunarity;
    //         }
    //     }
    //     max_octaves
    // }
    //
    pub(crate) fn normalize_amplitude(&self, amplitude: f32) -> f32 {
        let mut sum = 0.0;
        let mut cur = 1.0;
        for _ in 0..self.octaves {
            sum += cur;
            cur *= self.persistence;
        }
        amplitude / sum
    }
}

pub(crate) struct CustomBuilderConfig<'a, D: NoiseDimension> {
    pub(crate) octave_list: &'a [Octave<D>],
}

impl<'a, D: NoiseDimension> CustomBuilderConfig<'a, D> {
    pub(crate) fn normalize_grid_amplitude(&self, amplitude: f32) -> f32 {
        let mut sum = 0.0;
        for octave in self.octave_list {
            if octave.frequency.horizontal_max() < 1.0 {
                sum += octave.weight;
            }
        }

        if sum == 0.0 {
            return 0.0;
        }

        amplitude / sum
    }

    pub(crate) fn normalize_batch_amplitude(&self, amplitude: f32) -> f32 {
        let mut sum = 0.0;
        for octave in self.octave_list {
            sum += octave.weight;
        }

        if sum == 0.0 {
            return 0.0;
        }

        amplitude / sum
    }
}

// #[derive(Copy, Clone)]
pub(crate) struct GridConfig<D: NoiseDimension> {
    pub(crate) grid_seed: u64,
    pub(crate) position: D::IVec,
}
impl<D: NoiseDimension> Copy for GridConfig<D> {}
impl<D: NoiseDimension> Clone for GridConfig<D> {
    fn clone(&self) -> Self { *self }
}

pub(crate) struct BatchBuilderConfig<XIter, YIter, ZIter>
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
