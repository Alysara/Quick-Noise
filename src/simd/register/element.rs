use std::fmt;
use std::marker::PhantomData;
use std::ops::*;

use num_traits::NumCast;

use crate::simd::architectures::interface::*;
use crate::simd::array_trait::Array;
use crate::simd::mask::Mask;
use crate::simd::register::Simd;
use crate::simd::traits::*;

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub(crate) fn new(data: F::Vec) -> Self {
        Self {
            data,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn zero() -> Self {
        unsafe { Self::new(F::Vec::zero()) }
    }

    #[inline(always)]
    pub fn from_aligned_slice(slice: &[T]) -> Self {
        let ptr = slice.as_ptr();
        assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        assert!(slice.len() >= Self::LANES);
        unsafe { Self::new(F::Vec::load_aligned(ptr)) }
    }

    /// # Safety
    /// Does not check if the slice goes out of bounds.
    #[inline(always)]
    pub unsafe fn from_aligned_slice_unchecked(slice: &[T]) -> Self {
        let ptr = slice.as_ptr();
        debug_assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        debug_assert!(slice.len() >= Self::LANES);
        unsafe { Self::new(F::Vec::load_aligned(ptr)) }
    }

    #[inline(always)]
    pub fn from_slice(slice: &[T]) -> Self {
        if slice.len() >= Self::LANES {
            unsafe { Self::new(F::Vec::load_unaligned(slice.as_ptr())) }
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
        unsafe { Self::new(F::Vec::load_unaligned(slice.as_ptr())) }
    }

    #[inline(always)]
    pub fn copy_to_aligned_slice(self, slice: &mut [T]) {
        let ptr = slice.as_mut_ptr();
        assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        assert!(slice.len() >= Self::LANES);
        unsafe { self.data.store_aligned(ptr) };
    }

    /// # Safety
    /// Does not check if the slice goes out of bounds.
    #[inline(always)]
    pub unsafe fn copy_to_aligned_slice_unchecked(self, slice: &mut [T]) {
        let ptr = slice.as_mut_ptr();
        debug_assert!(ptr.align_offset(Self::SIMD_WIDTH) == 0);
        debug_assert!(slice.len() >= Self::LANES);
        unsafe { self.data.store_aligned(ptr) };
    }

    #[inline(always)]
    pub fn copy_to_slice(self, slice: &mut [T]) {
        if slice.len() >= Self::LANES {
            let ptr = slice.as_mut_ptr();
            unsafe { self.data.store_unaligned(ptr) };
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
        unsafe { self.data.store_unaligned(ptr) };
    }

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

impl<T: SimdElement, F: SimdFamily> Add for Simd<T, F> {
    type Output = Self;
    #[inline(always)]
    fn add(self, rhs: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.f64_add(rhs.data),
                SimdType::F32 => self.data.f32_add(rhs.data),
                SimdType::I64 => self.data.i64_add(rhs.data),
                SimdType::I32 => self.data.i32_add(rhs.data),
                SimdType::I16 => self.data.i16_add(rhs.data),
                SimdType::I8 => self.data.i8_add(rhs.data),
                SimdType::U64 => self.data.i64_add(rhs.data),
                SimdType::U32 => self.data.i32_add(rhs.data),
                SimdType::U16 => self.data.i16_add(rhs.data),
                SimdType::U8 => self.data.i8_add(rhs.data),
            })
        }
    }
}

impl<T: SimdElement, F: SimdFamily> Sub for Simd<T, F> {
    type Output = Self;
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.f64_sub(rhs.data),
                SimdType::F32 => self.data.f32_sub(rhs.data),
                SimdType::I64 => self.data.i64_sub(rhs.data),
                SimdType::I32 => self.data.i32_sub(rhs.data),
                SimdType::I16 => self.data.i16_sub(rhs.data),
                SimdType::I8 => self.data.i8_sub(rhs.data),
                SimdType::U64 => self.data.i64_sub(rhs.data),
                SimdType::U32 => self.data.i32_sub(rhs.data),
                SimdType::U16 => self.data.i16_sub(rhs.data),
                SimdType::U8 => self.data.i8_sub(rhs.data),
            })
        }
    }
}


impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    /// Clamps the values in a register between two bounds, inclusive.
    ///
    /// # Parameters:
    /// - `lower_bound`: Minimum value after clamping
    /// - `upper_bound`: Maximum value after clamping
    #[inline(always)]
    pub fn clamp(self, lower_bound: Self, upper_bound: Self) -> Self {
        self.min(upper_bound).max(lower_bound)
    }

    /// Shifts bytes N to the left within 128-bit blocks.
    #[inline(always)]
    pub fn block_left_byte_shift<const N: i32>(self) -> Self {
        unsafe { Self::new(self.data.block_left_byte_shift::<N>()) }
    }

    /// Shifts bytes N to the right within 128-bit blocks.
    #[inline(always)]
    pub fn block_right_byte_shift<const N: i32>(self) -> Self {
        unsafe { Self::new(self.data.block_right_byte_shift::<N>()) }
    }

    /// Blends two registers using an immediate mask.
    #[inline(always)]
    pub fn blend<const N: i32>(self, false_values: Self) -> Self {
        unsafe {
            match T::BIT_SIZE {
                BitSize::Size64 => Self::new(self.data.blend_32::<N>(false_values.data)),
                BitSize::Size32 => Self::new(self.data.blend_32::<N>(false_values.data)),
                _ => unreachable!(), // TODO: Add 16 and 8 for immediate blend.
            }
        }
    }
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    /// Broadcasts a value across the entire register.
    #[inline(always)]
    pub fn splat(val: T) -> Self {
        unsafe {
            Self::new(match T::BIT_SIZE {
                BitSize::Size64 => F::Vec::splat_64(val),
                BitSize::Size32 => F::Vec::splat_32(val),
                BitSize::Size16 => F::Vec::splat_16(val),
                BitSize::Size8 => F::Vec::splat_8(val),
            })
        }
    }

    /// Loads a register according to a mask.
    #[inline(always)]
    pub fn masked_load(slice: &[T], mask: Mask<T, F>) -> Self {
        unsafe {
            Self::new(match T::BIT_SIZE {
                BitSize::Size64 => F::Vec::masked_load_64(slice.as_ptr(), mask.data),
                BitSize::Size32 => F::Vec::masked_load_32(slice.as_ptr(), mask.data),
                _ => unreachable!(),
            })
        }
    }

    /// Loads only the first `amount` elements into the register.
    pub fn partial_load(slice: &[T], amount: usize) -> Self {
        assert!(slice.len() >= Self::LANES, "Attempted to do a partial load, but index is out of bounds!");
        let amount_vec = Self::splat(<T as NumCast>::from(amount).unwrap());
        let mask = Self::iota(<T as NumCast>::from(0).unwrap()).simd_lt(amount_vec);
        Self::masked_load(slice, mask)
    }

    /// Loads only the first `amount` elements into the register.
    ///
    /// # Safety
    /// - Slice must be greater than or equal to `ArchSimd::<T>::LANES`
    pub unsafe fn partial_load_unchecked(slice: &[T], amount: usize) -> Self {
        debug_assert!(slice.len() >= Self::LANES, "Index is out of bounds in unsafe code!");
        let amount_vec = Self::splat(<T as NumCast>::from(amount).unwrap());
        let mask = Self::iota(<T as NumCast>::from(0).unwrap()).simd_lt(amount_vec);
        Self::masked_load(slice, mask)
    }

    /// Stores the register using a given mask.
    #[inline(always)]
    pub fn masked_store(self, slice: &mut [T], mask: Mask<T, F>) {
        unsafe {
            match T::BIT_SIZE {
                BitSize::Size64 => {
                    F::Vec::masked_store_64(self.data, slice.as_mut_ptr(), mask.data)
                }
                BitSize::Size32 => {
                    F::Vec::masked_store_32(self.data, slice.as_mut_ptr(), mask.data)
                }
                _ => unreachable!(), // TODO: ADD SUPPORT FOR OTHER SIZES!!!
            }
        }
    }

    /// Stores only the first `amount` elements into the register.
    pub fn partial_store(self, slice: &mut [T], amount: usize) {
        match T::BIT_SIZE {
            BitSize::Size64 => {
                let iota = Simd::iota(0u64);
                let n_vec = Simd::splat(amount as u64);
                let mask = n_vec.simd_gt(iota);
                Self::masked_store(self, slice, mask.raw_cast());
            }
            BitSize::Size32 => {
                let iota = Simd::iota(0u32);
                let n_vec = Simd::splat(amount as u32);
                let mask = n_vec.simd_gt(iota);
                Self::masked_store(self, slice, mask.raw_cast());
            }
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn simd_eq(self, rhs: Self) -> Mask<T, F> {
        unsafe {
            Mask::new(match T::TYPE {
                SimdType::F64 => self.data.cmp_f64_eq(rhs.data),
                SimdType::F32 => self.data.cmp_f32_eq(rhs.data),
                SimdType::U64 => self.data.cmp_i64_eq(rhs.data),
                SimdType::U32 => self.data.cmp_i32_eq(rhs.data),
                SimdType::U16 => self.data.cmp_i16_eq(rhs.data),
                SimdType::U8 => self.data.cmp_i8_eq(rhs.data),
                SimdType::I64 => self.data.cmp_i64_eq(rhs.data),
                SimdType::I32 => self.data.cmp_i32_eq(rhs.data),
                SimdType::I16 => self.data.cmp_i16_eq(rhs.data),
                SimdType::I8 => self.data.cmp_i8_eq(rhs.data),
                // _ => unreachable!() // TODO: Add integer types .
            })
        }
    }

    #[inline(always)]
    pub fn simd_neq(self, rhs: Self) -> Mask<T, F> {
        unsafe {
            Mask::new(match T::TYPE {
                SimdType::F64 => self.data.cmp_f64_neq(rhs.data),
                SimdType::F32 => self.data.cmp_f32_neq(rhs.data),
                _ => self.simd_eq(rhs).data,
            })
        }
    }

    #[inline(always)]
    pub fn simd_gt(self, rhs: Self) -> Mask<T, F> {
        unsafe {
            Mask::new(match T::TYPE {
                SimdType::F64 => self.data.cmp_f64_gt(rhs.data),
                SimdType::F32 => self.data.cmp_f32_gt(rhs.data),
                SimdType::U64 => self.data.cmp_i64_gt(rhs.data),
                SimdType::U32 => self.data.cmp_i32_gt(rhs.data),
                SimdType::U16 => self.data.cmp_i16_gt(rhs.data),
                SimdType::U8 => self.data.cmp_i8_gt(rhs.data),
                SimdType::I64 => self.data.cmp_i64_gt(rhs.data),
                SimdType::I32 => self.data.cmp_i32_gt(rhs.data),
                SimdType::I16 => self.data.cmp_i16_gt(rhs.data),
                SimdType::I8 => self.data.cmp_i8_gt(rhs.data),
                // _ => unreachable!() // TODO: Add integer types .
            })
        }
    }

    // TODO: Find better way to handle comparisons.
    #[inline(always)]
    pub fn simd_ge(self, rhs: Self) -> Mask<T, F> {
        unsafe {
            Mask::new(match T::TYPE {
                SimdType::F64 => self.data.cmp_f64_ge(rhs.data),
                SimdType::F32 => self.data.cmp_f32_ge(rhs.data),
                // SimdType::U64 => self.data.cmp_i64_gt(rhs.data).or(self.data.cmp_i64_eq(rhs.data)),
                // SimdType::U32 => self.data.cmp_i32_gt(rhs.data).or(self.data.cmp_i32_eq(rhs.data)),
                // SimdType::U16 => self.data.cmp_i16_gt(rhs.data).or(self.data.cmp_i16_eq(rhs.data)),
                // SimdType::U8 => self.data.cmp_i8_gt(rhs.data).or(self.data.cmp_i8_eq(rhs.data)),
                SimdType::I64 => self
                    .data
                    .cmp_i64_gt(rhs.data)
                    .or(self.data.cmp_i64_eq(rhs.data)),
                SimdType::I32 => self
                    .data
                    .cmp_i32_gt(rhs.data)
                    .or(self.data.cmp_i32_eq(rhs.data)),
                SimdType::I16 => self
                    .data
                    .cmp_i16_gt(rhs.data)
                    .or(self.data.cmp_i16_eq(rhs.data)),
                SimdType::I8 => self
                    .data
                    .cmp_i8_gt(rhs.data)
                    .or(self.data.cmp_i8_eq(rhs.data)),
                _ => panic!("Unsigned integer types for less than not implemented!"), // TODO: Add integer types .
            })
        }
    }

    #[inline(always)]
    pub fn simd_lt(self, rhs: Self) -> Mask<T, F> {
        unsafe {
            Mask::new(match T::TYPE {
                SimdType::F64 => self.data.cmp_f64_lt(rhs.data),
                SimdType::F32 => self.data.cmp_f32_lt(rhs.data),
                _ => panic!("Less than for integers not implemented!"), // TODO: Add integer types .
            })
        }
    }

    #[inline(always)]
    pub fn simd_le(self, rhs: Self) -> Mask<T, F> {
        unsafe {
            Mask::new(match T::TYPE {
                SimdType::F64 => self.data.cmp_f64_le(rhs.data),
                SimdType::F32 => self.data.cmp_f32_le(rhs.data),
                _ => panic!("Less than or equal not implemented for integers!"), // TODO: Add integer types .
            })
        }
    }

    // TODO: Handle max for U64/I64.
    pub fn max(self, rhs: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.max_f64(rhs.data),
                SimdType::F32 => self.data.max_f32(rhs.data),
                SimdType::I32 => self.data.max_i32(rhs.data),
                SimdType::I16 => self.data.max_i16(rhs.data),
                SimdType::I8 => self.data.max_i8(rhs.data),
                SimdType::U32 => self.data.max_u32(rhs.data),
                SimdType::U16 => self.data.max_u16(rhs.data),
                SimdType::U8 => self.data.max_u8(rhs.data),
                _ => panic!("Max for U64/I64 not implemented!"),
            })
        }
    }

    pub fn min(self, rhs: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.min_f64(rhs.data),
                SimdType::F32 => self.data.min_f32(rhs.data),
                SimdType::I32 => self.data.min_i32(rhs.data),
                SimdType::I16 => self.data.min_i16(rhs.data),
                SimdType::I8 => self.data.min_i8(rhs.data),
                SimdType::U32 => self.data.min_u32(rhs.data),
                SimdType::U16 => self.data.min_u16(rhs.data),
                SimdType::U8 => self.data.min_u8(rhs.data),
                _ => panic!("Min for U64/I64 not implemented!"),
            })
        }
    }
}

