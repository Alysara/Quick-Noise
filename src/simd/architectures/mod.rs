pub mod intrinsics {
    #[cfg(target_arch = "x86_64")]
    pub mod avx2;
    #[cfg(target_arch = "x86_64")]
    pub mod sse;
    #[cfg(target_arch = "x86_64")]
    pub mod avx512;

    #[cfg(target_arch = "aarch64")]
    pub mod neon;
}

#[macro_use]
pub mod macros;
pub mod families;
pub mod arch_impl;