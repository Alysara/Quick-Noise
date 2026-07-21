use std::sync::LazyLock;

#[cfg(target_arch = "aarch64")]
pub use crate::simd::architectures::arch::Neon;
pub use crate::simd::architectures::arch::Scalar128;
#[cfg(target_arch = "x86_64")]
pub use crate::simd::architectures::arch::{Avx2, Avx512, Sse};
pub use crate::simd::architectures::interface::Arch;

pub static DETECTED_ARCH: LazyLock<Architecture> = LazyLock::new(detect_architecture);

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
    ($func:ident($($args:expr),*$(,)?)) => {
        match *$crate::simd::dispatch::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Sse => $func::<$crate::simd::dispatch::Sse>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx2 => $func::<$crate::simd::dispatch::Avx2>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx512 => $func::<$crate::simd::dispatch::Avx512>($($args),*),
            #[cfg(target_arch = "aarch64")]
            $crate::simd::dispatch::Architecture::Neon => $func::<$crate::simd::dispatch::Neon>($($args),*),
            $crate::simd::dispatch::Architecture::Scalar => $func::<$crate::simd::dispatch::Scalar128>($($args),*)
        }
    };

    ($func:ident::<$($generics:ident),+$(,)?>($($args:expr),*$(,)?)) => {
        match *$crate::simd::dispatch::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Sse => $func::<$crate::simd::dispatch::Sse, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx2 => $func::<$crate::simd::dispatch::Avx2, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx512 => $func::<$crate::simd::dispatch::Avx512, $($generics),+>($($args),*),
            #[cfg(target_arch = "aarch64")]
            $crate::simd::dispatch::Architecture::Neon => $func::<$crate::simd::dispatch::Neon, $($generics),+>($($args),*),
            $crate::simd::dispatch::Architecture::Scalar => $func::<$crate::simd::dispatch::Scalar128, $($generics),+>($($args),*)
        }
    };
}
pub use dispatch;

#[macro_export]
macro_rules! dispatch_async {
    ($func:ident($($args:expr),*$(,)?)) => {
        match *$crate::simd::dispatch::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Sse => $func::<$crate::simd::dispatch::Sse>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx2 => $func::<$crate::simd::dispatch::Avx2>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx512 => $func::<$crate::simd::dispatch::Avx512>($($args),*).await,
            #[cfg(target_arch = "aarch64")]
            $crate::simd::dispatch::Architecture::Neon => $func::<$crate::simd::dispatch::Neon>($($args),*).await,
            $crate::simd::dispatch::Architecture::Scalar => $func::<$crate::simd::dispatch::Scalar128>($($args),*).await
        }
    };

    ($func:ident::<$($generics:ident),+$(,)?>($($args:expr),*$(,)?)) => {
        match *$crate::simd::dispatch::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Sse => $func::<$crate::simd::dispatch::Sse, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx2 => $func::<$crate::simd::dispatch::Avx2, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::dispatch::Architecture::Avx512 => $func::<$crate::simd::dispatch::Avx512, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "aarch64")]
            $crate::simd::dispatch::Architecture::Neon => $func::<$crate::simd::dispatch::Neon, $($generics),+>($($args),*).await,
            $crate::simd::dispatch::Architecture::Scalar => $func::<$crate::simd::dispatch::Scalar128, $($generics),+>($($args),*).await
        }
    };
}
pub use dispatch_async;

// #[macro_export]
// macro_rules! dispatch_fn {
//     ($($prefix:tt )* $vis:vis fn $name:ident($($arg:ident : $ty:ty),*) $(-> $ret:ty)? { $($body:tt)* }) => {
//         $($prefix )* $vis:vis fn $name($($arg: $ty),*) $(-> $ret)? {
//             $($prefix )* fn internal<A: $crate::simd::architectures::interface::Arch>($($arg: $ty),*) $(-> $ret)? {
//                 $($body)*
//             }
//             dispatch!(detect_architecture(), internal($($arg),*))
//         }
//     };
// }
// pub use dispatch_fn;

// #[dispatch_arch(A)]
// pub fn simd_work(arg1, arg2) {
//     simd_function::<A>(arg1, arg2);
// }
//
// ->
//
// pub fn simd_work() {
//     {
//         fn simd_work_internal<A: Arch>() {
//             simd_function::<A>();
//         }
//
//         dispatch!(simd_work_internal());
//     }
// }
