pub mod arch_simd;
// pub mod simd_vec;
// pub mod avx2;
pub mod architectures;
pub mod array_trait;
pub mod mask;
pub mod register;
// pub mod simd_traits;
pub mod traits;

pub use register::iters::SimdSliceIterExt;
