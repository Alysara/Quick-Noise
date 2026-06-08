// use std::ops::{Index, IndexMut};
//
// use crate::simd::architectures::arch_impl::SimdFamily;
// use crate::simd::architectures::families::Avx2;
// use crate::simd::simd_mask::core::SimdMask;
// use crate::simd::simd_reg::core::Simd;
// use crate::simd::traits::{BitWidth, SimdElement};
//
// pub struct SimdBlock<T: SimdElement, F: SimdFamily, const N: usize> {
//     registers: [Simd<T, F>; N],
// }
//
// impl<T: SimdElement, F: SimdFamily, const N: usize> Default for SimdBlock<T, F, N> {
//     fn default() -> Self {
//         Self {
//             registers: std::array::from_fn(|_| Simd::<T, F>::default()),
//         }
//     }
// }
//
// impl<T: SimdElement, F: SimdFamily, const N: usize> Index<usize> for SimdBlock<T, F, N> {
//     type Output = Simd<T, F>;
//     fn index(&self, index: usize) -> &Self::Output {
//         &self.registers[index]
//     }
// }
//
// impl<T: SimdElement, F: SimdFamily, const N: usize> IndexMut<usize> for SimdBlock<T, F, N> {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {
//         &mut self.registers[index]
//     }
// }
//
// impl<T: SimdElement, F: SimdFamily, const N: usize> SimdBlock<T, Avx2, N> {
//     const BIT_SIZE: usize = T::BitWidthType::BIT_SIZE;
//     const BLOCK_SIZE: usize = 128 / BIT_SIZE;
//     const REGISTER_SIZE: usize = (F::SIMD_WIDTH * 8) / Self::BIT_SIZE;
//     const SIZE: usize = N * Self::REGISTER_SIZE;
//
//     pub fn splat(val: T) -> Self {
//         Self {
//             registers: std::array::from_fn(|_| Simd::<T, Avx2>::splat(val)),
//         }
//     }
//
//     pub const fn swizzle<const usize: M>(&mut self, indices: &[usize; M]) {
//         const _: () = assert!(M == SIZE, "Number of indices does not match SimdBlock size");
//
//         const BIT_SIZE: usize = T::BitWidthType::BIT_SIZE;
//         const BLOCK_SIZE: usize = 128 / BIT_SIZE;
//
//         for i in 0..N {
//             if indicies[i] >= N {
//                 panic!("Invalid swizzle value!");
//             }
//         }
//
//         // Check if all lanes should be set to one value.
//         let identity = 'outer: {
//             for i in 1..N {
//                 if indices[i - 1] != indices[i] {
//                     break 'outer false;
//                 }
//             }
//             true
//         };
//
//         // HANDLE IDENTITY CASE.
//
//         // Check if all swizzle operations can be performed
//         // inside of the same 128-bit blocks.
//         let cross_block = Self::check_blocked::<128>(&indices);
//
//         // HANDLE NON-CROSS BLOCK CASE.
//         let blocked_64 = Self::check_blocked::<64>(&indices);
//         let blocked_32 = Self::check_blocked::<32>(&indices);
//         let blocked_16 = Self::check_blocked::<16>(&indices);
//
//         // HANDLE BLOCKED CASES, and also check for using the immediate version of 16 to replace the
//         // 32, versus using a mask. If mask repeats, better to use mask.
//     }
//
//     const fn swizzle_register<const M: usize>(indices: &[usize; M], register: usize) {
//         const _: () = assert!(M == SIZE, "Number of indices does not match SimdBlock size");
//
//         // Check if the swizzle values are valid.
//         let start = register * Self::REGISTER_SIZE;
//         let end = start + Self::REGISTER_SIZE;
//         for i in start..end {
//             if indicies[i] - start >= Self::REGISTER_SIZE {
//                 panic!("Invalid register swizzle value!");
//             }
//         }
//
//         // Check if the swizzle is already solved.
//         for i in start..end {
//             if indices[i] == i {
//                 break;
//             }
//             return;
//         }
//
//         // Check if all lanes should be set to one value.
//         let identity = 'outer: {
//             for i in (start + 1)..end {
//                 if indices[i - 1] != indices[i] {
//                     break 'outer false;
//                 }
//             }
//             true
//         };
//
//         // HANDLE IDENTITY CASE.
//
//         // Check if all swizzle operations can be performed
//         // inside of the same 128-bit blocks.
//         let cross_block = Self::check_blocked::<M, 128>(&indices);
//
//         // HANDLE NON-CROSS BLOCK CASE.
//         let blocked_64 = Self::check_blocked::<M, 64>(&indices);
//         let blocked_32 = Self::check_blocked::<M, 32>(&indices);
//         let blocked_16 = Self::check_blocked::<M, 16>(&indices);
//
//         // HANDLE BLOCKED CASES, and also check for using the immediate version of 16 to replace the
//         // 32, versus using a mask. If mask repeats, better to use mask.
//     }
//
//     const fn identity_swizzle<const M: usize>(indices: &[usize; M], register: usize) -> bool {
//         let identity_reg = indicies[register * Self::REGISTER_SIZE] - register * Self::REGISTER_SIZE;
//         let swizzle_indices = Simd::<T::UType, F>::splat(identity_reg);
//         match T::BIT_SIZE {
//             BitSize::Size64 =>
//         }
//     }
//
//     const fn check_blocked<const M: usize, const W: usize>(indices: &[usize; M]) -> bool {
//         if T::BitWidthType::BIT_SIZE >= W {
//             return true;
//         }
//
//         const CHUNK_SIZE: usize = W / T::BIT_SIZE;
//         for i in 1..N {
//             if indices[i] / CHUNK_SIZE != i / CHUNK_SIZE {
//                 return false;
//             }
//         }
//         return true
//     }
//
//
//
//     const fn check_register_blocked<const M: usize, const W: usize>(indices: &[usize; M], register: usize) -> bool {
//         if T::BitWidthType::BIT_SIZE >= W {
//             return true;
//         }
//
//         const CHUNK_SIZE: usize = W / T::BIT_SIZE;
//         let start = register * Self::REGISTER_SIZE;
//         let end = start + Self::REGISTER_SIZE;
//
//         for i in start..end {
//             if indices[i] / CHUNK_SIZE != i / CHUNK_SIZE {
//                 return false;
//             }
//         }
//         return true
//     }
// }
//
