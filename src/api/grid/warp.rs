use std::iter::zip;
use std::marker::PhantomData;

use itertools::multizip;

use crate::api::batch::interface::DimIter;
use crate::simd::arch_simd::ArchSimd;
use crate::{BatchGenerator, BatchNoiseBuilder, Combiner, Grid};

impl Grid<2> {
    pub fn warp_builder<C: Combiner, G: BatchGenerator<2>>(
        &self,
        warp_strength: f32,
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> BatchNoiseBuilder<2, C, G, impl DimIter<2>> {
        let strength = ArchSimd::splat(warp_strength);
        let x_iter = zip(x_iter, self.x_iter()).map(move |(x, grid)| x.mul_add(strength, grid));
        let y_iter = zip(y_iter, self.y_iter()).map(move |(y, grid)| y.mul_add(strength, grid));

        BatchNoiseBuilder {
            iters: multizip((x_iter, y_iter)),
            noise_config: Default::default(),
            combiner_config: Default::default(),
            _noise_type: PhantomData::<G>,
        }
    }
}

impl Grid<3> {
    pub fn warp_builder<C: Combiner, G: BatchGenerator<3>>(
        &self,
        warp_strength: f32,
        x_iter: impl Iterator<Item = ArchSimd<f32>>,
        y_iter: impl Iterator<Item = ArchSimd<f32>>,
        z_iter: impl Iterator<Item = ArchSimd<f32>>,
    ) -> BatchNoiseBuilder<3, C, G, impl DimIter<3>> {
        let strength = ArchSimd::splat(warp_strength);
        let x_iter = zip(x_iter, self.x_iter()).map(move |(x, grid)| x.mul_add(strength, grid));
        let y_iter = zip(y_iter, self.y_iter()).map(move |(y, grid)| y.mul_add(strength, grid));
        let z_iter = zip(z_iter, self.z_iter()).map(move |(z, grid)| z.mul_add(strength, grid));

        BatchNoiseBuilder {
            iters: multizip((x_iter, y_iter, z_iter)),
            noise_config: Default::default(),
            combiner_config: Default::default(),
            _noise_type: PhantomData::<G>,
        }
    }
}
