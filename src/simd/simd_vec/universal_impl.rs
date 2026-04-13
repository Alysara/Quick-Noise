use crate::simd::architectures::arch_impl::*;
use num_traits::NumCast;
use crate::simd::traits::*;
use std::marker::PhantomData;
use crate::simd::simd_vec::core::SimdVec;
use std::fmt;
use std::ops::*;
use crate::simd::simd_traits::*;
use crate::simd::array_trait::Array;

// Universal Operations.
impl<T: SimdElement, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub(crate) fn new(data: F::Vec) -> Self {
        Self { data, _marker: PhantomData }
    }
}

impl<T: SimdElement, F: SimdFamily> SimdZero for SimdVec<T, F> {
    #[inline(always)]
    fn zero() -> Self {
        Self::new(F::Vec::zero())
    }
}

impl<T: SimdElement, F: SimdFamily> SimdLoad<T> for SimdVec<T, F> {
    #[inline(always)]
    fn load_aligned(slice: &[T]) -> Self {
        unsafe {
            let ptr = slice.as_ptr();
            // assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
            debug_assert!(slice.len() >= Self::LANES);
            Self::new(F::Vec::load_aligned(ptr))
        }
    }

    #[inline(always)]
    fn load(slice: &[T]) -> Self {
        unsafe {
            // assert!(slice.len() >= Self::LANES);
            Self::new(F::Vec::load_unaligned(slice.as_ptr()))
        }
    }
}

impl<T: SimdElement, F: SimdFamily> SimdStore<T> for SimdVec<T, F> {
    #[inline(always)]
    fn store_aligned(self, slice: &mut [T]) {
        let ptr = slice.as_mut_ptr();
        debug_assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        debug_assert!(slice.len() >= Self::LANES);
        self.data.store_aligned(ptr);
    }

    #[inline(always)]
    fn store(self, slice: &mut [T]) {
        unsafe {
            let ptr = slice.as_mut_ptr();
            // assert!(slice.len() >= Self::LANES);
            self.data.store_unaligned(ptr);
        }
    }
}

impl<T: SimdElement, F: SimdFamily> SimdToArray<T, F> for SimdVec<T, F> {
    #[inline(always)]
    fn to_array(self) -> T::Array<F> {
        let mut array = T::Array::<F>::from_fn(|_| T::from(0).unwrap());
        self.store(&mut array.as_mut_slice());
        array
    }
}

// TODO: Fdd non-generic constant version solution.
impl<T: SimdElement, F: SimdFamily> SimdIota<T> for SimdVec<T, F> {
    #[inline(always)]
    fn iota(offset: T) -> Self {
        let iota_array = T::Array::<F>::from_fn(|i| <T as NumCast>::from(i).unwrap() + (offset));
        Self::load(iota_array.as_slice())
    }
}

impl<T: SimdElement, F: SimdFamily> fmt::Debug for SimdVec<T, F> where SimdVec<T, F>: SimdVecBasic<T, F> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let buf= self.to_array();
        write!(f, "{:?}", buf)
    }
}

// === Assign operations ===
impl<T: SimdElement, F: SimdFamily> AddAssign for SimdVec<T, F> 
where
    Self: Add<Output = Self> + Copy
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> SubAssign for SimdVec<T, F>
where
    Self: Sub<Output = Self> + Copy
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> MulAssign for SimdVec<T, F>
where
    Self: Mul<Output = Self> + Copy
{
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> DivAssign for SimdVec<T, F>
where
    Self: Div<Output = Self> + Copy
{
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> RemAssign for SimdVec<T, F>
where
    Self: Rem<Output = Self> + Copy
{
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitAndAssign for SimdVec<T, F>
where
    Self: BitAnd<Output = Self> + Copy
{
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitOrAssign for SimdVec<T, F>
where
    Self: BitOr<Output = Self> + Copy
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitXorAssign for SimdVec<T, F>
where
    Self: BitXor<Output = Self> + Copy
{
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> Neg for SimdVec<T, F> {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self::zero() - self
    }
}

impl<T: SimdElement, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn raw_cast<S: SimdElement>(self) -> SimdVec::<S, F> {
        SimdVec::new(self.data)
    }
}

impl<T: SimdElement, F: SimdFamily> Default for SimdVec<T, F> {
    #[inline(always)]
    fn default() -> Self {
        Self::splat(<T as NumCast>::from(T::default()).unwrap())
    }
}

impl<T: SimdElement, F: SimdFamily> SimdClamp for SimdVec<T, F> {
    #[inline(always)]
    fn clamp(self, min_value: T, max_value: T) -> Self {
        self.clamp_min(min_value).clamp_max(max_value)
    }
    #[inline(always)]
    fn clamp_max(self, max_value: Self::Element) -> Self {
        let max_vec = Self::splat(max_value);
        self.simd_gt(max_vec).select(max_vec, self)
    }
    #[inline(always)]
    fn clamp_min(self, min_value: Self::Element) -> Self {
        let min_vec = Self::splat(min_value);
        self.simd_lt(min_vec).select(min_vec, self)
    }
}