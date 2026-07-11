use std::fmt;
use std::marker::PhantomData;
use std::ops::*;

use num_traits::NumCast;

use crate::simd::architectures::arch_impl::*;
use crate::simd::array_trait::Array;
use crate::simd::simd_reg::core::Simd;
use crate::simd::traits::*;

// Universal Operations.
impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub(crate) fn new(data: F::Vec) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn zero() -> Self {
        Self::new(F::Vec::zero())
    }

    #[inline(always)]
    pub fn from_aligned_slice(slice: &[T]) -> Self {
        let ptr = slice.as_ptr();
        assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        assert!(slice.len() >= Self::LANES);
        Self::new(F::Vec::load_aligned(ptr))
    }

    /// # Safety
    /// Does not check if the slice goes out of bounds.
    #[inline(always)]
    pub unsafe fn from_aligned_slice_unchecked(slice: &[T]) -> Self {
        let ptr = slice.as_ptr();
        debug_assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        debug_assert!(slice.len() >= Self::LANES);
        Self::new(F::Vec::load_aligned(ptr))
    }

    #[inline(always)]
    pub fn from_slice(slice: &[T]) -> Self {
        if slice.len() >= Self::LANES {
            Self::new(F::Vec::load_unaligned(slice.as_ptr()))
        } else {
            let mut array = Self::zero().to_array();
            for (arr, val) in array.iter_mut().zip(slice.iter()) {
                *arr = *val;
            }
            unsafe { Self::from_slice_unchecked(array.as_slice()) }
        }
    }

    /// # Safety
    /// Requires allocated memory to be behind (left) of the slice.
    /// Bounds are not checked.
    /// Length of the slice must be less than or equal to the number of lanes.
    #[inline(always)]
    pub unsafe fn from_slice_partial(slice: &[T]) -> Self {
        debug_assert!(slice.len() <= Self::LANES);
        unsafe {
            let offset = Self::LANES - slice.len();
            let raw_ptr = slice.as_ptr().sub(offset);
            let simd = Self::new(F::Vec::load_unaligned(raw_ptr));
            simd.left_lane_shift(offset as u32)
        }
    }

    /// # Safety
    /// Does not check if the slice goes out of bounds.
    #[inline(always)]
    pub unsafe fn from_slice_unchecked(slice: &[T]) -> Self {
        debug_assert!(slice.len() >= Self::LANES);
        Self::new(F::Vec::load_unaligned(slice.as_ptr()))
    }

    #[inline(always)]
    pub fn copy_to_aligned_slice(self, slice: &mut [T]) {
        let ptr = slice.as_mut_ptr();
        assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        assert!(slice.len() >= Self::LANES);
        self.data.store_aligned(ptr);
    }

    /// # Safety
    /// Does not check if the slice goes out of bounds.
    #[inline(always)]
    pub unsafe fn copy_to_aligned_slice_unchecked(self, slice: &mut [T]) {
        let ptr = slice.as_mut_ptr();
        debug_assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        debug_assert!(slice.len() >= Self::LANES);
        self.data.store_aligned(ptr);
    }

    #[inline(always)]
    pub fn copy_to_slice(self, slice: &mut [T]) {
        if slice.len() >= Self::LANES {
            let ptr = slice.as_mut_ptr();
            self.data.store_unaligned(ptr);
        } else {
            // Scalar/tail case.
            let array = self.to_array();
            slice
                .iter_mut()
                .zip(array.iter())
                .for_each(|(src, new)| *src = *new);
        }
    }

    /// # Safety
    /// Does not check if the slice goes out of bounds.
    #[inline(always)]
    pub unsafe fn copy_to_slice_unchecked(self, slice: &mut [T]) {
        let ptr = slice.as_mut_ptr();
        debug_assert!(slice.len() >= Self::LANES);
        self.data.store_unaligned(ptr);
    }

    // pub fn copy_to_slice(self, slice: &mut [T]) {
    //
    // }

    /// Converts the Simd register into an array.
    ///
    /// # Example
    ///
    /// TODO
    /// use quick_noise::simd::
    #[inline(always)]
    pub fn to_array(self) -> T::Array<F> {
        let mut array = T::Array::<F>::from_fn(|_| T::from(0).unwrap());
        self.copy_to_slice(array.as_mut_slice());
        array
    }

    #[inline(always)]
    pub fn iota(offset: T) -> Self {
        let iota_array =
            T::Array::<F>::from_fn(|i| <T as NumCast>::from(i).unwrap().safe_add(offset));
        Self::from_slice(iota_array.as_slice())
    }
}

impl<T: SimdElement, F: SimdFamily> fmt::Debug for Simd<T, F> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let buf = self.to_array();
        write!(f, "{:?}", buf)
    }
}

// === Assign operations ===
impl<T: SimdElement, F: SimdFamily> AddAssign for Simd<T, F>
where
    Self: Add<Output = Self> + Copy,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> SubAssign for Simd<T, F>
