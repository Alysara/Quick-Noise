// #![feature(const_cmp, const_trait_impl, generic_const_exprs, associated_type_defaults)]
// #![feature(portable_simd)]
#![feature(trace_macros)]

pub mod simd;

pub mod math {
    pub(crate) mod random;
    pub(crate) mod vec;
    pub use vec::{Vec2, Vec3};
}

pub mod testing {
    pub mod profiler;
}

pub mod emit {
    pub mod grayscale;
}

mod api;
pub use api::batch::interface::{Batch2D, Batch3D, BatchNoise};
pub use api::defaults::*;
pub use api::grid::interface::{Grid2D, Grid3D, GridNoise};
pub use api::methods::{Dim2, Dim3, Octave2D, Octave3D};
pub use noise::cellular::Cellular;
pub use noise::perlin::Perlin;
pub use noise::simplex::Simplex;
pub use noise::value::Value;

mod noise;
pub use noise::*;
