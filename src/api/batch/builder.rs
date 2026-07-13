use std::marker::PhantomData;

use itertools::{Zip, multizip};

use crate::Ridged;
use crate::noise::combiners::Combiner;
use crate::api::batch::interface::{BatchNoise, BatchGenerator, DimIter};
use crate::api::configs::*;
use crate::api::parameters::*;
use crate::math::random::Random;
use crate::simd::arch_simd::ArchSimd;

pub struct BatchNoiseBuilder<const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>> {
    noise_config: NoiseConfig<D>,
    fractal_config: F::Config,
    iters: I,
    _noise_type: PhantomData<S>,
}

params_noise_builder!(BatchNoiseBuilder, [const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>], [D, F, S, I]);
params_lacunarity_builder!(BatchNoiseBuilder, [const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>], [D, F, S, I]);
params_ridged_builder!(BatchNoiseBuilder, [const D: usize, T: BatchGenerator<D>, I: DimIter<D>], [D, Ridged, T, I]);
params_grid_seed_builder!(BatchNoiseBuilder, [const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>], [D, F, S, I]);
params_noise_scaling_2d!(BatchNoiseBuilder, [F: Combiner, S: BatchGenerator<2>, I: DimIter<2>], [2, F, S, I]);
params_noise_scaling_3d!(BatchNoiseBuilder, [F: Combiner, S: BatchGenerator<3>, I: DimIter<3>], [3, F, S, I]);

impl<F: Combiner, S: BatchGenerator<2>> BatchNoise<2, F, S> {
    pub fn builder<X, Y>(x_iter: X, y_iter: Y) -> BatchNoiseBuilder<2, F, S, Zip<(X, Y)>>
    where
        X: Iterator<Item = ArchSimd<f32>>,
        Y: Iterator<Item = ArchSimd<f32>>,
        Zip<(X, Y)>: DimIter<2>,
    {
        BatchNoiseBuilder::<2, F, S, _>::new(x_iter, y_iter)
    }
}

impl<S, F, X, Y> BatchNoiseBuilder<2, F, S, Zip<(X, Y)>>
where
    S: BatchGenerator<2>,
    F: Combiner,
    X: Iterator<Item = ArchSimd<f32>>,
    Y: Iterator<Item = ArchSimd<f32>>,
    Zip<(X, Y)>: DimIter<2>,
{
    pub fn new(x_iter: X, y_iter: Y) -> Self {
        Self {
            noise_config: Default::default(),
            fractal_config: Default::default(),
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: NoiseConfig<2>,
        fractal_config: F::Config,
        x_iter: X,
        y_iter: Y,
    ) -> Self {
        Self {
            noise_config,
            fractal_config,
            iters: multizip((x_iter, y_iter)),
            _noise_type: PhantomData::<S>,
        }
    }
}

impl<F: Combiner, S: BatchGenerator<3>> BatchNoise<3, F, S> {
    pub fn sample_builder<X, Y, Z>(
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> BatchNoiseBuilder<3, F, S, Zip<(X, Y, Z)>>
    where
        X: Iterator<Item = ArchSimd<f32>>,
        Y: Iterator<Item = ArchSimd<f32>>,
        Z: Iterator<Item = ArchSimd<f32>>,
        Zip<(X, Y)>: DimIter<3>,
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
            fractal_config: Default::default(),
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }

    pub fn from_configs(
        noise_config: NoiseConfig<3>,
        fractal_config: F::Config,
        x_iter: X,
        y_iter: Y,
        z_iter: Z,
    ) -> Self {
        Self {
            noise_config,
            fractal_config,
            iters: multizip((x_iter, y_iter, z_iter)),
            _noise_type: PhantomData::<S>,
        }
    }
}

impl<const D: usize, F: Combiner, S: BatchGenerator<D>, I: DimIter<D>> BatchNoiseBuilder<D, F, S, I> {
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
        BatchNoise::<D, F, S>::sample(self.noise_config, self.fractal_config, self.iters)
    });
}
