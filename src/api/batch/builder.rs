use std::marker::PhantomData;

use itertools::{Zip, multizip};

use crate::api::batch::interface::{BatchGenerator, BatchNoise, DimIter};
use crate::api::configs::*;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::noise::combiners::Combiner;
use crate::simd::arch_simd::ArchSimd;
use crate::{HybridMulti, PingPong, Ridged, Terrace};

pub struct BatchNoiseBuilder<const D: usize, C: Combiner, G: BatchGenerator<D>, I: DimIter<D>> {
    pub(crate) noise_config: NoiseConfig<D>,
    pub(crate) combiner_config: C::Config,
    pub(crate) iters: I,
    pub(crate) _noise_type: PhantomData<G>,
}

params_noise_builder!(BatchNoiseBuilder, [const D: usize, C: Combiner, G: BatchGenerator<D>, I: DimIter<D>], [D, C, G, I]);
params_lacunarity_builder!(BatchNoiseBuilder, [const D: usize, C: Combiner, G: BatchGenerator<D>, I: DimIter<D>], [D, C, G, I]);
params_combiner_builder!(BatchNoiseBuilder, [const D: usize, C: Combiner<Config: Sized>, G: BatchGenerator<D>, I: DimIter<D>], [D, C, G, I]);
params_ridged_builder!(BatchNoiseBuilder, [const D: usize, G: BatchGenerator<D>, I: DimIter<D>], [D, Ridged, G, I]);
params_ping_pong_builder!(BatchNoiseBuilder, [const D: usize, G: BatchGenerator<D>, I: DimIter<D>], [D, PingPong, G, I]);
params_terrace_builder!(BatchNoiseBuilder, [const D: usize, G: BatchGenerator<D>, I: DimIter<D>], [D, Terrace, G, I]);
params_hybrid_multi_builder!(BatchNoiseBuilder, [const D: usize, G: BatchGenerator<D>, I: DimIter<D>], [D, HybridMulti, G, I]);
params_grid_seed_builder!(BatchNoiseBuilder, [const D: usize, C: Combiner, G: BatchGenerator<D>, I: DimIter<D>], [D, C, G, I]);
params_noise_scaling_2d!(BatchNoiseBuilder, [C: Combiner, G: BatchGenerator<2>, I: DimIter<2>], [2, C, G, I]);
params_noise_scaling_3d!(BatchNoiseBuilder, [C: Combiner, G: BatchGenerator<3>, I: DimIter<3>], [3, C, G, I]);

impl<C: Combiner, G: BatchGenerator<2>> BatchNoise<2, C, G> {
    /// Creates a new builder to easily configure batches of noise.
    pub fn builder<X, Y>(x_iter: X, y_iter: Y) -> BatchNoiseBuilder<2, C, G, Zip<(X, Y)>>
    where
        X: Iterator<Item = ArchSimd<f32>>,
        Y: Iterator<Item = ArchSimd<f32>>,
        Zip<(X, Y)>: DimIter<2>,
    {
        BatchNoiseBuilder::<2, C, G, _>::new(x_iter, y_iter)
    }
}

impl<G, C, X, Y> BatchNoiseBuilder<2, C, G, Zip<(X, Y)>>
where
    G: BatchGenerator<2>,
    C: Combiner,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y)>: DimIter<2>,
{
    pub fn new(x_iter: X, y_iter: Y) -> Self {
        Self {
            noise_config: Default::default(),
            combiner_config: Default::default(),
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<G>,
        }
    }

    pub fn from_configs(
        noise_config: NoiseConfig<2>,
        combiner_config: C::Config,
        x_iter: X,
        y_iter: Y,
    ) -> Self {
        Self {
            noise_config,
            combiner_config,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<G>,
        }
    }
}

impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
    /// Creates a new builder to easily configure batches of noise.
    pub fn builder<X, Y, Z>(
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> BatchNoiseBuilder<3, F, S, Zip<(X, Y, Z)>>
    where
        X: Iterator<Item = ArchSimd<f32>>,
        Y: Iterator<Item = ArchSimd<f32>>,
        Z: Iterator<Item = ArchSimd<f32>>,
        Zip<(X, Y, Z)>: DimIter<3>,
    {
        BatchNoiseBuilder::<3, F, S, _>::new(x_iter, y_iter, z_iter)
    }
}

impl<S, F, X, Y, Z> BatchNoiseBuilder<3, F, S, Zip<(X, Y, Z)>>
where
    S: BatchGenerator<3>,
    F: Combiner,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Z: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y, Z)>: DimIter<3>,
{
    pub fn new(x_iter: X, y_iter: Y, z_iter: Z) -> Self {
        Self {
            noise_config: Default::default(),
            combiner_config: Default::default(),
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: NoiseConfig<3>,
        combiner_config: F::Config,
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> Self {
        Self {
            noise_config,
            combiner_config,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }
}

impl<const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>>
    BatchNoiseBuilder<D, F, S, I>
{
    declare_fill!(self, output, {
        if self.noise_config.initialization {
            for (i, x) in self.into_iter().enumerate() {
                x.copy_to_slice(&mut output[i * ArchSimd::<f32>::LANES..]);
            }
        } else {
            for (i, x) in self.into_iter().enumerate() {
                let index = i * ArchSimd::<f32>::LANES;
                let cur = ArchSimd::from_slice(&output[index..]);
                let x = cur + x;
                x.copy_to_slice(&mut output[i..]);
            }
        }
    });

    declare_build!(self, { self.into_iter().collect() });

    declare_into_iter!(self, {
        BatchNoise::<D, F, S>::sample(self.noise_config, self.combiner_config, self.iters)
    });
}
