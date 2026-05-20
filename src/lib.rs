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

pub mod api {
    pub mod builders;
    pub mod batch;
    pub mod grid;
}

mod noise;
pub use noise::*;
