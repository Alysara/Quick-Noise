use crate::simd::Arch;
use crate::simd::mask::Mask;
use crate::simd::register::Simd;

// Static dispatch for identifying lane sizes and number of simd registers.
cfg_select! {
    all(target_arch = "x86_64", target_feature = "avx512f", target_feature = "fma") => {
        use crate::simd::architectures::families::Avx512;
        pub type StaticSimd<T> = Simd<T, Avx512>;
        pub type StaticMask<T> = Mask<T, Avx512>;
        pub type StaticArch = Avx512;
    }
    all(target_arch = "x86_64", target_feature = "avx2", target_feature = "fma") => {
        use crate::simd::architectures::families::Avx2;
        pub type StaticSimd<T> = Simd<T, Avx2>;
        pub type StaticMask<T> = Mask<T, Avx2>;
        pub type StaticArch = Avx2;
    }
    all(target_arch = "x86_64", target_feature = "sse4.2") => {
        use crate::simd::architectures::families::Sse;
        pub type StaticSimd<T> = Simd<T, Sse>;
        pub type StaticMask<T> = Mask<T, Sse>;
        pub type StaticArch = Sse;
    }
    all(target_arch = "aarch64", target_feature = "neon") => {
        use crate::simd::architectures::families::Neon;
        pub type StaticSimd<T> = Simd<T, Neon>;
        pub type StaticMask<T> = Mask<T, Neon>;
        pub type StaticArch = Neon;
    }
    _ => {
        use crate::simd::architectures::arch::Scalar128;
        pub type StaticSimd<T> = Simd<T, Scalar128>;
        pub type StaticMask<T> = Mask<T, Scalar128>;
        pub type StaticArch = Scalar128;
    }
}

pub type ScalarArch = <StaticArch as Arch>::ScalarArch;
pub type ScalarSimd<T> = Simd<T, ScalarArch>;
pub type ScalarMask<T> = Mask<T, ScalarArch>;
