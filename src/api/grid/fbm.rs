use std::marker::PhantomData;

use crate::api::configs::*;
use crate::api::grid::interface::{GridNoiseImpl, GridNoiseParams};
use crate::api::parameters::*;
use crate::api::seed::gen_octave_seed;
use crate::math::random::Random;
use crate::math::vec::{BasicVec, Vec2, Vec3, VecHorz};
use crate::simd::SimdSliceIterExt;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;
use crate::simd::simd_reg::iters::IntoSimdIterator;

#[inline(always)]
pub fn fbm_noise<const D: usize, T: GridNoiseImpl<D>, const INIT: bool>(
    grid_config: &GridConfig<D>,
    general_config: &GeneralConfig,
    fbm_config: &FbmConfig<D>,
    result: &mut [f32],
) {
    let octaves = fbm_config.num_grid_octaves();

    // Fill with zeroes if there are no octaves.
    if octaves == 0 {
        if INIT {
            result.fill(0.0)
        }
        return;
    }

    let base_seed = Random::static_mix_u64_pair(grid_config.grid_seed, general_config.seed);

    // FBM algorithm:
    let mut frequency = std::array::from_fn(|i| fbm_config.scaling[i] * fbm_config.frequency);
    let mut weight = if general_config.normalization {
        fbm_config.normalize_amplitude(general_config.amplitude)
    } else {
        general_config.amplitude
    };

    // First octave:
    let first_seed = gen_octave_seed(frequency, base_seed);

    let mut params = GridNoiseParams {
        seed: gen_octave_seed(frequency, base_seed),
        grid_size: grid_config.dimensions,
        position: grid_config.position,
        magnification: general_config.magnification,
        tiling: grid_config.tiling,
        frequency,
        weight,
    };

    T::sample::<INIT>(params, result);

    // Subsequent octaves:
    for _ in 1..octaves {
        params.weight *= fbm_config.persistence;
        params.frequency = std::array::from_fn(|i| params.frequency[i] * fbm_config.lacunarity);
        params.seed = gen_octave_seed(params.frequency, base_seed);

        T::sample::<false>(params, result);
    }
}

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default, Copy, Clone)]
pub struct FbmGridBuilder<const D: usize, T: GridNoiseImpl<D>> {
    grid_config: GridConfig<D>,
    general_config: GeneralConfig,
    fbm_config: FbmConfig<D>,
    _noise_type: PhantomData<T>,
}

params_general_builder!(FbmGridBuilder, [const D: usize, T: GridNoiseImpl<D>], [D, T]);
params_fbm_builder!(FbmGridBuilder, [const D: usize, T: GridNoiseImpl<D>], [D, T]);
params_fbm_scaling_2d!(FbmGridBuilder, [T: GridNoiseImpl<2>], [2, T]);
params_fbm_scaling_3d!(FbmGridBuilder, [T: GridNoiseImpl<3>], [3, T]);

impl<const D: usize, T: GridNoiseImpl<D>> FbmGridBuilder<D, T> {
    #[inline(always)]
    pub(crate) fn from_config(grid_config: GridConfig<D>) -> Self {
        let mut config = Self::default();
        config.grid_config = grid_config;
        config
    }

    declare_build!(self, {
        let size = self.grid_config.dimensions.iter().fold(1, |p, x| p * x);
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
