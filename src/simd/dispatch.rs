use std::sync::LazyLock;

#[cfg(target_arch = "aarch64")]
pub use crate::simd::architectures::arch::Neon;
pub use crate::simd::architectures::arch::Scalar128;
#[cfg(target_arch = "x86_64")]
pub use crate::simd::architectures::arch::{Avx2, Avx512, Sse};

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
    Scalar128,
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

    Architecture::Scalar128
}

#[macro_export]
macro_rules! dispatch {
    ($func:ident($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => $func::<$crate::simd::Sse>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => $func::<$crate::simd::Avx2>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => $func::<$crate::simd::Avx512>($($args),*),
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => $func::<$crate::simd::Neon>($($args),*),
            $crate::simd::Architecture::Scalar => $func::<$crate::simd::Scalar128>($($args),*)
        }
    };

    ($func:ident::<$($generics:ident),+$(,)?>($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => $func::<$crate::simd::Sse, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => $func::<$crate::simd::Avx2, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => $func::<$crate::simd::Avx512, $($generics),+>($($args),*),
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => $func::<$crate::simd::Neon, $($generics),+>($($args),*),
            $crate::simd::Architecture::Scalar => $func::<$crate::simd::Scalar128, $($generics),+>($($args),*)
        }
    };

    (Self::$func:ident($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => Self::$func::<$crate::simd::Sse>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => Self::$func::<$crate::simd::Avx2>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => Self::$func::<$crate::simd::Avx512>($($args),*),
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => Self::$func::<$crate::simd::Neon>($($args),*),
            $crate::simd::Architecture::Scalar => Self::$func::<$crate::simd::Scalar128>($($args),*)
        }
    };

    (Self::$func:ident::<$($generics:ident),+$(,)?>($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => Self::$func::<$crate::simd::Sse, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => Self::$func::<$crate::simd::Avx2, $($generics),+>($($args),*),
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => Self::$func::<$crate::simd::Avx512, $($generics),+>($($args),*),
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => Self::$func::<$crate::simd::Neon, $($generics),+>($($args),*),
            $crate::simd::Architecture::Scalar => Self::$func::<$crate::simd::Scalar128, $($generics),+>($($args),*)
        }
    };
}
pub use dispatch;

#[macro_export]
macro_rules! dispatch_async {
    ($func:ident($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => $func::<$crate::simd::Sse>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => $func::<$crate::simd::Avx2>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => $func::<$crate::simd::Avx512>($($args),*).await,
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => $func::<$crate::simd::Neon>($($args),*).await,
            $crate::simd::Architecture::Scalar => $func::<$crate::simd::Scalar128>($($args),*).await
        }
    };

    ($func:ident::<$($generics:ident),+$(,)?>($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => $func::<$crate::simd::Sse, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => $func::<$crate::simd::Avx2, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => $func::<$crate::simd::Avx512, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => $func::<$crate::simd::Neon, $($generics),+>($($args),*).await,
            $crate::simd::Architecture::Scalar => $func::<$crate::simd::Scalar128, $($generics),+>($($args),*).await
        }
    };

    (Self::$func:ident($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => Self::$func::<$crate::simd::Sse>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => Self::$func::<$crate::simd::Avx2>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => Self::$func::<$crate::simd::Avx512>($($args),*).await,
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => Self::$func::<$crate::simd::Neon>($($args),*).await,
            $crate::simd::Architecture::Scalar => Self::$func::<$crate::simd::Scalar128>($($args),*).await
        }
    };

    (Self::$func:ident::<$($generics:ident),+$(,)?>($($args:expr),*$(,)?)) => {
        match *$crate::simd::DETECTED_ARCH {
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Sse => Self::$func::<$crate::simd::Sse, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx2 => Self::$func::<$crate::simd::Avx2, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "x86_64")]
            $crate::simd::Architecture::Avx512 => Self::$func::<$crate::simd::Avx512, $($generics),+>($($args),*).await,
            #[cfg(target_arch = "aarch64")]
            $crate::simd::Architecture::Neon => Self::$func::<$crate::simd::Neon, $($generics),+>($($args),*).await,
            $crate::simd::Architecture::Scalar => Self::$func::<$crate::simd::Scalar128, $($generics),+>($($args),*).await
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
