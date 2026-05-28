use crate::api::configs::*;
use crate::api::parameters::*;
use crate::api::seed::OctaveSeed;
use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3, VecHorzMax};
use crate::perlin::{Octave2D, Octave3D, PerlinGridNoise2D, PerlinGridNoise3D};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_traits::*;

// ————————————————————————————————————————————————————————————————
// ————— 2D Custom Perlin Grid ————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// A struct for creating 2D Perlin noise set on a uniform grid with
/// a custom list of octaves. Uses the performant perlin algorithm.
#[derive(Default)]
pub struct CustomPerlinGrid2D<'a, const X: usize, const Y: usize, const N: usize> {
    pub(crate) grid_config: GridConfig2D,
    pub(crate) general_config: GeneralBuilderConfig,
    pub(crate) custom_config: CustomBuilderConfig<'a, Octave2D>,
}

params_general_builder!(CustomPerlinGrid2D, ['a, const X: usize, const Y: usize, const N: usize], ['a, X, Y, N]);
params_custom_builder_2d!(
    CustomPerlinGrid2D,
    [const X: usize, const Y: usize, const N: usize],
    [X, Y, N],
    self, octave_list, {
        CustomPerlinGrid2D {
            grid_config: self.grid_config,
            general_config: self.general_config,
            custom_config: CustomBuilderConfig {
                octave_list: octave_list,
            },
        }
    }
);

impl<'a, const X: usize, const Y: usize, const N: usize> CustomPerlinGrid2D<'a, X, Y, N> {
    #[inline(always)]

    pub(crate) fn custom_noise<const INITIALIZE: bool>(self, result: &mut SimdArray<f32, N>) {
        let position = self.grid_config.position;
        let magnification = self.general_config.magnification;

        let seed =
            Random::static_mix_u64_pair(self.general_config.seed, self.grid_config.grid_seed);

        let weight_coef = if self.general_config.normalization {
            let mut sum = 0.0;
            for octave in self.custom_config.octave_list {
                if octave.frequency.horizontal_max() < 1.0 {
                    sum += octave.weight;
                }
            }

            if sum == 0.0 {
                result.fill(0.0);
                return;
            }

            self.general_config.amplitude / sum
        } else {
            self.general_config.amplitude
        };

        // First octave:
        let mut octave_iter = self.custom_config.octave_list.iter();
        let mut cur = octave_iter.next();
        while cur.is_some() {
            let octave = cur.unwrap();
            if octave.frequency.horizontal_max() < 1.0 {
                let octave_seed = octave.frequency.octave_seed(seed);
                PerlinGridNoise2D::<X, Y, N>::grid_2d::<INITIALIZE>(
                    octave_seed,
                    result,
                    position,
                    octave.frequency,
                    octave.weight * weight_coef,
                    magnification,
                );
                break;
            }
            cur = octave_iter.next();
        }

        // Subsequent octaves:
        for octave in octave_iter {
            if octave.frequency.x.max(octave.frequency.y) < 1.0 {
                let octave_seed = octave.frequency.octave_seed(seed);
                PerlinGridNoise2D::<X, Y, N>::grid_2d::<false>(
                    octave_seed,
                    result,
                    position,
                    octave.frequency,
                    octave.weight * weight_coef,
                    magnification,
                );
            }
        }
    }

    declare_build!(self, {
        let mut result = unsafe { SimdArray::new_uninit() };
        self.custom_noise::<true>(&mut result);
        result
    });

    declare_fill!(self, result, {
        self.custom_noise::<true>(result);
    });

    declare_fill_onto!(self, result, {
        self.custom_noise::<false>(result);
    });

    declare_into_iter!(self, { self.build().into_iter() });
}

// ————————————————————————————————————————————————————————————————
// ————— 3D Custom Perlin Grid ————————————————————————————————————
// ————————————————————————————————————————————————————————————————

