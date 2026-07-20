#[cfg(target_arch = "aarch64")]
pub use crate::simd::architectures::arch::Neon;
pub use crate::simd::architectures::arch::Scalar128;
#[cfg(target_arch = "x86_64")]
pub use crate::simd::architectures::arch::{Avx2, Avx512, Sse};
pub use crate::simd::architectures::interface::Arch;

pub enum Architecture {
    #[cfg(target_arch = "x86_64")]
    Sse,
    #[cfg(target_arch = "x86_64")]
    Avx2,
    #[cfg(target_arch = "x86_64")]
    Avx512,
    #[cfg(target_arch = "aarch64")]
    Neon,
    Scalar,
}

pub fn detect_architecture() -> Architecture {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("fma") {
            if is_x86_feature_detected!("avx512f") {
                return Architecture::Avx512;
            } else if is_x86_feature_detected!("avx2") {
                return Architecture::Avx2;
            }
        }

        if is_x86_feature_detected!("sse4.2") {
            return Architecture::Sse;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        use std::arch::is_aarch64_feature_detected;

        if is_aarch64_feature_detected!("neon") {
            return Architecture::Neon;
        }
    }

    Architecture::Scalar
}

#[macro_export]
macro_rules! dispatch {
    ($enum:ident, $func:ident($($args:expr),*)) => {
        match $enum {
            #[cfg(target_arch = "x86_64")]
            Architecture::Sse => $func::<Sse>($($args),*),
            #[cfg(target_arch = "x86_64")]
            Architecture::Avx2 => $func::<Avx2>($($args),*),
            #[cfg(target_arch = "x86_64")]
            Architecture::Avx512 => $func::<Avx512>($($args),*),
            #[cfg(target_arch = "aarch64")]
            Architecture::Neon => $func::<Neon>($($args),*),
            Architecture::Scalar => $func::<Scalar128>($($args),*)
        }
    };

    ($enum:ident, $func:ident::<$($generics:ident),+>($($args:expr),*)) => {
        match $enum {
            #[cfg(target_arch = "x86_64")]
            Architecture::Sse => $func::<Sse, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            Architecture::Avx2 => $func::<Avx2, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            Architecture::Avx512 => $func::<Avx512, $($generics),+>($($args),*),
            #[cfg(target_arch = "aarch64")]
            Architecture::Neon => $func::<Neon, $($generics),+>($($args),*),
            Architecture::Scalar => $func::<Scalar128, $($generics),+>($($args),*)
        }
    };
}
pub use dispatch;
