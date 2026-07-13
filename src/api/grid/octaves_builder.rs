use std::marker::PhantomData;

use crate::{Combiner, GridGenerator, Ridged};
use crate::api::configs::*;
use crate::api::grid::interface::GridNoise;
use crate::api::octave::Octave;
use crate::api::parameters::*;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::register::iters::IntoSimdIterator;

/// A struct for creating 2D Perlin noise set on a uniform grid with
/// a custom list of octaves. Uses the performant perlin algorithm.
#[derive(Default)]
pub struct OctaveGridNoiseBuilder<'a, const D: usize, F: Combiner, T: GridGenerator<D>> {
    grid_config: GridConfig<D>,
    noise_config: NoiseConfig<D>,
    fractal_config: F::Config,
    octave_list: &'a [Octave<D>],
    _noise_type: PhantomData<T>,
}

params_lacunarity_builder!(OctaveGridNoiseBuilder, ['a, const D: usize, F: Combiner, T: GridGenerator<D>], ['a, D, F, T]);
params_ridged_builder!(OctaveGridNoiseBuilder, ['a, const D: usize, T: GridGenerator<D>], ['a, D, Ridged, T]);
params_noise_scaling_2d!(OctaveGridNoiseBuilder, ['a, F: Combiner, T: GridGenerator<2>], ['a, 2, F, T]);
params_noise_scaling_3d!(OctaveGridNoiseBuilder, ['a, F: Combiner, T: GridGenerator<3>], ['a, 3, F, T]);

impl<'a, const D: usize, F: Combiner, T: GridGenerator<D>> OctaveGridNoiseBuilder<'a, D, F, T> {
    pub(crate) fn new(grid_config: GridConfig<D>, octave_list: &'a [Octave<D>]) -> Self {
        Self {
            grid_config,
            octave_list,
            ..Default::default()
        }
    }

    declare_build!(self, {
        let size = self.grid_config.grid_size.iter().product();
        let mut result = vec![0.0; size];
        GridNoise::<D, F, T>::sample_with_octaves(
            &self.grid_config,
            &self.noise_config,
            &self.fractal_config,
            self.octave_list,
            result.as_mut_slice(),
        );
        result
    });

    declare_fill!(self, result, {
        GridNoise::<D, F, T>::sample_with_octaves(
            &self.grid_config,
            &self.noise_config,
            &self.fractal_config,
            self.octave_list,
            result,
        );
    });

    // declare_fill_onto!(self, result, {
    //     self.custom_noise::<false>(result);
    // });

    declare_into_iter!(self, { self.build().into_simd_iter() });
}
