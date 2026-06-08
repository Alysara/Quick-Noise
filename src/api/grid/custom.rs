use std::marker::PhantomData;

use crate::api::configs::*;
use crate::api::grid::interface::GridNoise;
use crate::api::methods::{NoiseDimension, Octave};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::math::vec::VecHorzMax;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::simd_array::SimdArray;

/// A struct for creating 2D Perlin noise set on a uniform grid with
/// a custom list of octaves. Uses the performant perlin algorithm.
pub struct CustomGridBuilder<
    'a,
    T: GridNoise,
    D: NoiseDimension,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
> {
    grid_config: GridConfig<D>,
    general_config: GeneralBuilderConfig,
    custom_config: CustomBuilderConfig<'a, D>,
    _noise_type: PhantomData<T>,
}

params_general_builder!(
    CustomGridBuilder, ['a, T: GridNoise, D: NoiseDimension, const X: usize, const Y: usize, const Z: usize, const N: usize],
    ['a, T, D, X, Y, Z, N]
);

impl<
    'a,
    T: GridNoise,
    D: NoiseDimension,
    const X: usize,
    const Y: usize,
    const Z: usize,
    const N: usize,
> CustomGridBuilder<'a, T, D, X, Y, Z, N>
{
    pub(crate) fn new(grid_config: GridConfig<D>, octave_list: &'a [Octave<D>]) -> Self {
        Self {
            grid_config,
            general_config: Default::default(),
            custom_config: CustomBuilderConfig::<'a, D> { octave_list },
            _noise_type: PhantomData::<T>,
        }
    }

    #[inline(always)]
    pub(crate) fn custom_noise<const INITIALIZE: bool>(self, result: &mut SimdArray<f32, N>) {
        let position = self.grid_config.position;
        let magnification = self.general_config.magnification;

        let seed =
            Random::static_mix_u64_pair(self.general_config.seed, self.grid_config.grid_seed);

        let weight_coef = if self.general_config.normalization {
            self.custom_config
                .normalize_grid_amplitude(self.general_config.amplitude)
        } else {
            self.general_config.amplitude
        };

        // First octave:
        let mut octave_iter = self.custom_config.octave_list.iter();
        let mut cur = octave_iter.next();
        while cur.is_some() {
            let octave = cur.unwrap();
            if octave.frequency.horizontal_max() < 1.0 {
                let octave_seed = D::octave_seed(octave.frequency, seed);
                D::grid::<T, X, Y, Z, N, INITIALIZE>(
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
            if octave.frequency.horizontal_max() < 1.0 {
                let octave_seed = D::octave_seed(octave.frequency, seed);
                D::grid::<T, X, Y, Z, N, false>(
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
