// #![feature(const_cmp, const_trait_impl, generic_const_exprs, associated_type_defaults)]
// #![feature(portable_simd)]
// #![feature(trace_macros)]

pub mod simd;

pub mod math {
    pub(crate) mod random;
    pub(crate) mod vec;
    pub use vec::{Vec2, Vec3};
}

pub mod testing {
    pub mod profiler;
}

#[cfg(feature = "image")]
pub mod emit {
    pub mod grayscale;
}

mod api;
pub use api::batch::interface::BatchNoise;
pub use api::batch::fbm::BatchNoiseBuilder;
pub use api::defaults::*;
pub use api::grid::interface::{GridBuilder, GridNoise};
pub use api::grid::fbm::GridNoiseBuilder;
pub use api::methods::Octave;
pub use noise::cellular::Cellular;
pub use noise::perlin::Perlin;
pub use noise::simplex::Simplex;
pub use noise::value::Value;
pub use noise::fractal::{Fractal, FractalState, FractalArray, Fbm, Billow, Ridged};

mod noise;
pub use noise::*;
