use crate::math::random::Random;
use crate::math::vec::{Vec2, Vec3, VecN};

pub trait OctaveSeed: VecN<f32> {
    fn octave_seed(&self, seed: u64) -> u32;
}

impl OctaveSeed for Vec2<f32> {
    fn octave_seed(&self, seed: u64) -> u32 {
        Random::static_mix_u64_pair(
            seed.wrapping_mul(self.x.to_bits() as u64),
            seed.wrapping_mul(self.y.to_bits() as u64),
        ) as u32
    }
}

impl OctaveSeed for Vec3<f32> {
    fn octave_seed(&self, seed: u64) -> u32 {
        Random::mix_u64_triple(
            seed.wrapping_mul(self.x.to_bits() as u64),
            seed.wrapping_mul(self.y.to_bits() as u64),
            seed.wrapping_mul(self.z.to_bits() as u64),
        ) as u32
    }
}