/// A struct for creating 3D Perlin noise set on a uniform grid with
/// a custom list of octaves. Uses the performant perlin algorithm.
#[derive(Default)]
pub struct CustomPerlinGrid3D<'a, const X: usize, const Y: usize, const Z: usize, const N: usize> {
    pub(crate) grid_config: GridConfig3D,
    pub(crate) general_config: GeneralBuilderConfig,
    pub(crate) custom_config: CustomBuilderConfig<'a, Octave3D>,
}

params_general_builder!(CustomPerlinGrid3D,
    ['a, const X: usize, const Y: usize, const Z: usize, const N: usize],
    ['a, X, Y, Z, N]
);
params_custom_builder_3d!(
    CustomPerlinGrid3D,
    [const X: usize, const Y: usize, const Z: usize, const N: usize],
    [X, Y, Z, N],
    self, octave_list, {
        CustomPerlinGrid3D {
            grid_config: self.grid_config,
            general_config: self.general_config,
            custom_config: CustomBuilderConfig {
                octave_list: octave_list,
            },
        }
    }
);

impl<'a, const X: usize, const Y: usize, const Z: usize, const N: usize>
    CustomPerlinGrid3D<'a, X, Y, Z, N>
{
    #[inline(always)]
    fn new(grid_config: GridConfig3D) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    #[inline(always)]
    pub(crate) fn octave_seed(seed: u64, frequency: Vec3<f32>) -> u32 {
        Random::mix_u64_triple(
            seed.wrapping_mul(frequency.x.to_bits() as u64),
            seed.wrapping_mul(frequency.y.to_bits() as u64),
            seed.wrapping_mul(frequency.z.to_bits() as u64),
        ) as u32
    }

    pub(crate) fn custom_noise<const INITIALIZE: bool>(self, result: &mut SimdArray<f32, N>) {
        let position = self.grid_config.position;
        let magnification = self.general_config.magnification;

        let seed =
            Random::static_mix_u64_pair(self.general_config.seed, self.grid_config.grid_seed);

        let weight_coef = if self.general_config.normalization {
            let mut sum = 0.0;
            for octave in self.custom_config.octave_list {
                if octave.frequency.horizontal_max() < 1.0 {
                    sum += octave.weight;
                }
            }

            if sum == 0.0 {
                result.fill(0.0);
                return;
            }

            self.general_config.amplitude / sum
        } else {
            self.general_config.amplitude
        };

        // First octave:
        let mut octave_iter = self.custom_config.octave_list.iter();
        let mut cur = octave_iter.next();
        while cur.is_some() {
            let octave = cur.unwrap();
            if octave.frequency.horizontal_max() < 1.0 {
                let octave_seed = octave.frequency.octave_seed(seed);
                PerlinGridNoise3D::<X, Y, Z, N>::grid_3d::<INITIALIZE>(
                    octave_seed,
                    result,
                    position,
                    octave.frequency,
                    octave.weight * weight_coef,
                    magnification,
                );
            }
            cur = octave_iter.next();
        }

        // Subsequent octaves:
        while cur.is_some() {
            let octave = cur.unwrap();
            if octave.frequency.x.max(octave.frequency.y) < 1.0 {
                let octave_seed = octave.frequency.octave_seed(seed);
                PerlinGridNoise3D::<X, Y, Z, N>::grid_3d::<false>(
                    octave_seed,
                    result,
                    position,
                    octave.frequency,
                    octave.weight * weight_coef,
                    magnification,
                );
            }
            cur = octave_iter.next();
        }
    }

    declare_build!(self, {
        let mut result = unsafe { SimdArray::new_uninit() };
        self.custom_noise::<true>(&mut result);
        result
    });

    declare_fill!(self, result, {
        self.custom_noise::<true>(result);
    });

    declare_fill_onto!(self, result, {
        self.custom_noise::<false>(result);
    });

    declare_into_iter!(self, { self.build().into_iter() });
}
