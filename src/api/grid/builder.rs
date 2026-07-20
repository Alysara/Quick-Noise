use std::marker::PhantomData;

use crate::api::configs::*;
use crate::api::grid::interface::{GridGenerator, GridNoise};
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::simd::StaticArch;
use crate::simd::register::iters::IntoSimdIterator;
use crate::{Combiner, HybridMulti, PingPong, Ridged, Terrace};

/// A struct for creating FBM noise set on a uniform grid.
/// The most performant way to generate Perlin noise.
#[derive(Default, Copy, Clone)]
pub struct GridNoiseBuilder<const D: usize, C: Combiner, G: GridGenerator<D>> {
    grid_config: GridConfig<D>,
    noise_config: NoiseConfig<D>,
    combiner_config: C::Config,
    _noise_type: PhantomData<G>,
}

params_noise_builder!(GridNoiseBuilder, [const D: usize, C: Combiner, G: GridGenerator<D>], [D, C, G]);
params_lacunarity_builder!(GridNoiseBuilder, [const D: usize, C: Combiner, G: GridGenerator<D>], [D, C, G]);
params_combiner_builder!(GridNoiseBuilder, [const D: usize, C: Combiner<Config: Sized>, G: GridGenerator<D>], [D, C, G]);
params_ridged_builder!(GridNoiseBuilder, [const D: usize, G: GridGenerator<D>], [D, Ridged, G]);
params_ping_pong_builder!(GridNoiseBuilder, [const D: usize, G: GridGenerator<D>], [D, PingPong, G]);
params_terrace_builder!(GridNoiseBuilder, [const D: usize, G: GridGenerator<D>], [D, Terrace, G]);
params_hybrid_multi_builder!(GridNoiseBuilder, [const D: usize, G: GridGenerator<D>], [D, HybridMulti, G]);
params_noise_scaling_2d!(GridNoiseBuilder, [C: Combiner, G: GridGenerator<2>], [2, C, G]);
params_noise_scaling_3d!(GridNoiseBuilder, [C: Combiner, G: GridGenerator<3>], [3, C, G]);

impl<const D: usize, C: Combiner, G: GridGenerator<D>> GridNoiseBuilder<D, C, G> {
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
        GridNoise::<D, C, G>::sample::<StaticArch>(
            &self.grid_config,
            &self.noise_config,
            &self.combiner_config,
            result.as_mut_slice(),
        );
        result
    });

    declare_fill!(self, result, {
        GridNoise::<D, C, G>::sample::<StaticArch>(
            &self.grid_config,
            &self.noise_config,
            &self.combiner_config,
            result,
        );
    });

    declare_into_iter!(StaticArch, self, { self.build().into_simd_iter() });
}
