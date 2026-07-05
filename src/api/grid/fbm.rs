use std::marker::PhantomData;

use crate::api::configs::*;
use crate::api::grid::interface::GridNoiseImpl;
use crate::api::methods::{Dim2, Dim3, NoiseDimension};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::{BasicVec, Vec2, Vec3, VecHorz};
use crate::simd::SimdSliceIterExt;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_reg::iters::IntoSimdIterator;

#[inline(always)]
pub fn fbm_noise<D: NoiseDimension, T: GridNoiseImpl, const INITIALIZE: bool>(
    grid_config: &GridConfig<D>,
    general_config: &GeneralConfig,
    fbm_config: &FbmConfig<D>,
    result: &mut [f32],
) {
    let octaves = fbm_config.num_grid_octaves();

    if octaves == 0 {
        if INITIALIZE {
            result.fill(0.0)
        }
        return;
    }

    let dimensions = grid_config.dimensions;
    let position = grid_config.position;
    let magnification = general_config.magnification;

    let seed = Random::static_mix_u64_pair(grid_config.grid_seed, general_config.seed);

    // FBM algorithm:
    let mut frequency = fbm_config.scaling * D::FVec::splat(fbm_config.frequency);
    let mut weight = if general_config.normalization {
        fbm_config.normalize_amplitude(general_config.amplitude)
    } else {
        general_config.amplitude
    };

    // First octave:
    let first_seed = D::octave_seed(frequency * fbm_config.scaling, seed);
    D::grid::<T, INITIALIZE>(
        first_seed,
        result,
        dimensions,
        position,
        frequency,
        weight,
        magnification,
        grid_config.tiling,
    );

    // Subsequent octaves:
    for _ in 1..octaves {
        weight *= fbm_config.persistence;
        frequency *= D::FVec::splat(fbm_config.lacunarity);

        let octave_seed = D::octave_seed(frequency * fbm_config.scaling, seed);
        D::grid::<T, false>(
            octave_seed,
            result,
            dimensions,
            position,
            frequency,
            weight,
            magnification,
            grid_config.tiling,
        );
    }
}

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default, Copy, Clone)]
pub struct FbmGridBuilder<D: NoiseDimension, T: GridNoiseImpl> {
    grid_config: GridConfig<D>,
    general_config: GeneralConfig,
    fbm_config: FbmConfig<D>,
    _noise_type: PhantomData<T>,
}

params_general_builder!(FbmGridBuilder, [D: NoiseDimension, T: GridNoiseImpl], [D, T]);
params_fbm_builder!(FbmGridBuilder, [D: NoiseDimension, T: GridNoiseImpl], [D, T]);
params_fbm_scaling_2d!(FbmGridBuilder, [T: GridNoiseImpl], [Dim2, T]);
params_fbm_scaling_3d!(FbmGridBuilder, [T: GridNoiseImpl], [Dim3, T]);

impl<D: NoiseDimension, T: GridNoiseImpl> FbmGridBuilder<D, T> {
    #[inline(always)]
    pub(crate) fn from_config(grid_config: GridConfig<D>) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    declare_build!(self, {
        let size = self.grid_config.dimensions.horizontal_product();
        let mut result = vec![0.0; size];
        fbm_noise::<D, T, true>(
            &self.grid_config,
            &self.general_config,
            &self.fbm_config,
            &mut result.as_mut_slice(),
        );
        result
    });

    declare_fill!(self, result, {
        fbm_noise::<D, T, true>(
            &self.grid_config,
            &self.general_config,
            &self.fbm_config,
            result,
        );
    });

    declare_fill_onto!(self, result, {
        fbm_noise::<D, T, false>(
            &self.grid_config,
            &self.general_config,
            &self.fbm_config,
            result,
        );
    });

    declare_into_iter!(self, { self.build().into_simd_iter() });
}
