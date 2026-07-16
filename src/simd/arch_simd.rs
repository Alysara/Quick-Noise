use crate::simd::architectures::interface::SimdFamily;
use crate::simd::mask::Mask;
use crate::simd::register::Simd;

// Static dispatch for identifying lane sizes and number of simd registers.
cfg_select! {
    all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "fma") => {
        use crate::simd::architectures::families::Avx512;
        pub const SIMD_WIDTH: usize = 64;
        pub const NUM_SIMD_REG: usize = 32;
        pub type ArchSimd<T> = Simd<T, Avx512>;
        pub type ArchMask<T> = Mask<T, Avx512>;
        pub type ArchFamily = Avx512;
    }
    all(target_arch = "x86_64", target_feature = "avx2", target_feature = "fma") => {
        use crate::simd::architectures::families::Avx2;
        pub const SIMD_WIDTH: usize = 32;
        pub const NUM_SIMD_REG: usize = 16;
        pub type ArchSimd<T> = Simd<T, Avx2>;
        pub type ArchMask<T> = Mask<T, Avx2>;
        pub type ArchFamily = Avx2;
    }
    all(target_arch = "x86_64", target_feature = "sse4.2") => {
        use crate::simd::architectures::families::Sse;
        pub const SIMD_WIDTH: usize = 16;
        pub const NUM_SIMD_REG: usize = 16;
        pub type ArchSimd<T> = Simd<T, Sse>;
        pub type ArchMask<T> = Mask<T, Sse>;
        pub type ArchFamily = Sse;
    }
    all(target_arch = "aarch64", target_feature = "neon") => {
        use crate::simd::architectures::families::Neon;
        pub const SIMD_WIDTH: usize = 16;
        pub const NUM_SIMD_REG: usize = 32;
        pub type ArchSimd<T> = Simd<T, Neon>;
        pub type ArchMask<T> = Mas<T, Neon>;
        pub type ArchFamily = Neon;
    }
    _ => {
        use crate::simd::architectures::families::Scalar128;
        pub const SIMD_WIDTH: usize = 4;
        pub const NUM_SIMD_REG: usize = 8;
        pub type ArchSimd<T> = Simd<T, Scalar128>;
        pub type ArchMask<T> = Mask<T, Scalar128>;
        pub type ArchFamily = Scalar128;
    }
}

pub type ScalarFamily = <ArchFamily as SimdFamily>::ScalarFamily;
pub type ScalarSimd<T> = Simd<T, ScalarFamily>;
pub type ScalarMask<T> = Mask<T, ScalarFamily>;
