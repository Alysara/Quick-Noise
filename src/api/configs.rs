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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OctaveNoiseConfig<const D: usize> {
    pub seed: u64,
    pub amplitude: f32,
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GridConfig<const D: usize> {
    pub grid_seed: u64,
    pub grid_size: [usize; D],
    pub position: [i32; D],
    pub tiling: [Option<u32>; D],
}
