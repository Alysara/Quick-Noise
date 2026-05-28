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
use crate::simd::simd_reg::core::Simd;

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

    // TODO: Support other bit_sizes
    #[inline(always)]
    pub fn first_n_true(n: u32) -> SimdMask<T, F> {
        let iota = Simd::iota(0u32);
        let n_vec = Simd::splat(n);
        n_vec.simd_gt(iota).raw_cast()
    }

    #[inline(always)]
    pub fn first_n_false(n: u32) -> SimdMask<T, F> {
        let iota = Simd::iota(1u32);
        let n_vec = Simd::splat(n);
        iota.simd_gt(n_vec).raw_cast()
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
    fn select(self, true_values: Simd<T, F>, false_values: Simd<T, F>) -> Simd<T, F> {
        match T::BIT_SIZE {
            BitSize::Size64 => Simd::new(self.data.vblend_64(true_values.data, false_values.data)),
            BitSize::Size32 => Simd::new(self.data.vblend_32(true_values.data, false_values.data)),
            BitSize::Size8 => Simd::new(self.data.vblend_8(true_values.data, false_values.data)),
            _ => panic!("Select for 16 bit types not implemented yet!")
        }
    }
}

impl<T: SimdElement, F: SimdFamily> SimdMaskToBits for SimdMask<T, F> {
    fn to_bits(self) -> u64 {
        match T::BIT_SIZE {
            BitSize::Size64 => self.data.to_bits_64(),
            BitSize::Size32 => self.data.to_bits_32(),
            BitSize::Size8 => self.data.to_bits_8(),
            _ => unreachable!() // TODO: Add to_bits_16.
        }
    }
}