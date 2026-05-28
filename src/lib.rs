// #![feature(const_cmp, const_trait_impl, generic_const_exprs, associated_type_defaults)]
// #![feature(portable_simd)]
#![feature(trace_macros)]

pub mod simd;

pub mod math {
    pub mod random;
    pub mod vec;
}

pub mod testing {
    pub mod profiler;
}

pub mod emit {
    pub mod grayscale;
}

mod api;
pub use api::parameters::*;
pub use api::defaults::*;
pub use api::grid::fbm::{PerlinGrid2D, PerlinGrid3D};
pub use api::grid::custom::{CustomPerlinGrid2D, CustomPerlinGrid3D};
pub use api::grid::interface::{Grid2D, Grid3D};
pub use api::batch::interface::{Batch2D, Batch3D};

mod noise;
pub use noise::*;
