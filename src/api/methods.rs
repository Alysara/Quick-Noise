use crate::api::batch::interface::BatchNoise;
use crate::cellular::Cellular;
use crate::perlin::Perlin;
use crate::simd::arch_simd::ArchSimd;
use crate::simplex::Simplex;
use crate::value::Value;

pub enum NoiseMethod {
    Perlin = 0,
    Value = 1,
    Simplex = 2,
    Cellular = 3,
}

/// Necessary converter due to limitation of types for const generics.
impl NoiseMethod {
    pub const PERLIN_U8: u8 = NoiseMethod::Perlin as u8;
    pub const VALUE_U8: u8 = NoiseMethod::Value as u8;
    pub const SIMPLEX_U8: u8 = NoiseMethod::Simplex as u8;
    pub const CELLULAR_U8: u8 = NoiseMethod::Cellular as u8;

    #[inline(always)]
    pub const fn from_u8_const(val: u8) -> Self {
        match val {
            0 => NoiseMethod::Perlin,
            1 => NoiseMethod::Value,
            2 => NoiseMethod::Simplex,
            3 => NoiseMethod::Cellular,
            _ => panic!("Invalid NoiseMethod enum value!"),
        }
    }

    #[inline(always)]
    pub fn batch_2d(
        &self,
        seed: u32,
        x: ArchSimd<f32>,
        y: ArchSimd<f32>,
        x_freq: ArchSimd<f32>,
        y_freq: ArchSimd<f32>,
    ) -> ArchSimd<f32> {
        match &self {
            NoiseMethod::Perlin => Perlin::batch_2d(seed, x, y, x_freq, y_freq),
            NoiseMethod::Value => Value::batch_2d(seed, x, y, x_freq, y_freq),
            NoiseMethod::Simplex => Simplex::batch_2d(seed, x, y, x_freq, y_freq),
            NoiseMethod::Cellular => Cellular::batch_2d(seed, x, y, x_freq, y_freq),
        }
    }

    #[inline(always)]
    pub fn batch_3d(
        &self,
        seed: u32,
        x: ArchSimd<f32>,
        y: ArchSimd<f32>,
        z: ArchSimd<f32>,
        x_freq: ArchSimd<f32>,
        y_freq: ArchSimd<f32>,
        z_freq: ArchSimd<f32>,
    ) -> ArchSimd<f32> {
        match &self {
            NoiseMethod::Perlin => Perlin::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
            NoiseMethod::Value => Value::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
            NoiseMethod::Simplex => Simplex::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
            NoiseMethod::Cellular => Cellular::batch_3d(seed, x, y, z, x_freq, y_freq, z_freq),
        }
    }
}
