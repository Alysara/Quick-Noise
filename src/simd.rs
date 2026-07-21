mod static_simd;
pub mod architectures;
pub mod array_trait;
pub mod mask;
pub mod register;
pub mod traits;
pub mod dispatch;

pub use traits::{SimdElement, SimdFloat, SimdInteger};
pub use architectures::interface::Arch;
pub use static_simd::*;
pub use register::Simd;
pub use mask::Mask;
pub use register::iters::SimdSliceIterExt;
pub use quick_noise_macros::dispatch_simd;
// pub use dispatch::{Architecture, detect_architecture, dispatch};
