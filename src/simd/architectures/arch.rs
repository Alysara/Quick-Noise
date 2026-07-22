#[cfg(target_arch = "x86_64")]
use crate::simd::{Architecture, SimdElement};
use crate::simd::architectures::interface::Arch;

#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::intrinsics::avx2::Avx2Reg;
#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::intrinsics::avx512::{Avx512Reg, Avx512Mask};
#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::intrinsics::sse::SseReg;
#[cfg(target_arch = "aarch64")]
use crate::simd::architectures::intrinsics::neon::NeonReg;

use crate::simd::architectures::intrinsics::scalar::{ScalarReg, ScalarMask};
#[cfg(target_arch = "x86_64")]
use crate::simd::register::Simd;
use std::fmt::Debug;

#[derive(Copy, Clone, Default)]
#[cfg(target_arch = "x86_64")]
pub struct Sse;
#[cfg(target_arch = "x86_64")]
impl Arch for Sse {
    const SIMD_WIDTH: usize = 16;
    const NUM_SIMD_REG: usize = 16;
    const ARCHITECTURE: Architecture = Architecture::Sse;
    type Block2<T: SimdElement> = [Simd<T, Self>; 4];
    type Block4<T: SimdElement> = [Simd<T, Self>; 2];

    type Vec = SseReg;
    type Mask = SseReg;
    type ScalarArch = Scalar128;

    type Array64<T: Debug + Copy> = [T; 2];
    type Array32<T: Debug + Copy> = [T; 4];
    type Array16<T: Debug + Copy> = [T; 8];
    type Array8<T: Debug + Copy> = [T; 16];
}

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone, Default)]
pub struct Avx2;
#[cfg(target_arch = "x86_64")]
impl Arch for Avx2 {
    const SIMD_WIDTH: usize = 32;
    const NUM_SIMD_REG: usize = 16;
    const ARCHITECTURE: Architecture = Architecture::Avx2;
    type Block2<T: SimdElement> = [Simd<T, Self>; 4];
    type Block4<T: SimdElement> = [Simd<T, Self>; 2];

    type Vec = Avx2Reg;
    type Mask = Avx2Reg;
    type ScalarArch = Scalar256;

    type Array64<T: Debug + Copy> = [T; 4];
    type Array32<T: Debug + Copy> = [T; 8];
    type Array16<T: Debug + Copy> = [T; 16];
    type Array8<T: Debug + Copy> = [T; 32];
}

#[cfg(target_arch = "x86_64")]
#[derive(Copy, Clone, Default)]
pub struct Avx512;
#[cfg(target_arch = "x86_64")]
impl Arch for Avx512 {
    const SIMD_WIDTH: usize = 64;
    const NUM_SIMD_REG: usize = 32;
    const ARCHITECTURE: Architecture = Architecture::Avx512;
    type Block2<T: SimdElement> = [Simd<T, Self>; 8];
    type Block4<T: SimdElement> = [Simd<T, Self>; 4];

    type Vec = Avx512Reg;
    type Mask = Avx512Mask;
    type ScalarArch = Scalar512;

    type Array64<T: Debug + Copy> = [T; 8];
    type Array32<T: Debug + Copy> = [T; 16];
    type Array16<T: Debug + Copy> = [T; 32];
    type Array8<T: Debug + Copy> = [T; 64];
}

#[cfg(target_arch = "aarch64")]
#[derive(Copy, Clone, Default)]
pub struct Neon;
#[cfg(target_arch = "aarch64")]
impl Arch for Neon {
    const SIMD_WIDTH: usize = 16;
    const NUM_SIMD_REG: usize = 32;
    const ARCHITECTURE: Architecture = Architecture::Neon;
    type Block2<T: SimdElement> = [Simd<T, Self>; 8];
    type Block4<T: SimdElement> = [Simd<T, Self>; 4];

    type Vec = NeonReg;
    type Mask = NeonReg;
    type ScalarArch = Scalar128;

    type Array64<T: Debug + Copy> = [T; 2];
    type Array32<T: Debug + Copy> = [T; 4];
    type Array16<T: Debug + Copy> = [T; 8];
    type Array8<T: Debug + Copy> = [T; 16];
}

#[derive(Copy, Clone, Default)]
pub struct Scalar128;
impl Arch for Scalar128 {
    const SIMD_WIDTH: usize = 16;
    const NUM_SIMD_REG: usize = 16;
    const ARCHITECTURE: Architecture = Architecture::Scalar128;
    type Block2<T: SimdElement> = [Simd<T, Self>; 4];
    type Block4<T: SimdElement> = [Simd<T, Self>; 2];

    type Vec = ScalarReg<16>;
    type Mask = ScalarMask<16>;
    type ScalarArch = Self;

    type Array64<T: Debug + Copy> = [T; 2];
    type Array32<T: Debug + Copy> = [T; 4];
    type Array16<T: Debug + Copy> = [T; 8];
    type Array8<T: Debug + Copy> = [T; 16];
}

#[derive(Copy, Clone, Default)]
pub struct Scalar256;
impl Arch for Scalar256 {
    const SIMD_WIDTH: usize = 32;
    const NUM_SIMD_REG: usize = 16;
    const ARCHITECTURE: Architecture = Architecture::Scalar128;
    type Block2<T: SimdElement> = [Simd<T, Self>; 4];
    type Block4<T: SimdElement> = [Simd<T, Self>; 2];

    type Vec = ScalarReg<32>;
    type Mask = ScalarMask<32>;
    type ScalarArch = Self;

    type Array64<T: Debug + Copy> = [T; 4];
    type Array32<T: Debug + Copy> = [T; 8];
    type Array16<T: Debug + Copy> = [T; 16];
    type Array8<T: Debug + Copy> = [T; 32];
}

#[derive(Copy, Clone, Default)]
pub struct Scalar512;
impl Arch for Scalar512 {
    const SIMD_WIDTH: usize = 64;
    const NUM_SIMD_REG: usize = 16;
    const ARCHITECTURE: Architecture = Architecture::Scalar128; type Block2<T: SimdElement> = [Simd<T, Self>; 4]; type Block4<T: SimdElement> = [Simd<T, Self>; 2];

    type Vec = ScalarReg<64>;
    type Mask = ScalarMask<64>;
    type ScalarArch = Self;

    type Array64<T: Debug + Copy> = [T; 8];
    type Array32<T: Debug + Copy> = [T; 16];
    type Array16<T: Debug + Copy> = [T; 32];
    type Array8<T: Debug + Copy> = [T; 64];
}
