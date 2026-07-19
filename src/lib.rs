#![doc = include_str!("../README.md")]

//! Maximum performance SIMD-accelerated procedural noise library
//! with up to 10x+ performance on uniform grids. Works on stable Rust.

mod api;
pub mod math;
mod noise;
pub mod simd;

#[cfg(feature = "image")]
pub mod emit {
    mod grayscale;
    pub use grayscale::NoiseImageExt;
}

pub use api::batch::interface::{BatchGenerator, BatchNoise};
pub use api::defaults::*;
pub use api::grid::interface::{Grid, GridGenerator, GridNoise, GridNoiseParams};
pub use api::octave::Octave;
pub use api::{
    BatchNoiseBuilder, GridNoiseBuilder, OctaveBatchNoiseBuilder, OctaveGridNoiseBuilder,
};
pub use noise::combiners::{
    Billow, Combiner, CombinerArray, CombinerState, Fbm, HybridMulti, Multi, PingPong, Ridged,
    Terrace,
};
pub use noise::generators::{Cellular, Perlin, Simplex, Value};
pub use noise::*;
