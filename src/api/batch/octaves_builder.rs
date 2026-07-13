use std::marker::PhantomData;

use itertools::{Zip, multizip};

use crate::{BatchGenerator, Ridged};
use crate::api::batch::interface::{BatchNoise, DimIter};
use crate::api::configs::*;
use crate::api::octave::Octave;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::noise::combiners::Combiner;
use crate::simd::arch_simd::ArchSimd;

pub struct OctaveBatchNoiseBuilder<
    'a,
    const D: usize,
    F: Combiner,
    S: BatchGenerator<D>,
    I: DimIter<D>,
> {
    noise_config: OctaveNoiseConfig<D>,
    fractal_config: F::Config,
    octave_list: &'a [Octave<D>],
    iters: I,
    _noise_type: PhantomData<S>,
}

params_noise_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>], ['a, D, F, S, I]);
params_grid_seed_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>], ['a, D, F, S, I]);
params_ridged_builder!(OctaveBatchNoiseBuilder, ['a, const D: usize, T: BatchGenerator<D>, I: DimIter<D>], ['a, D, Ridged, T, I]);
params_noise_scaling_2d!(OctaveBatchNoiseBuilder, ['a, F: Combiner, S: BatchGenerator<2>, I: DimIter<2>], ['a, 2, F, S, I]);
params_noise_scaling_3d!(OctaveBatchNoiseBuilder, ['a, F: Combiner, S: BatchGenerator<3>, I: DimIter<3>], ['a, 3, F, S, I]);

impl<F: Combiner, S: BatchGenerator<2>> BatchNoise<2, F, S> {
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
            fractal_config: Default::default(),
            octave_list,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: OctaveNoiseConfig<2>,
        fractal_config: F::Config,
        octave_list: &'a [Octave<2>],
        x_iter: X,
        y_iter: Y,
    ) -> Self {
        Self {
            noise_config,
            fractal_config,
            octave_list,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
        }
    }
}

impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
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
        Zip<(X, Y)>: DimIter<3>,
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
            fractal_config: Default::default(),
            octave_list,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: OctaveNoiseConfig<3>,
        fractal_config: F::Config,
        octave_list: &'a [Octave<3>],
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> Self {
        Self {
            noise_config,
            fractal_config,
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
        let mut i = 0;
        self.into_iter().for_each(|x| {
            x.copy_to_slice(&mut output[i..]);
            i += ArchSimd::<f32>::LANES;
        });
    });

    declare_fill_onto!(self, output, {
        let mut i = 0;
        self.into_iter().for_each(|x| {
            let cur = ArchSimd::from_slice(&output[i..]) + x;
            cur.copy_to_slice(&mut output[i..]);
            i += ArchSimd::<f32>::LANES;
        });
    });

    // declare_build!(self, { self.into_iter().collect() });

    declare_into_iter!(self, {
        BatchNoise::<D, F, S>::sample_with_octaves(
            self.noise_config,
            self.fractal_config,
            self.octave_list,
            self.iters,
        )
    });
}
