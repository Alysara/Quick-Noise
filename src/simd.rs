pub mod arch_simd;
pub mod architectures;
pub mod array_trait;
pub mod mask;
pub mod register;
pub mod traits;

pub use arch_simd::{ArchSimd, ArchMask, ArchFamily, NUM_SIMD_REG};
pub use register::iters::SimdSliceIterExt;
