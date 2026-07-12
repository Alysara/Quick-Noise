use crate::api::methods::Octave;
use crate::simd::arch_simd::ArchSimd;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NoiseConfig<const D: usize> {
    pub seed: u64,
    pub octaves: usize,
    pub frequency: f32,
    pub amplitude: f32,
    pub lacunarity: f32,
    pub persistence: f32,
    pub normalization: bool,
    pub initialization: bool,
    pub finalization: bool,
    pub magnification: f32,
    pub scaling: [f32; D],
}

impl<const D: usize> NoiseConfig<D> {
    pub(crate) fn num_grid_octaves(&self) -> usize {
        let max_scaling = self.scaling.iter().fold(0.0, |max, x| x.max(max));

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

pub(crate) struct CustomBuilderConfig<'a, const D: usize> {
    pub(crate) octave_list: &'a [Octave<D>],
}

impl<'a, const D: usize> CustomBuilderConfig<'a, D> {
    pub(crate) fn normalize_grid_amplitude(&self, amplitude: f32) -> f32 {
        let mut sum = 0.0;
        for octave in self.octave_list {
            let max_freq = octave.frequency.iter().fold(0.0, |max, x| x.max(max));
            if max_freq < 1.0 {
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct GridConfig<const D: usize> {
    pub(crate) grid_seed: u64,
    pub(crate) grid_size: [usize; D],
    pub(crate) position: [i32; D],
    pub(crate) tiling: [Option<u32>; D],
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
