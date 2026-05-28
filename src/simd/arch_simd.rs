// use std::simd::{Simd, SimdElement, StdFloat};
use crate::simd::architectures::arch_impl::SimdFamily;
#[cfg(target_arch = "aarch64")]
use crate::simd::architectures::families::Neon;
use crate::simd::architectures::families::Scalar128;
#[cfg(target_arch = "x86_64")]
use crate::simd::architectures::families::{Avx2, Avx512, Sse};
use crate::simd::simd_mask::core::SimdMask;
use crate::simd::simd_reg::core::Simd;

// Static dispatch for identifying lane sizes and number of simd registers.

cfg_if::cfg_if! {
    // x86_64
    if #[cfg(all(target_arch = "x86_64", target_feature = "avx512f"))] {
        pub const SIMD_WIDTH: usize = 64;
        pub const NUM_SIMD_REG: usize = 32;
        pub type ArchSimd<T> = Simd<T, Avx512>;
        pub type ArchMask<T> = SimdMask<T, Avx512>;
        pub type ArchFamily = Avx512;
    } else if #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] {
        pub const SIMD_WIDTH: usize = 32;
        pub const NUM_SIMD_REG: usize = 16;
        pub type ArchSimd<T> = Simd<T, Avx2>;
        pub type ArchMask<T> = SimdMask<T, Avx2>;
        pub type ArchFamily = Avx2;
    } else if #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))] {
        pub const SIMD_WIDTH: usize = 16;
        pub const NUM_SIMD_REG: usize = 16;
        pub type ArchSimd<T> = Simd<T, Sse>;
        pub type ArchMask<T> = SimdMask<T, Sse>;
        pub type ArchFamily = Sse;
    }

    // aarch64
    // else if #[cfg(all(target_arch = "aarch64", target_feature = "sve"))] {
    //     pub const SIMD_WIDTH: usize = 32;
    //     pub const NUM_SIMD_REG: usize = 32;
    //     pub type ArchSimd<T> = Simd<T, Sse>;
    //     pub type ArchMask<T> = SimdMask<T, Sse>;
    //     pub type ArchFamily = Sse;

    else if #[cfg(all(target_arch = "aarch64", target_feature = "neon"))] {
        pub const SIMD_WIDTH: usize = 16;
        pub const NUM_SIMD_REG: usize = 32;
        pub type ArchSimd<T> = Simd<T, Neon>;
        pub type ArchMask<T> = SimdMask<T, Neon>;
        pub type ArchFamily = Neon;
    }

    // wasm
    else if #[cfg(all(any(target_arch = "wasm32", target_arch = "wasm64"), target_feature = "simd128"))] {
        pub const SIMD_WIDTH: usize = 16;
        pub const NUM_SIMD_REG: usize = 16;
        pub type ArchSimd<T> = Simd<T, Scaler128>;
        pub type ArchMask<T> = SimdMask<T, Scaler128>;
        pub type ArchFamily = Scaler128;
    }

    // riscv
    else if #[cfg(all(any(target_arch = "riscv64", target_arch = "riscv32"), target_feature = "v"))] {
        pub const SIMD_WIDTH: usize = 32;
        pub const NUM_SIMD_REG: usize = 32;
        pub type ArchSimd<T> = Simd<T, Scaler128>;
        pub type ArchMask<T> = SimdMask<T, Scaler128>;
        pub type ArchFamily = Scaler128;
    }

    // fallback
    else {
        pub const SIMD_WIDTH: usize = 4;
        pub const NUM_SIMD_REG: usize = 8;
        pub type ArchSimd<T> = Simd<T, Scaler128>;
        pub type ArchMask<T> = SimdMask<T, Scaler128>;
        pub type ArchFamily = Scaler128;
    }
}

pub type ScalarFamily = <ArchFamily as SimdFamily>::ScalarFamily;
pub type ScalarSimd<T> = Simd<T, ScalarFamily>;
pub type ScalarMask<T> = SimdMask<T, ScalarFamily>;

// pub type ArchSimd<T: SimdInfo> = Simd<T, { T::LANES }>;

// pub trait SimdInfo: SimdElement {
//     const LANES: usize;
// }

// impl<T: SimdElement> SimdInfo for T {
//     const LANES: usize = SIMD_WIDTH / std::mem::size_of::<T>();
// }

