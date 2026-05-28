use crate::api::configs::*;
use crate::api::parameters::*;
use crate::api::seed::OctaveSeed;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3, VecHorzMax};
use crate::perlin::{PerlinGridNoise2D, PerlinGridNoise3D};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;

// ————————————————————————————————————————————————————————————————
// ————— 2D Perlin Uniform Grid ———————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// A struct for creating perlin noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default)]
pub struct PerlinGrid2D<const X: usize, const Y: usize, const N: usize> {
    grid_config: GridConfig2D,
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig2D,
}

params_general_builder!(PerlinGrid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);
params_fbm_builder!(PerlinGrid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);
params_fbm_scaling_2d!(PerlinGrid2D, [const X: usize, const Y: usize, const N: usize], [X, Y, N]);

impl<const X: usize, const Y: usize, const N: usize> PerlinGrid2D<X, Y, N> {
    #[inline(always)]
    pub(crate) fn new(grid_config: GridConfig2D) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    pub(crate) fn fbm_noise<const INITIALIZE: bool>(self, result: &mut SimdArray<f32, N>) {
        let octaves = 'outer: {
            let max_scaling = self.fbm_config.scaling.horizontal_max();

            let mut cur_freq = self.fbm_config.frequency * max_scaling;
            if cur_freq >= 1.0 || self.fbm_config.lacunarity >= 1.0 {
                for i in 0..self.fbm_config.octaves {
                    if cur_freq >= 1.0 {
                        break 'outer i;
                    }
                    cur_freq *= self.fbm_config.lacunarity;
                }
            }

            self.fbm_config.octaves
        };

        if octaves == 0 {
            if INITIALIZE {
                result.fill(0.0)
            }
            return;
        }

        let position = self.grid_config.position;
        let magnification = self.general_config.magnification;

        let seed =
            Random::static_mix_u64_pair(self.general_config.seed, self.grid_config.grid_seed);

        // FBM algorithm:
        let mut frequency = self.fbm_config.scaling * self.fbm_config.frequency;

        let mut weight = if self.general_config.normalization {
            let mut sum = 0.0;
            let mut cur = 1.0;
            for _ in 0..octaves {
                sum += cur;
                cur *= self.fbm_config.persistence;
            }
            self.general_config.amplitude / sum
        } else {
            self.general_config.amplitude
        };

        // First octave:
        let first_seed = (self.fbm_config.frequency * self.fbm_config.scaling).octave_seed(seed);
        PerlinGridNoise2D::<X, Y, N>::grid_2d::<INITIALIZE>(
            first_seed,
            result,
            position,
            frequency,
            weight,
            magnification,
        );

        // Subsequent octaves:
        for _ in 1..octaves {
            weight *= self.fbm_config.persistence;
            frequency *= self.fbm_config.lacunarity;

            let octave_seed =
                (self.fbm_config.frequency * self.fbm_config.scaling).octave_seed(seed);
            PerlinGridNoise2D::<X, Y, N>::grid_2d::<false>(
                octave_seed,
                result,
                position,
                frequency,
                weight,
                magnification,
            );
        }
    }

    declare_build!(self, {
        let mut result = unsafe { SimdArray::new_uninit() };
        self.fbm_noise::<true>(&mut result);
        result
    });

    declare_fill!(self, result, {
        self.fbm_noise::<true>(result);
    });

    declare_fill_onto!(self, result, {
        self.fbm_noise::<false>(result);
    });

    declare_into_iter!(self, { self.build().into_iter() });
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Perlin Uniform Grid ———————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// A struct for creating perlin noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default)]
pub struct PerlinGrid3D<const X: usize, const Y: usize, const Z: usize, const N: usize> {
    grid_config: GridConfig3D,
    general_config: GeneralBuilderConfig,
    fbm_config: FBMBuilderConfig3D,
}

params_general_builder!(PerlinGrid3D, [const X: usize, const Y: usize, const Z: usize, const N: usize], [X, Y, Z, N]);
params_fbm_builder!(PerlinGrid3D, [const X: usize, const Y: usize, const Z: usize, const N: usize], [X, Y, Z, N]);
params_fbm_scaling_3d!(PerlinGrid3D, [const X: usize, const Y: usize, const Z: usize, const N: usize], [X, Y, Z, N]);

impl<const X: usize, const Y: usize, const Z: usize, const N: usize> PerlinGrid3D<X, Y, Z, N> {
    #[inline(always)]
    pub(crate) fn new(grid_config: GridConfig3D) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    #[inline(always)]
    pub(crate) fn fbm_noise<const INITIALIZE: bool>(self, result: &mut SimdArray<f32, N>) {
        let octaves = 'outer: {
            let max_scaling = self.fbm_config.scaling.horizontal_max();

            let mut cur_freq = self.fbm_config.frequency * max_scaling;
            if cur_freq >= 1.0 || self.fbm_config.lacunarity >= 1.0 {
                for i in 0..self.fbm_config.octaves {
                    if cur_freq >= 1.0 {
                        break 'outer i;
                    }
                    cur_freq *= self.fbm_config.lacunarity;
                }
            }

            self.fbm_config.octaves
        };

        if octaves == 0 {
            if INITIALIZE {
                result.fill(0.0)
            }
            return;
        }

        let seed =
            Random::static_mix_u64_pair(self.general_config.seed, self.grid_config.grid_seed);

        let position = self.grid_config.position;
        let magnification = self.general_config.magnification;

        // FBM algorithm:
        let mut frequency = self.fbm_config.scaling * self.fbm_config.frequency;
        let mut weight = if self.general_config.normalization {
            let mut sum = 0.0;
            let mut cur = 1.0;
            for _ in 0..octaves {
                sum += cur;
                cur *= self.fbm_config.persistence;
            }
            self.general_config.amplitude / sum
        } else {
            self.general_config.amplitude
        };

        // First octave:
        let first_seed = frequency.octave_seed(seed);
        PerlinGridNoise3D::<X, Y, Z, N>::grid_3d::<INITIALIZE>(
            first_seed,
            result,
            position,
            frequency,
            weight,
            magnification,
        );

        // Subsequent octaves:
        for _ in 1..octaves {
            weight *= self.fbm_config.persistence;
            frequency *= self.fbm_config.lacunarity;

            let octave_seed = frequency.octave_seed(seed);
            PerlinGridNoise3D::<X, Y, Z, N>::grid_3d::<false>(
                octave_seed,
                result,
                position,
                frequency,
                weight,
                magnification,
            );
        }
    }

    declare_build!(self, {
        let mut result = unsafe { SimdArray::new_uninit() };
        self.fbm_noise::<true>(&mut result);
        result
    });

    declare_fill!(self, result, {
        self.fbm_noise::<true>(result);
    });

    declare_fill_onto!(self, result, {
        self.fbm_noise::<false>(result);
    });

    declare_into_iter!(self, { self.build().into_iter() });
}
