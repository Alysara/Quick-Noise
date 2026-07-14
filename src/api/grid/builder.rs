use std::marker::PhantomData;

use crate::api::configs::*;
use crate::api::grid::interface::{GridNoise, GridGenerator};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::simd::arch_simd::ArchSimd;
use crate::simd::register::iters::IntoSimdIterator;
use crate::{Combiner, HybridMulti, PingPong, Ridged, Terrace};

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default, Copy, Clone)]
pub struct GridNoiseBuilder<const D: usize, F: Combiner, T: GridGenerator<D>> {
    grid_config: GridConfig<D>,
    noise_config: NoiseConfig<D>,
    combiner_config: F::Config,
    _noise_type: PhantomData<T>,
}

params_noise_builder!(GridNoiseBuilder, [const D: usize, F: Combiner, T: GridGenerator<D>], [D, F, T]);
params_lacunarity_builder!(GridNoiseBuilder, [const D: usize, F: Combiner, T: GridGenerator<D>], [D, F, T]);
params_ridged_builder!(GridNoiseBuilder, [const D: usize, T: GridGenerator<D>], [D, Ridged, T]);
params_ping_pong_builder!(GridNoiseBuilder, [const D: usize, T: GridGenerator<D>], [D, PingPong, T]);
params_terrace_builder!(GridNoiseBuilder, [const D: usize, T: GridGenerator<D>], [D, Terrace, T]);
params_hybrid_multi_builder!(GridNoiseBuilder, [const D: usize, T: GridGenerator<D>], [D, HybridMulti, T]);
params_noise_scaling_2d!(GridNoiseBuilder, [F: Combiner, T: GridGenerator<2>], [2, F, T]);
params_noise_scaling_3d!(GridNoiseBuilder, [F: Combiner, T: GridGenerator<3>], [3, F, T]);

impl<const D: usize, F: Combiner, T: GridGenerator<D>> GridNoiseBuilder<D, F, T> {
    #[inline(always)]
    pub(crate) fn from_config(grid_config: GridConfig<D>) -> Self {
        Self {
            grid_config,
            ..Default::default()
        }
    }

    declare_build!(self, {
        let size = self.grid_config.grid_size.iter().product();
        let mut result = vec![0.0; size];
        GridNoise::<D, F, T>::sample(
            &self.grid_config,
            &self.noise_config,
            &self.combiner_config,
            result.as_mut_slice(),
        );
        result
    });

    declare_fill!(self, result, {
        GridNoise::<D, F, T>::sample(
            &self.grid_config,
            &self.noise_config,
            &self.combiner_config,
            result,
        );
    });

    // declare_fill_onto!(self, result, {
    //     sample_grid::<D, F, T, true>(
    //         &self.grid_config,
    //         &self.noise_config,
    //         self.fractal_config,
    //         result,
    //     );
    // });
    //
    //

    declare_into_iter!(self, { self.build().into_simd_iter() });
}