where
    Self: Sub<Output = Self> + Copy,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> MulAssign for Simd<T, F>
where
    Self: Mul<Output = Self> + Copy,
{
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> DivAssign for Simd<T, F>
where
    Self: Div<Output = Self> + Copy,
{
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> RemAssign for Simd<T, F>
where
    Self: Rem<Output = Self> + Copy,
{
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitAndAssign for Simd<T, F>
where
    Self: BitAnd<Output = Self> + Copy,
{
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitOrAssign for Simd<T, F>
where
    Self: BitOr<Output = Self> + Copy,
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitXorAssign for Simd<T, F>
where
    Self: BitXor<Output = Self> + Copy,
{
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> Neg for Simd<T, F> {
    type Output = Self;
    #[inline(always)]
    fn neg(self) -> Self {
        Self::zero() - self
    }
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn raw_cast<S: SimdElement>(self) -> Simd<S, F> {
        Simd::new(self.data)
    }
}

impl<T: SimdElement, F: SimdFamily> Default for Simd<T, F> {
    #[inline(always)]
    fn default() -> Self {
        Self::splat(<T as NumCast>::from(T::default()).unwrap())
    }
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn clamp(self, lower_bound: Self, upper_bound: Self) -> Self {
        self.min(upper_bound).max(lower_bound)
    }

    #[inline(always)]
    pub fn clamp_max(self, max_value: T) -> Self {
        let max_vec = Self::splat(max_value);
        self.simd_gt(max_vec).select(max_vec, self)
    }

    #[inline(always)]
    pub fn clamp_min(self, min_value: T) -> Self {
        let min_vec = Self::splat(min_value);
        self.simd_lt(min_vec).select(min_vec, self)
    }

    #[inline(always)]
    pub fn block_left_byte_shift<const N: i32>(self) -> Self {
        Self::new(self.data.block_left_byte_shift::<N>())
    }

    #[inline(always)]
    pub fn block_right_byte_shift<const N: i32>(self) -> Self {
        Self::new(self.data.block_right_byte_shift::<N>())
    }

    #[inline(always)]
    pub fn blend<const N: i32>(self, false_values: Self) -> Self {
        match T::BIT_SIZE {
            BitSize::Size64 => Self::new(self.data.blend_32::<N>(false_values.data)),
            BitSize::Size32 => Self::new(self.data.blend_32::<N>(false_values.data)),
            _ => unreachable!(), // TODO: Add 16 and 8 for immediate blend.
        }
    }
}

// // Lack of const expr requires explicit declaration of every case.
// impl<T: SimdElement, F: SimdFamily> SimdLaneShift for Simd<T, F> {
//     fn left_lane_shift<const N: i32>(self) -> Self {
//         if Self::SIMD_WIDTH == 16 {
//             match T::BIT_SIZE {
//                 BitSize::Size8 => self.block_left_byte_shift::<N>(),
//                 BitSize::Size16 => {
//                     match N {
//                         0 => self,
//                         1 => self.block_left_byte_shift::<2>(),
//                         2 => self.block_left_byte_shift::<4>(),
//                         3 => self.block_left_byte_shift::<6>(),
//                         4 => self.block_left_byte_shift::<8>(),
//                         5 => self.block_left_byte_shift::<10>(),
//                         6 => self.block_left_byte_shift::<12>(),
//                         7 => self.block_left_byte_shift::<14>(),
//                         8 => self.block_left_byte_shift::<16>(),
//                         9 => self.block_left_byte_shift::<18>(),
//                         10 => self.block_left_byte_shift::<20>(),
//                         11 => self.block_left_byte_shift::<22>(),
//                         12 => self.block_left_byte_shift::<24>(),
//                         13 => self.block_left_byte_shift::<26>(),
//                         14 => self.block_left_byte_shift::<28>(),
//                         15 => self.block_left_byte_shift::<30>(),
//                         _ => Self::zero(), // Zero out large shifts.
//                     }
//                 }
//                 BitSize::Size32 => {
//                     match N {
//                         0 => self,
//                         1 => self.block_left_byte_shift::<4>(),
//                         2 => self.block_left_byte_shift::<8>(),
//                         3 => self.block_left_byte_shift::<12>(),
//                         4 => self.block_left_byte_shift::<16>(),
//                         5 => self.block_left_byte_shift::<20>(),
//                         6 => self.block_left_byte_shift::<24>(),
//                         7 => self.block_left_byte_shift::<28>(),
//                         _ => Self::zero(), // Zero out large shifts.
//                     }
//                 },
//                 BitSize::Size64 => {
//                     match N {
//                         0 => self,
//                         1 => self.block_left_byte_shift::<8>(),
//                         2 => self.block_left_byte_shift::<16>(),
//                         3 => self.block_left_byte_shift::<24>(),
//                         _ => Self::zero(), // Zero out large shifts.
//                     }
//                 }
//             }
//         } else if Self::SIMD_WIDTH == 32 {

//         }
//     }
//     fn right_lane_shift<const N: i32>(self) -> Self {

//     }
// }