// pub trait ArchSimdExt<T: SimdInfo>
// where
//     ArchSimd<T>: StdFloat,
//     ArchSimd<T>: Mul<Output = ArchSimd<T>>,
// {
//     fn quintic_lerp(self) -> Self;
//     fn gather_u32<const N: usize>(array: &[f32; N], indicies: ArchSimd<u32>) -> ArchSimd<f32>;
//     fn dyn_swizzle_u32<const N: usize>(array: &[f32; N], indicies: ArchSimd<u32>) -> ArchSimd<f32>;
//     fn fast_sin(self) -> Self;
//     fn fast_cos(self) -> Self;
// }

// impl<T: SimdInfo + Float> ArchSimdExt<T> for ArchSimd<T>
// where
//     ArchSimd<T>: StdFloat,
//     ArchSimd<T>: Mul<Output = ArchSimd<T>>,
//     ArchSimd<T>: Neg<Output = ArchSimd<T>>,
// {
//     // #[inline(never)]
//     fn quintic_lerp(self) -> Self {
//         let six = Self::splat(NumCast::from(6.0).unwrap());
//         let ten = Self::splat(NumCast::from(10.0).unwrap());
//         let neg_fifteen = Self::splat(NumCast::from(-15.0).unwrap());
//         let t = self;
//         t * t * t * t.mul_add(t.mul_add(six, neg_fifteen), ten)
//     }

//     // #[inline(never)]
//     // fn gather_u32<const N: usize>(array: &[T; N], indicies: ArchSimd<u32>) -> Self {
//     //     let indicies_array = indicies.to_array();
//     //     let mut result: [T; N] = [NumCast::from(0).unwrap(); N];
//     //     for i in 0..T::LANES {
//     //         unsafe {
//     //             *result.get_unchecked_mut(i) = *array.get_unchecked(*indicies_array.get_unchecked(i as usize) as usize);
//     //         }
//     //     }
//     //     unsafe { std::mem::transmute_copy(&result) }
//     // }

//     // #[inline(never)]
//     fn gather_u32<const N: usize>(
//         array: &[f32; N],
//         indices: ArchSimd<u32>,
//     ) -> ArchSimd<f32> {
//         debug_assert!(N >= 8);
//         unsafe {
//             let base_ptr = array.as_ptr();
//             let index_vec: __m256i = std::mem::transmute(indices);
//             let gathered: __m256 = _mm256_i32gather_ps(base_ptr, index_vec, 4);
//             std::mem::transmute(gathered)
//         }
//     }

//     fn dyn_swizzle_u32<const N: usize>(
//         array: &[f32; N],
//         indices: ArchSimd<u32>,
//     ) -> ArchSimd<f32> {
//         debug_assert!(N >= 8);
//         unsafe {
//             let base_ptr = array.as_ptr();
//             let index_vec: __m256i = std::mem::transmute(indices);
//             let data: __m256 = _mm256_loadu_ps(base_ptr);
//             let result: __m256 = _mm256_permutevar8x32_ps(data, index_vec);
//             std::mem::transmute(result)
//         }
//     }

//     fn fast_sin(self) -> Self {
//         let x2 = self * self;
//         let c1 = Self::splat(NumCast::from(1.0 * SQRT_2).unwrap());
//         let c2 = Self::splat(NumCast::from(0.16666667 * SQRT_2).unwrap());
//         let c3 = Self::splat(NumCast::from(0.00833333 * SQRT_2).unwrap());
//         let c4 = Self::splat(NumCast::from(0.000198413 * SQRT_2).unwrap());
//         let r = (-x2).mul_add(c4, c3);    // c3 - x2*c4
//         let r = (-x2).mul_add(r, c2);     // c2 - x2*r
//         let r = (-x2).mul_add(r, c1);     // 1 - x2*r
//         self * r
//     }

//     fn fast_cos(self) -> Self {
//         let x2 = self * self;
//         let c1 = Self::splat(NumCast::from(1.0 * SQRT_2).unwrap());
//         let c2 = Self::splat(NumCast::from(0.5 * SQRT_2).unwrap());
//         let c3 = Self::splat(NumCast::from(0.04166667 * SQRT_2).unwrap());
//         let c4 = Self::splat(NumCast::from(0.00138889 * SQRT_2).unwrap());
//         let c5 = Self::splat(NumCast::from(0.0000248016 * SQRT_2).unwrap());
//         // nested: 1 - x2*(c2 - x2*(c3 - x2*(c4 - x2*c5)))
//         let r = (-x2).mul_add(c5, c4);         // c4 - x2*c5
//         let r = (-x2).mul_add(r, c3);          // c3 - x2*r
//         let r = (-x2).mul_add(r, c2);          // c2 - x2*r
//         (-x2).mul_add(r, c1)                   // 1 - x2*r
//     }
// }
