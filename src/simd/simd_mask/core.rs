use crate::simd::architectures::arch_impl::SimdAllBitsImpl;
use crate::simd::architectures::arch_impl::SimdFamily;
use crate::simd::simd_traits::*;
use crate::simd::traits::*;
use std::fmt::Debug;
use std::fmt;
use std::ops::*;
use num_traits::NumCast;
use crate::simd::traits::*;
use std::marker::PhantomData;
use crate::simd::architectures::arch_impl::MaskArch;
use crate::simd::architectures::arch_impl::*;
use crate::simd::simd_vec::core::SimdVec;

#[derive(Clone, Copy)]
pub struct SimdMask<T, F: SimdFamily> {
    pub(crate) data: F::Mask,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: SimdElement, F: SimdFamily> SimdContext for SimdMask<T, F> {
    type Element = T;
    type Family = F;
}

impl<T: SimdElement, F: SimdFamily> SimdMask<T, F> {
    #[inline(always)]
    pub(crate) fn new(data: F::Mask) -> Self {
        Self { data, _marker: PhantomData }
    }

    #[inline(always)]
    pub fn raw_cast<S: SimdElement>(self) -> SimdMask<S, F> {
        SimdMask::new(self.data)
    }

    #[inline(always)]
    pub fn all_false(self) -> bool {
        self.data.all_zero()
    }
}

impl<T: SimdElement, F: SimdFamily> BitAnd for SimdMask<T, F> {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self::new(self.data.and(rhs.data))
    } 
}

impl<T: SimdElement, F: SimdFamily> BitOr for SimdMask<T, F> {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self::new(self.data.or(rhs.data))
    } 
}

impl<T: SimdElement, F: SimdFamily> BitXor for SimdMask<T, F> {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self::new(self.data.xor(rhs.data))
    } 
}

impl<T: SimdElement, F: SimdFamily> SimdAndNot for SimdMask<T, F> {
    #[inline(always)]
    fn andnot(self, rhs: Self) -> Self {
        Self::new(self.data.and_not(rhs.data))
    }
}

impl<T: SimdElement, F: SimdFamily> Not for SimdMask<T, F> {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self {
        Self::new(self.data.not())
    }
}

// TODO: Add 16 bit select.
impl<T: SimdElement, F: SimdFamily> SimdSelect for SimdMask<T, F> {
    fn select(self, true_values: SimdVec<T, F>, false_values: SimdVec<T, F>) -> SimdVec<T, F> {
        match T::BIT_SIZE {
            BitSize::Size64 => SimdVec::new(self.data.vblend_64(true_values.data, false_values.data)),
            BitSize::Size32 => SimdVec::new(self.data.vblend_32(true_values.data, false_values.data)),
            BitSize::Size8 => SimdVec::new(self.data.vblend_8(true_values.data, false_values.data)),
            _ => panic!("Select for 16 bit types not implemented yet!")
        }
    }
}