impl<T: SimdElement + SimdElement<BitWidthType = B32>, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn permute_32(self, indices: Simd<u32, F>) -> Self {
        unsafe { Self::new(self.data.permute_32(indices.data)) }
    }
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn permute_8(self, indices: Simd<u8, F>) -> Self {
        unsafe { Self::new(self.data.permute_8(indices.data)) }
    }

    #[inline(always)]
    pub fn permute_8_pattern_32(self, indices: [u8; 4]) -> Self {
        let pattern = u32::from_ne_bytes(indices);
        let pattern_vec = Simd::<u32, F>::splat(pattern);
        unsafe { Self::new(self.data.permute_8(pattern_vec.data)) }
    }
}

// TODO: Super early version gather.
impl<F: SimdFamily> Simd<u32, F> {
    pub fn gather<S: SimdElement + SimdElement<BitWidthType = B32>, const N: usize>(
        self,
        slice: &[S; N],
    ) -> Simd<S, F> {
        if N <= Self::LANES {
            let data = Simd::<S, F>::from_slice(&slice[..]);
            data.permute_32(self)
        } else {
            unsafe { Simd::new(self.data.gather_32_from_32::<S, 4>(slice.as_ptr())) }
        }
    }
}

impl<F: SimdFamily> Simd<u64, F> {
    pub fn gather<S: SimdElement + SimdElement<BitWidthType = B64>, const N: usize>(
        self,
        slice: &[S; N],
    ) -> Simd<S, F> {
        unsafe { Simd::new(self.data.gather_64_from_64::<S, 8>(slice.as_ptr())) }
    }
}

// TODO: Add other types of lane shifts.
impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    pub fn left_lane_shift(self, n: u32) -> Self {
        match T::BIT_SIZE {
            BitSize::Size32 => unsafe { Self::new(self.data.left_lane_shift_32(n)) },
            _ => unreachable!(),
        }
    }
    pub fn right_lane_shift(self, n: u32) -> Self {
        match T::BIT_SIZE {
            BitSize::Size32 => unsafe { Self::new(self.data.right_lane_shift_32(n)) },
            _ => unreachable!(),
        }
    }
}
