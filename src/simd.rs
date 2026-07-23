pub mod architectures;
pub mod array_trait;
mod dispatch;
pub mod mask;
pub mod register;
mod static_simd;
pub mod traits;

pub use architectures::interface::Arch;
pub use dispatch::*;
pub use mask::Mask;
pub use quick_noise_macros::{dispatch_simd, enable_targets};
pub use register::Simd;
pub use register::iters::SimdSliceIterExt;
pub use static_simd::*;
pub use traits::{SimdElement, SimdFloat, SimdInteger};
// pub use dispatch::{Architecture, detect_architecture, dispatch};
