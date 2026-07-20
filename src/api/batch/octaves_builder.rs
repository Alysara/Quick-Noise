use std::marker::PhantomData;

use itertools::{Zip, multizip};

use crate::api::batch::interface::{BatchNoise, DimIter};
use crate::api::configs::*;
use crate::api::octave::Octave;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::noise::combiners::Combiner;
use crate::simd::{Arch, Simd};
use crate::{BatchGenerator, HybridMulti, PingPong, Ridged, Terrace};

pub struct OctaveBatchNoiseBuilder<
    'a,
    const D: usize,
    C: Combiner,
    G: BatchGenerator<D>,
    A: Arch,
    I: DimIter<A, D>,
> {
    noise_config: OctaveNoiseConfig<D>,
    combiner_config: C::Config,
    octave_list: &'a [Octave<D>],
    iters: I,
    _noise_type: PhantomData<G>,
    _arch: PhantomData<A>,
}

params_noise_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, C: Combiner, G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, C, G, A, I]);
params_grid_seed_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, C: Combiner, G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, C, G, A, I]);
params_ridged_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, Ridged, T, A, I]);
params_combiner_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, C: Combiner<Config: Sized>, G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, C, G, A, I]);
params_ping_pong_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, PingPong, T, A, I]);
params_terrace_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, Terrace, T, A, I]);
params_hybrid_multi_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, A: Arch, I: DimIter<A, D>], ['a, D, HybridMulti, T, A, I]);
params_noise_scaling_2d!(OctaveBatchNoiseBuilder, ['a, C: Combiner, G: BatchGenerator<2>, A: Arch, I: DimIter<A, 2>], ['a, 2, C, G, A, I]);
params_noise_scaling_3d!(OctaveBatchNoiseBuilder, ['a, C: Combiner, G: BatchGenerator<3>, A: Arch, I: DimIter<A, 3>], ['a, 3, C, G, A, I]);

impl<F: Combiner, S: BatchGenerator<2>> BatchNoise<2, F, S> {
    /// Creates a new builder using a custom octave list to configure
    /// batches of noise.
    pub fn builder_with_octaves<'a, A, X, Y>(
        octave_list: &'a [Octave<2>],
        x_iter: X,
        y_iter: Y,
    ) -> OctaveBatchNoiseBuilder<'a, 2, F, S, A, Zip<(X, Y)>>
    where
        A: Arch,
        X: Iterator<Item = Simd<f32, A>>,
        Y: Iterator<Item = Simd<f32, A>>,
        Zip<(X, Y)>: DimIter<A, 2>,
    {
        OctaveBatchNoiseBuilder::<'a, 2, F, S, A, _>::new(octave_list, x_iter, y_iter)
    }
}

impl<'a, S, F, A, X, Y> OctaveBatchNoiseBuilder<'a, 2, F, S, A, Zip<(X, Y)>>
where
    S: BatchGenerator<2>,
    F: Combiner,
    A: Arch,
    X: Iterator<Item = Simd<f32, A>>,
    Y: Iterator<Item = Simd<f32, A>>,
    Zip<(X, Y)>: DimIter<A, 2>,
{
    pub fn new(octave_list: &'a [Octave<2>], x_iter: X, y_iter: Y) -> Self {
        Self {
            noise_config: Default::default(),
            combiner_config: Default::default(),
            octave_list,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
            _arch: PhantomData::<A>,
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
            _arch: PhantomData::<A>,
        }
    }
}

impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
    /// Creates a new builder using a custom octave list to configure
    /// batches of noise.
    pub fn builder_with_octaves<'a, A, X, Y, Z>(
        octave_list: &'a [Octave<3>],
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> OctaveBatchNoiseBuilder<'a, 3, F, S, A, Zip<(X, Y, Z)>>
    where
        A: Arch, 
        X: Iterator<Item = Simd<f32, A>>,
        Y: Iterator<Item = Simd<f32, A>>,
        Z: Iterator<Item = Simd<f32, A>>,
        Zip<(X, Y, Z)>: DimIter<A, 3>,
    {
        OctaveBatchNoiseBuilder::<3, F, S, A, _>::new(octave_list, x_iter, y_iter, z_iter)
    }
}

impl<'a, S, F, A, X, Y, Z> OctaveBatchNoiseBuilder<'a, 3, F, S, A, Zip<(X, Y, Z)>>
where
    S: BatchGenerator<3>,
    F: Combiner,
    A: Arch, 
    X: Iterator<Item = Simd<f32, A>>,
    Y: Iterator<Item = Simd<f32, A>>,
    Z: Iterator<Item = Simd<f32, A>>,
    Zip<(X, Y, Z)>: DimIter<A, 3>,
{
    pub fn new(octave_list: &'a [Octave<3>], x_iter: X, y_iter: Y, z_iter: Z) -> Self {
        Self {
            noise_config: Default::default(),
            combiner_config: Default::default(),
            octave_list,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
            _arch: PhantomData::<A>,
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
            _arch: PhantomData::<A>,
        }
    }
}

impl<'a, const D: usize, C: Combiner, G: BatchGenerator<D>, A: Arch, I: DimIter<A, D>>
    OctaveBatchNoiseBuilder<'a, D, C, G, A, I>
{
    declare_fill!(self, output, {
        if self.noise_config.initialize {
            for (i, x) in self.into_iter().enumerate() {
                x.copy_to_slice(&mut output[i * Simd::<f32, A>::LANES..]);
            }
        } else {
            for (i, x) in self.into_iter().enumerate() {
                let index = i * Simd::<f32, A>::LANES;
                let cur = Simd::from_slice(&output[index..]);
                let x = cur + x;
                x.copy_to_slice(&mut output[i..]);
            }
        }
    });

    declare_build!(self, { self.into_iter().collect() });

    declare_into_iter!(A, self, {
        BatchNoise::<D, C, G>::sample_with_octaves(
            self.noise_config,
            self.combiner_config,
            self.octave_list,
            self.iters,
        )
    });
}
