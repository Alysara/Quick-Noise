use crate::api::grid::interface::GridNoise;
use crate::math::vec::{Vec2, Vec3};
use crate::value::grid_2d::ValueGridNoise2D;
use crate::value::grid_3d::ValueGridNoise3D;
use crate::simd::simd_array::SimdArray;

#[derive(Default, Copy, Clone)]
pub struct Value {}

impl GridNoise for Value {
    fn grid_2d<const X: usize, const Y: usize, const N: usize, const INITIALIZE: bool>(
        seed: u32,
        result: &mut SimdArray<f32, N>,
        position: Vec2<i32>,
        frequency: Vec2<f32>,
        weight: f32,
        magnification: f32,
        tiling: Vec2<Option<u32>>,
    ) {
        ValueGridNoise2D::<X, Y, N>::grid_2d::<INITIALIZE>(
            seed,
            result,
            position,
            frequency,
            weight,
            magnification,
            tiling,
        );
    }

    fn grid_3d<
        const X: usize,
        const Y: usize,
        const Z: usize,
        const N: usize,
        const INITIALIZE: bool,
    >(
        seed: u32,
        result: &mut SimdArray<f32, N>,
        position: Vec3<i32>,
        frequency: Vec3<f32>,
        weight: f32,
        magnification: f32,
        tiling: Vec3<Option<u32>>,
    ) {
        ValueGridNoise3D::<X, Y, Z, N>::grid_3d::<INITIALIZE>(
            seed,
            result,
            position,
            frequency,
            weight,
            magnification,
            tiling,
        );
    }
}
