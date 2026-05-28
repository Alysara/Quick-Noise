use crate::simd::architectures::arch_impl::*;
use num_traits::NumCast;
use crate::simd::traits::*;
use std::marker::PhantomData;
use crate::simd::simd_reg::core::Simd;
use std::fmt;
use std::ops::*;
use crate::simd::simd_traits::*;
use crate::simd::array_trait::Array;

// Universal Operations.
impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub(crate) fn new(data: F::Vec) -> Self {
        Self { data, _marker: PhantomData }
    }
}

impl<T: SimdElement, F: SimdFamily> SimdZero for Simd<T, F> {
    #[inline(always)]
    fn zero() -> Self {
        Self::new(F::Vec::zero())
    }
}

impl<T: SimdElement, F: SimdFamily> SimdLoad<T> for Simd<T, F> {
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

impl<T: SimdElement, F: SimdFamily> SimdStore<T> for Simd<T, F> {
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

impl<T: SimdElement, F: SimdFamily> SimdToArray<T, F> for Simd<T, F> {
    #[inline(always)]
    fn to_array(self) -> T::Array<F> {
        let mut array = T::Array::<F>::from_fn(|_| T::from(0).unwrap());
        self.store(&mut array.as_mut_slice());
        array
    }
}

// TODO: Fdd non-generic constant version solution.
impl<T: SimdElement, F: SimdFamily> SimdIota<T> for Simd<T, F> {
    #[inline(always)]
    fn iota(offset: T) -> Self {
        let iota_array = T::Array::<F>::from_fn(|i| <T as NumCast>::from(i).unwrap() + (offset));
        Self::load(iota_array.as_slice())
    }
}

impl<T: SimdElement, F: SimdFamily> fmt::Debug for Simd<T, F> where Simd<T, F>: SimdBasic<T, F> {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let buf= self.to_array();
        write!(f, "{:?}", buf)
    }
}

// === Assign operations ===
impl<T: SimdElement, F: SimdFamily> AddAssign for Simd<T, F> 
where
    Self: Add<Output = Self> + Copy
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> SubAssign for Simd<T, F>
where
    Self: Sub<Output = Self> + Copy
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> MulAssign for Simd<T, F>
where
    Self: Mul<Output = Self> + Copy
{
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> DivAssign for Simd<T, F>
where
    Self: Div<Output = Self> + Copy
{
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> RemAssign for Simd<T, F>
where
    Self: Rem<Output = Self> + Copy
{
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitAndAssign for Simd<T, F>
where
    Self: BitAnd<Output = Self> + Copy
{
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitOrAssign for Simd<T, F>
where
    Self: BitOr<Output = Self> + Copy
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl<T: SimdElement, F: SimdFamily> BitXorAssign for Simd<T, F>
where
    Self: BitXor<Output = Self> + Copy
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
    pub fn raw_cast<S: SimdElement>(self) -> Simd::<S, F> {
        Simd::new(self.data)
    }
}

impl<T: SimdElement, F: SimdFamily> Default for Simd<T, F> {
    #[inline(always)]
    fn default() -> Self {
        Self::splat(<T as NumCast>::from(T::default()).unwrap())
    }
}

impl<T: SimdElement, F: SimdFamily> SimdClamp for Simd<T, F> {
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

impl<T: SimdElement, F: SimdFamily> SimdBlockByteShift for Simd<T, F> {
    #[inline(always)]
    fn block_left_byte_shift<const N: i32>(self) -> Self {
        Self::new(self.data.block_left_byte_shift::<N>())
    }
    #[inline(always)]
    fn block_right_byte_shift<const N: i32>(self) -> Self {
        Self::new(self.data.block_right_byte_shift::<N>())
    }
}

impl<T: SimdElement, F: SimdFamily> SimdImmediateBlend for Simd<T, F> {
    #[inline(always)]
    fn blend<const N: i32>(self, false_values: Self) -> Self {
        match T::BIT_SIZE {
            BitSize::Size64 => Self::new(self.data.blend_32::<N>(false_values.data)),
            BitSize::Size32 => Self::new(self.data.blend_32::<N>(false_values.data)),
            _ => unreachable!() // TODO: Add 16 and 8 for immediate blend.
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
