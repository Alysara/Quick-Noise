use std::marker::PhantomData;

use itertools::{Zip, multizip};

use crate::api::batch::interface::{BatchNoise, DimIter};
use crate::api::configs::*;
use crate::api::octave::Octave;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::noise::combiners::Combiner;
use crate::simd::arch_simd::ArchSimd;
use crate::{BatchGenerator, HybridMulti, PingPong, Ridged, Terrace};

pub struct OctaveBatchNoiseBuilder<
    'a,
    const D: usize,
    C: Combiner,
    G: BatchGenerator<D>,
    I: DimIter<D>,
> {
    noise_config: OctaveNoiseConfig<D>,
    combiner_config: C::Config,
    octave_list: &'a [Octave<D>],
    iters: I,
    _noise_type: PhantomData<G>,
}

params_noise_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, C: Combiner, G: BatchGenerator<D>, I: DimIter<D>], ['a, D, C, G, I]);
params_grid_seed_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, C: Combiner, G: BatchGenerator<D>, I: DimIter<D>], ['a, D, C, G, I]);
params_ridged_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, I: DimIter<D>], ['a, D, Ridged, T, I]);
params_combiner_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, C: Combiner<Config: Sized>, G: BatchGenerator<D>, I: DimIter<D>], ['a, D, C, G, I]);
params_ping_pong_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, I: DimIter<D>], ['a, D, PingPong, T, I]);
params_terrace_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, I: DimIter<D>], ['a, D, Terrace, T, I]);
params_hybrid_multi_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, I: DimIter<D>], ['a, D, HybridMulti, T, I]);
params_noise_scaling_2d!(OctaveBatchNoiseBuilder, ['a, C: Combiner, G: BatchGenerator<2>, I: DimIter<2>], ['a, 2, C, G, I]);
params_noise_scaling_3d!(OctaveBatchNoiseBuilder, ['a, C: Combiner, G: BatchGenerator<3>, I: DimIter<3>], ['a, 3, C, G, I]);

impl<F: Combiner, S: BatchGenerator<2>> BatchNoise<2, F, S> {
    /// Creates a new builder using a custom octave list to configure
    /// batches of noise.
    pub fn builder_with_octaves<'a, X, Y>(
        octave_list: &'a [Octave<2>],
        x_iter: X,
        y_iter: Y,
    ) -> OctaveBatchNoiseBuilder<'a, 2, F, S, Zip<(X, Y)>>
    where
        X: Iterator<Item = ArchSimd<f32>>,
        Y: Iterator<Item = ArchSimd<f32>>,
        Zip<(X, Y)>: DimIter<2>,
    {
        OctaveBatchNoiseBuilder::<'a, 2, F, S, _>::new(octave_list, x_iter, y_iter)
    }
}

impl<'a, S, F, X, Y> OctaveBatchNoiseBuilder<'a, 2, F, S, Zip<(X, Y)>>
where
    S: BatchGenerator<2>,
    F: Combiner,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y)>: DimIter<2>,
{
    pub fn new(octave_list: &'a [Octave<2>], x_iter: X, y_iter: Y) -> Self {
        Self {
            noise_config: Default::default(),
            combiner_config: Default::default(),
            octave_list,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: OctaveNoiseConfig<2>,
        combiner_config: F::Config,
        octave_list: &'a [Octave<2>],
        x_iter: X,
        y_iter: Y,
    ) -> Self {
        Self {
            noise_config,
            combiner_config,
            octave_list,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
        }
    }
}

impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
    /// Creates a new builder using a custom octave list to configure
    /// batches of noise.
    pub fn builder_with_octaves<'a, X, Y, Z>(
        octave_list: &'a [Octave<3>],
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> OctaveBatchNoiseBuilder<'a, 3, F, S, Zip<(X, Y, Z)>>
    where
        X: Iterator<Item = ArchSimd<f32>>,
        Y: Iterator<Item = ArchSimd<f32>>,
        Z: Iterator<Item = ArchSimd<f32>>,
        Zip<(X, Y, Z)>: DimIter<3>,
    {
        OctaveBatchNoiseBuilder::<3, F, S, _>::new(octave_list, x_iter, y_iter, z_iter)
    }
}

impl<'a, S, F, X, Y, Z> OctaveBatchNoiseBuilder<'a, 3, F, S, Zip<(X, Y, Z)>>
where
    S: BatchGenerator<3>,
    F: Combiner,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Z: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y, Z)>: DimIter<3>,
{
    pub fn new(octave_list: &'a [Octave<3>], x_iter: X, y_iter: Y, z_iter: Z) -> Self {
        Self {
            noise_config: Default::default(),
            combiner_config: Default::default(),
            octave_list,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: OctaveNoiseConfig<3>,
        combiner_config: F::Config,
        octave_list: &'a [Octave<3>],
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> Self {
        Self {
            noise_config,
            combiner_config,
            octave_list,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }
}

impl<'a, const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>>
    OctaveBatchNoiseBuilder<'a, D, F, S, I>
{
    declare_fill!(self, output, {
        if self.noise_config.initialize {
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
        BatchNoise::<D, F, S>::sample_with_octaves(
            self.noise_config,
            self.combiner_config,
            self.octave_list,
            self.iters,
        )
    });
}
