use std::ops::*;

use crate::simd::architectures::arch_impl::SimdFamily;
use crate::simd::simd_mask::core::SimdMask;
use crate::simd::simd_reg::core::Simd;
use crate::simd::traits::*;

pub trait SimdBasic<T: SimdElement, F: SimdFamily>: Sized + SimdToArray<T, F> {}

impl<T: SimdElement, F: SimdFamily> SimdBasic<T, F> for Simd<T, F> {}

pub trait SimdRegInteger:
    Sized
    + Shl<Self>
    + ShlAssign<Self>
    + Shr<Self>
    + ShrAssign<Self>
    + Shl<usize>
    + ShlAssign<usize>
    + Shr<usize>
    + ShrAssign<usize>
    + BitAnd
    + BitAndAssign
    + BitOr
    + BitOrAssign
    + BitXor
    + BitXorAssign
{
}

pub trait SimdRegFloat:
    Sized + SimdRound + SimdMulAdd + SimdPartialOrd + SimdEq + SimdSqrt + SimdRecipSqrt
{
}

pub trait SimdContext {
    type Element: SimdElement;
    type Family: SimdFamily;
}
pub trait SimdZero {
    fn zero() -> Self;
}

pub trait SimdLoad<T> {
    fn load_aligned(slice: &[T]) -> Self;
    fn load(slice: &[T]) -> Self;
}

pub trait SimdStore<T> {
    fn store_aligned(self, slice: &mut [T]);
    fn store(self, slice: &mut [T]);
}

pub trait SimdToArray<T: SimdElement, F: SimdFamily> {
    fn to_array(self) -> T::Array<F>;
}

pub trait SimdIota<T> {
    fn iota(offset: T) -> Self;
}

pub trait SimdMaskedLoad<T, F: SimdFamily> {
    fn masked_load(slice: &[T], mask: SimdMask<T, F>) -> Self;
    fn partial_load(slice: &[T], amount: usize) -> Self;
}

pub trait SimdMaskedStore<T, F: SimdFamily> {
    fn masked_store(self, slice: &mut [T], mask: SimdMask<T, F>);
    fn partial_store(self, slice: &mut [T], amount: usize);
}

pub trait SimdAndNot {
    fn andnot(self, rhs: Self) -> Self;
}

pub trait SimdMulAdd {
    fn mul_add(self, mult: Self, add: Self) -> Self;
    fn mul_sub(self, mult: Self, sub: Self) -> Self;
    fn negated_mul_add(self, mult: Self, add: Self) -> Self;
    fn negated_mul_sub(self, mult: Self, sub: Self) -> Self;
}

pub trait SimdRound {
    fn floor(self) -> Self;
    fn round(self) -> Self;
    fn ceil(self) -> Self;
    fn fract(self) -> Self;
}

pub trait SimdEq: SimdContext {
    fn simd_eq(self, rhs: Self) -> SimdMask<Self::Element, Self::Family>;
    fn simd_neq(self, rhs: Self) -> SimdMask<Self::Element, Self::Family>;
}

pub trait SimdPartialOrd: SimdContext {
    fn simd_lt(self, rhs: Self) -> SimdMask<Self::Element, Self::Family>;
    fn simd_le(self, rhs: Self) -> SimdMask<Self::Element, Self::Family>;
    fn simd_gt(self, rhs: Self) -> SimdMask<Self::Element, Self::Family>;
    fn simd_ge(self, rhs: Self) -> SimdMask<Self::Element, Self::Family>;

    fn max(self, rhs: Self) -> Self;
    fn min(self, rhs: Self) -> Self;
}

pub trait SimdSelect: SimdContext {
    fn select(
        self,
        true_values: Simd<Self::Element, Self::Family>,
        false_values: Simd<Self::Element, Self::Family>,
    ) -> Simd<Self::Element, Self::Family>;
}

pub trait SimdMaskToBits {
    fn to_bits(self) -> u64;
}

pub trait SimdSqrt {
    fn sqrt(self) -> Self;
}

pub trait SimdRecipSqrt {
    fn rsqrt(self) -> Self;
}

pub trait SimdClamp: SimdContext {
    fn clamp(self, min_value: Self::Element, max_value: Self::Element) -> Self;
    fn clamp_min(self, min_value: Self::Element) -> Self;
    fn clamp_max(self, max_value: Self::Element) -> Self;
}

pub trait SimdBlockByteShift {
    fn block_left_byte_shift<const N: i32>(self) -> Self;
    fn block_right_byte_shift<const N: i32>(self) -> Self;
}

pub trait SimdImmediateBlend {
    fn blend<const N: i32>(self, false_values: Self) -> Self;
}

// pub trait SimdLaneShift {
//     fn left_lane_shift<const N: i32>(self) -> Self;
//     fn right_lane_shift<const N: i32>(self) -> Self;
// }

// pub trait SimdGather<T>: SimdContext {
//     fn gather<S: SimdElement + SimdElement<BitWidth>, const N: usize>(self, slice: &[S; N]) -> Simd<S, Self::Family>;
// }

//  pub fn zero() -> Self {
//         Self::new(A::zero())
//     }

//     pub fn load_aligned(slice: &[T]) -> Self {
//         unsafe {
//             let ptr = slice.as_ptr();
//             assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
//             assert!(slice.len() >= Self::LANES);
//             Self::new(A::load_aligned(ptr))
//         }
//     }

//     pub fn load(slice: &[T]) -> Self {
//         unsafe {
//             assert!(slice.len() >= Self::LANES);
//             Self::new(A::load_unaligned(slice.as_ptr()))
//         }
//     }

//     pub fn store_aligned(self, slice: &mut [T]) {
//         unsafe {
//             let ptr = slice.as_mut_ptr();
//             assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
//             assert!(slice.len() >= Self::LANES);
//             self.data.store_aligned(ptr);
//         }
//     }

//     pub fn store(self, slice: &mut [T]) {
//         unsafe {
//             let ptr = slice.as_mut_ptr();
//             assert!(slice.len() >= Self::LANES);
//             self.data.store_unaligned(ptr);
//         }
//     }

//     pub fn to_array(self) -> [T; Self::LANES] {
//         let mut array: [T; Self::LANES] = [T::default(); Self::LANES];
//         let mut slice = array.as_mut_slice();
//         self.store(&mut slice);
//         array
//     }

//     pub fn iota(offset: T) -> Self
//     where
//         [T; Self::LANES]:,
//         T: Add<Output = T>,
//     {
//         let iota_array: [T; Self::LANES] = std::array::from_fn(|i| <T as NumCast>::from(i).unwrap() + offset);
//         Self::load(iota_array.as_slice())
//     }

