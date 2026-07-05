use crate::simd::architectures::arch_impl::*;
use crate::simd::array_trait::Array;
use crate::simd::traits::*;
use crate::simd::simd_reg::core::Simd;
use crate::simd::simd_mask::core::Mask;
use num_traits::NumCast;

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
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

    #[inline(always)]
    pub fn masked_load(slice: &[T], mask: Mask<T, F>) -> Self {
        Self::new(
            match T::BIT_SIZE {
                BitSize::Size64 => F::Vec::masked_load_64(slice.as_ptr(), mask.data),
                BitSize::Size32 => F::Vec::masked_load_32(slice.as_ptr(), mask.data),
                _ => unreachable!()
            }
        )
    }

    pub fn partial_load(slice: &[T], amount: usize) -> Self {
        let amount_vec = Self::splat(<T as NumCast>::from(amount).unwrap());
        let mask = Self::iota(<T as NumCast>::from(0).unwrap()).simd_lt(amount_vec);
        Self::masked_load(slice, mask)
    }
    
    #[inline(always)]
    pub fn masked_store(self, slice: &mut [T], mask: Mask<T, F>) {
        match T::BIT_SIZE {
            BitSize::Size64 => F::Vec::masked_store_64(self.data, slice.as_mut_ptr(), mask.data),
            BitSize::Size32 => F::Vec::masked_store_32(self.data, slice.as_mut_ptr(), mask.data),
            _ => unreachable!() // TODO: ADD SUPPORT FOR OTHER SIZES!!!
        }
    }

    // TODO: Support all bit sizes, and also use integer comparisons rather than floats.
    pub fn partial_store(self, slice: &mut [T], amount: usize) {
        match T::BIT_SIZE {
            BitSize::Size64 => {
                let iota = Simd::iota(0u64);
                let n_vec = Simd::splat(amount as u64);
                let mask = n_vec.simd_gt(iota);
                Self::masked_store(self, slice, mask.raw_cast());
            },
            BitSize::Size32 => {
                let iota = Simd::iota(0u32);
                let n_vec = Simd::splat(amount as u32);
                let mask = n_vec.simd_gt(iota);
                Self::masked_store(self, slice, mask.raw_cast());
            },
            _ => unreachable!()
        }
    }
}

impl<T: SimdElement + SimdElement<BitWidthType = B32>, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn permute_32(self, indices: Simd<u32, F>) -> Self {
        Self::new(self.data.permute_32(indices.data))
    }
}

impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn permute_8(self, indices: Simd<u8, F>) -> Self {
        Self::new(self.data.permute_8(indices.data))
    }

    #[inline(always)]
    pub fn permute_8_pattern_32(self, indices: [u8; 4]) -> Self {
        let pattern = u32::from_ne_bytes(indices);
        let pattern_vec = Simd::<u32, F>::splat(pattern);
        Self::new(self.data.permute_8(pattern_vec.data))
    }
}

// TODO: Super early version gather.
impl<F: SimdFamily> Simd<u32, F> {
    pub fn gather<S: SimdElement + SimdElement<BitWidthType = B32>, const N: usize>(self, slice: &[S; N]) -> Simd<S, F> {
        if N <= Self::LANES {
            let data = Simd::<S, F>::from_slice(&slice[..]);
            data.permute_32(self)
        } else {
            Simd::new(self.data.gather_32_from_32::<S, 4>(slice.as_ptr()))
        }
    }
}

impl<F: SimdFamily> Simd<u64, F> {
    pub fn gather<S: SimdElement + SimdElement<BitWidthType = B64>, const N: usize>(self, slice: &[S; N]) -> Simd<S, F> {
        Simd::new(self.data.gather_64_from_64::<S, 8>(slice.as_ptr()))
    }
}

// TODO: Add other types of lane shifts.
impl<T: SimdElement, F: SimdFamily> Simd<T, F> {
    pub fn left_lane_shift(self, n: u32) -> Self {
        match T::BIT_SIZE {
            BitSize::Size32 => Self::new(self.data.left_lane_shift_32(n)),
            _ => unreachable!()
        }
    }
    pub fn right_lane_shift(self, n: u32) -> Self {
        match T::BIT_SIZE {
            BitSize::Size32 => Self::new(self.data.right_lane_shift_32(n)),
            _ => unreachable!()
        }
    }
}
