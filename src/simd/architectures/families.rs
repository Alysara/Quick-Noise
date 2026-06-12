use crate::simd::architectures::arch_impl::SimdFamily;

#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::intrinsics::avx2::Avx2Reg;
#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::intrinsics::avx512::{Avx512Reg, Avx512Mask};
#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::intrinsics::sse::SseReg;
#[cfg(target_arch = "aarch64")]
use crate::simd::architectures::intrinsics::neon::NeonReg;

use crate::simd::architectures::intrinsics::scalar::{ScalarReg, ScalarMask};
use std::fmt::Debug;

#[derive(Copy, Clone)]
#[cfg(target_arch = "x86_64")]
pub struct Sse;
#[cfg(target_arch = "x86_64")]
impl SimdFamily for Sse {
    const SIMD_WIDTH: usize = 16;
    type Vec = SseReg;
    type Mask = SseReg;
    type ScalarFamily = Scalar128;

    type Array64<T: Debug + Copy> = [T; 2];
    type Array32<T: Debug + Copy> = [T; 4];
    type Array16<T: Debug + Copy> = [T; 8];
    type Array8<T: Debug + Copy> = [T; 16];
}

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone)]
pub struct Avx2;
#[cfg(target_arch = "x86_64")]
impl SimdFamily for Avx2 {
    const SIMD_WIDTH: usize = 32;
    type Vec = Avx2Reg;
    type Mask = Avx2Reg;
    type ScalarFamily = Scalar256;

    type Array64<T: Debug + Copy> = [T; 4];
    type Array32<T: Debug + Copy> = [T; 8];
    type Array16<T: Debug + Copy> = [T; 16];
    type Array8<T: Debug + Copy> = [T; 32];
}

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone)]
pub struct Avx512;
#[cfg(target_arch = "x86_64")]
impl SimdFamily for Avx512 {
    const SIMD_WIDTH: usize = 64;
    type Vec = Avx512Reg;
    type Mask = Avx512Mask;
    type ScalarFamily = Scalar512;

    type Array64<T: Debug + Copy> = [T; 8];
    type Array32<T: Debug + Copy> = [T; 16];
    type Array16<T: Debug + Copy> = [T; 32];
    type Array8<T: Debug + Copy> = [T; 64];
}

#[cfg(target_arch = "aarch64")]
#[derive(Copy, Clone)]
pub struct Neon;
#[cfg(target_arch = "aarch64")]
impl SimdFamily for Neon {
    const SIMD_WIDTH: usize = 16;
    type Vec = NeonReg;
    type Mask = NeonReg;
    type ScalarFamily = Scalar128;

    type Array64<T: Debug + Copy> = [T; 2];
    type Array32<T: Debug + Copy> = [T; 4];
    type Array16<T: Debug + Copy> = [T; 8];
    type Array8<T: Debug + Copy> = [T; 16];
}

#[derive(Copy, Clone)]
pub struct Scalar128;
impl SimdFamily for Scalar128 {
    const SIMD_WIDTH: usize = 16;
    type Vec = ScalarReg<16>;
    type Mask = ScalarMask<16>;
    type ScalarFamily = Self;

    type Array64<T: Debug + Copy> = [T; 2];
    type Array32<T: Debug + Copy> = [T; 4];
    type Array16<T: Debug + Copy> = [T; 8];
    type Array8<T: Debug + Copy> = [T; 16];
}

#[derive(Copy, Clone)]
pub struct Scalar256;
impl SimdFamily for Scalar256 {
    const SIMD_WIDTH: usize = 32;
    type Vec = ScalarReg<32>;
    type Mask = ScalarMask<32>;
    type ScalarFamily = Self;

    type Array64<T: Debug + Copy> = [T; 4];
    type Array32<T: Debug + Copy> = [T; 8];
    type Array16<T: Debug + Copy> = [T; 16];
    type Array8<T: Debug + Copy> = [T; 32];
}

#[derive(Copy, Clone)]
pub struct Scalar512;
impl SimdFamily for Scalar512 {
    const SIMD_WIDTH: usize = 64;
    type Vec = ScalarReg<64>;
    type Mask = ScalarMask<64>;
    type ScalarFamily = Self;

    type Array64<T: Debug + Copy> = [T; 8];
    type Array32<T: Debug + Copy> = [T; 16];
    type Array16<T: Debug + Copy> = [T; 32];
    type Array8<T: Debug + Copy> = [T; 64];
}
