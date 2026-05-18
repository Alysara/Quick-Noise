use crate::simd::architectures::arch_impl::*;
use crate::simd::array_trait::Array;
use crate::simd::traits::*;
use crate::simd::simd_vec::core::SimdVec;
use crate::simd::simd_mask::core::SimdMask;
use crate::simd::simd_traits::*;
use crate::simd::simd_vec::universal_impl;
use num_traits::NumCast;

impl<T: SimdElement, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn splat(val: T) -> Self {
        Self::new (
            match T::BIT_SIZE {
                BitSize::Size64 => F::Vec::splat_64(val),
                BitSize::Size32 => F::Vec::splat_32(val),
                BitSize::Size16 => F::Vec::splat_16(val),
                BitSize::Size8 => F::Vec::splat_8(val),
            }
        )
    }
}

impl<T: SimdWideType, F: SimdFamily> SimdMaskedLoad<T, F> for SimdVec<T, F> {
    #[inline(always)]
    fn masked_load(slice: &[T], mask: SimdMask<T, F>) -> Self {
        Self::new(
            match T::BIT_SIZE {
                BitSize::Size64 => F::Vec::masked_load_64(slice.as_ptr(), mask.data),
                BitSize::Size32 => F::Vec::masked_load_32(slice.as_ptr(), mask.data),
                _ => unreachable!()
            }
        )
    }

    fn partial_load(slice: &[T], amount: usize) -> Self {
        let amount_vec = Self::splat(<T as NumCast>::from(amount).unwrap());
        let mask = Self::iota(<T as NumCast>::from(0).unwrap()).simd_lt(amount_vec);
        Self::masked_load(slice, mask)
    }
}

impl<T: SimdElement, F: SimdFamily> SimdMaskedStore<T, F> for SimdVec<T, F> {
    #[inline(always)]
    fn masked_store(self, slice: &mut [T], mask: SimdMask<T, F>) {
        match T::BIT_SIZE {
            BitSize::Size64 => F::Vec::masked_store_64(self.data, slice.as_mut_ptr(), mask.data),
            BitSize::Size32 => F::Vec::masked_store_32(self.data, slice.as_mut_ptr(), mask.data),
            _ => unreachable!() // TODO: ADD SUPPORT FOR OTHER SIZES!!!
        }
    }

    // TODO: Support all bit sizes, and also use integer comparisons rather than floats.
    fn partial_store(self, slice: &mut [T], amount: usize) {
        match T::BIT_SIZE {
            BitSize::Size64 => {
                let amount_vec = SimdVec::<f64, F>::splat(amount as f64);
                let mask = SimdVec::<f64, F>::iota(0.0).simd_lt(amount_vec);
                Self::masked_store(self, slice, mask.raw_cast());
            },
            BitSize::Size32 => {
                let amount_vec = SimdVec::<f32, F>::splat(amount as f32);
                let mask = SimdVec::<f32, F>::iota(0.0).simd_lt(amount_vec);
                Self::masked_store(self, slice, mask.raw_cast());
            },
            _ => unreachable!()
        }
    }
}

impl<T: SimdElement + SimdElement<BitWidthType = B32>, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn permute_32(self, indices: SimdVec<u32, F>) -> Self {
        Self::new(self.data.permute_32(indices.data))
    }
}

impl<T: SimdElement, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn permute_8(self, indices: SimdVec<u8, F>) -> Self {
        Self::new(self.data.permute_8(indices.data))
    }

    #[inline(always)]
    pub fn permute_8_pattern_32(self, indices: [u8; 4]) -> Self {
        let pattern = u32::from_ne_bytes(indices);
        let pattern_vec = SimdVec::<u32, F>::splat(pattern);
        Self::new(self.data.permute_8(pattern_vec.data))
    }
}

// TODO: Super early version gather.
impl<F: SimdFamily> SimdVec<u32, F> {
    pub fn gather<S: SimdElement + SimdElement<BitWidthType = B32>, const N: usize>(self, slice: &[S; N]) -> SimdVec<S, F> {
        if N <= Self::LANES {
            let data = SimdVec::<S, F>::load(&slice[..]);
            data.permute_32(self)
        } else {
            SimdVec::new(self.data.gather_32_from_32::<S, 4>(slice.as_ptr()))
        }
    }
}

impl<F: SimdFamily> SimdVec<u64, F> {
    pub fn gather<S: SimdElement + SimdElement<BitWidthType = B64>, const N: usize>(self, slice: &[S; N]) -> SimdVec<S, F> {
        SimdVec::new(self.data.gather_64_from_64::<S, 8>(slice.as_ptr()))
    }
}

// TODO: Add other types of lane shifts.
impl<T: SimdElement, F: SimdFamily> SimdVec<T, F> {
    pub fn left_lane_shift<const N: i32>(self) -> Self {
        match T::BIT_SIZE {
            BitSize::Size32 => Self::new(self.data.left_lane_shift_32::<N>()),
            _ => unreachable!()
        }
    }
    pub fn right_lane_shift<const N: i32>(self) -> Self {
        match T::BIT_SIZE {
            BitSize::Size32 => Self::new(self.data.right_lane_shift_32::<N>()),
            _ => unreachable!()
        }
    }
}
