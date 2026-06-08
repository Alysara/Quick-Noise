use crate::api::methods::{Dim2, Dim3, NoiseDimension};
use crate::math::random::Random;
use crate::math::vec::{BasicVec, FloatVec, Vec2, Vec3};

pub trait OctaveSeed: NoiseDimension {
    fn octave_seed(vec: Self::FVec, seed: u64) -> u32;
}

impl OctaveSeed for Dim2 {
    fn octave_seed(vec: Vec2<f32>, seed: u64) -> u32 {
        Random::static_mix_u64_pair(
            seed.wrapping_mul(vec.x.to_bits() as u64),
            seed.wrapping_mul(vec.y.to_bits() as u64),
        ) as u32
    }
}

impl OctaveSeed for Dim3 {
    fn octave_seed(vec: Vec3<f32>, seed: u64) -> u32 {
        Random::mix_u64_triple(
            seed.wrapping_mul(vec.x.to_bits() as u64),
            seed.wrapping_mul(vec.y.to_bits() as u64),
            seed.wrapping_mul(vec.z.to_bits() as u64),
        ) as u32
    }
}
