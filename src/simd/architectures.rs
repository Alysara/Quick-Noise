#![allow(
    clippy::missing_transmute_annotations,
    unused_unsafe,
    clippy::useless_transmute,
    clippy::macro_metavars_in_unsafe
)]

pub mod intrinsics {
    #[cfg(target_arch = "x86_64")]
    pub mod avx2;
    #[cfg(target_arch = "x86_64")]
    pub mod avx512;
    #[cfg(target_arch = "x86_64")]
    pub mod sse;

    #[cfg(target_arch = "aarch64")]
    pub mod neon;

    pub mod scalar;
}

#[macro_use]
pub mod macros;
pub mod arch;
pub mod interface;

