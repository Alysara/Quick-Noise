use std::marker::PhantomData;

use crate::api::configs::*;
use crate::api::grid::interface::GridNoise;
use crate::api::methods::{Dim2, Dim3, NoiseDimension};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{BasicVec, Vec2, Vec3};
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default)]
pub struct FbmGridBuilder<
    T: GridNoise,
    D: NoiseDimension,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
> {
    grid_config: GridConfig<D>,
    general_config: GeneralBuilderConfig,
    fbm_config: FbmBuilderConfig<D>,
    _noise_type: PhantomData<T>,
}

params_general_builder!(FbmGridBuilder, [T: GridNoise, D: NoiseDimension, const X: usize, const Y: usize, const Z: usize, const N: usize], [T, D, X, Y, Z, N]);
params_fbm_builder!(FbmGridBuilder, [T: GridNoise, D: NoiseDimension, const X: usize, const Y: usize, const Z: usize, const N: usize], [T, D, X, Y, Z, N]);
params_fbm_scaling_2d!(FbmGridBuilder, [T: GridNoise, const X: usize, const Y: usize, const Z: usize, const N: usize], [T, Dim2, X, Y, Z, N]);
params_fbm_scaling_3d!(FbmGridBuilder, [T: GridNoise, const X: usize, const Y: usize, const Z: usize, const N: usize], [T, Dim3, X, Y, Z, N]);

impl<
    T: GridNoise,
    D: NoiseDimension,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
> FbmGridBuilder<T, D, X, Y, Z, N>
{
    #[inline(always)]
    pub(crate) fn new(grid_config: GridConfig<D>) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    pub(crate) fn fbm_noise<const INITIALIZE: bool>(self, result: &mut SimdArray<f32, N>) {
        let octaves = self.fbm_config.num_grid_octaves();

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
        let mut frequency = self.fbm_config.scaling * D::FVec::splat(self.fbm_config.frequency);
        let mut weight = if self.general_config.normalization {
            self.fbm_config
                .normalize_amplitude(self.general_config.amplitude)
        } else {
            self.general_config.amplitude
        };

        // First octave:
        let first_seed = D::octave_seed(frequency * self.fbm_config.scaling, seed);
        D::grid::<T, X, Y, Z, N, INITIALIZE>(
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
            frequency *= D::FVec::splat(self.fbm_config.lacunarity);

            let octave_seed = D::octave_seed(frequency * self.fbm_config.scaling, seed);
            D::grid::<T, X, Y, Z, N, false>(
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
