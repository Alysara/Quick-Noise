use crate::simd::architectures::arch_impl::*;
use crate::simd::traits::*;
use std::fmt::Debug;
use std::fmt;
use std::ops::*;
use num_traits::NumCast;
use crate::simd::traits::*;
use std::marker::PhantomData;
use crate::simd::simd_vec::core::SimdVec;
use crate::simd::simd_mask::core::SimdMask;
use crate::simd::simd_traits::*;
use std::mem::transmute_copy;

impl<T: SimdFloat, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn fract(self) -> Self {
        self - self.floor()
    }
}

impl<T: SimdFloat, F: SimdFamily> SimdRound for SimdVec<T, F> {
    #[inline(always)]
    fn floor(self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.floor_f64(),
            SimdType::F32 => self.data.floor_f32(),
        _ => unreachable!()} )
    }

    #[inline(always)]
    fn round(self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.round_f64(),
            SimdType::F32 => self.data.round_f32(),
        _ => unreachable!()} )
    }

    #[inline(always)]
    fn ceil(self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.ceil_f64(),
            SimdType::F32 => self.data.ceil_f32(),
        _ => unreachable!()} )
    }

    #[inline(always)]
    fn fract(self) -> Self {
        self - self.floor()
    }
}

impl<T: SimdFloat, F: SimdFamily> SimdMulAdd for SimdVec<T, F> {
    #[inline(always)]
    fn mul_add(self, mult: Self, add: Self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.mul_add_f64(mult.data, add.data),
            SimdType::F32 => self.data.mul_add_f32(mult.data, add.data),
        _ => unreachable!()} )
    }

    #[inline(always)]
    fn mul_sub(self, mult: Self, sub: Self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.mul_sub_f64(mult.data, sub.data),
            SimdType::F32 => self.data.mul_sub_f32(mult.data, sub.data),
        _ => unreachable!()} )
    }

    #[inline(always)]
    fn negated_mul_add(self, mult: Self, add: Self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.negated_mul_add_f64(mult.data, add.data),
            SimdType::F32 => self.data.negated_mul_add_f32(mult.data, add.data),
        _ => unreachable!()} )
    }

    #[inline(always)]
    fn negated_mul_sub(self, mult: Self, sub: Self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.negated_mul_sub_f64(mult.data, sub.data),
            SimdType::F32 => self.data.negated_mul_sub_f32(mult.data, sub.data),
        _ => unreachable!()} )
    }
}

impl<T: SimdElement, F: SimdFamily> SimdEq for SimdVec<T, F> {
    #[inline(always)]
    fn simd_eq(self, rhs: Self) -> SimdMask<T, F> {
        SimdMask::new(match T::TYPE {
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

    #[inline(always)]
    fn simd_neq(self, rhs: Self) -> SimdMask<T, F> {
        SimdMask::new(match T::TYPE {
            SimdType::F64 => self.data.cmp_f64_neq(rhs.data),
            SimdType::F32 => self.data.cmp_f32_neq(rhs.data),
            _ => unreachable!() // TODO: Add integer types .
        })
    }
}

impl<T: SimdElement, F: SimdFamily> SimdPartialOrd for SimdVec<T, F> {
    #[inline(always)]
    fn simd_gt(self, rhs: Self) -> SimdMask<T, F> {
        SimdMask::new(match T::TYPE {
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

    // TODO: Find better way to handle comparisons.
    #[inline(always)]
    fn simd_ge(self, rhs: Self) -> SimdMask<T, F> {
        SimdMask::new(match T::TYPE {
            SimdType::F64 => self.data.cmp_f64_ge(rhs.data),
            SimdType::F32 => self.data.cmp_f32_ge(rhs.data),
            // SimdType::U64 => self.data.cmp_i64_gt(rhs.data).or(self.data.cmp_i64_eq(rhs.data)),
            // SimdType::U32 => self.data.cmp_i32_gt(rhs.data).or(self.data.cmp_i32_eq(rhs.data)),
            // SimdType::U16 => self.data.cmp_i16_gt(rhs.data).or(self.data.cmp_i16_eq(rhs.data)),
            // SimdType::U8 => self.data.cmp_i8_gt(rhs.data).or(self.data.cmp_i8_eq(rhs.data)),
            SimdType::I64 => self.data.cmp_i64_gt(rhs.data).or(self.data.cmp_i64_eq(rhs.data)),
            SimdType::I32 => self.data.cmp_i32_gt(rhs.data).or(self.data.cmp_i32_eq(rhs.data)),
            SimdType::I16 => self.data.cmp_i16_gt(rhs.data).or(self.data.cmp_i16_eq(rhs.data)),
            SimdType::I8 => self.data.cmp_i8_gt(rhs.data).or(self.data.cmp_i8_eq(rhs.data)),
            _ => unreachable!() // TODO: Add integer types .
        })
    }

    #[inline(always)]
    fn simd_lt(self, rhs: Self) -> SimdMask<T, F> {
        SimdMask::new(match T::TYPE {
            SimdType::F64 => self.data.cmp_f64_lt(rhs.data),
            SimdType::F32 => self.data.cmp_f32_lt(rhs.data),
            _ => unreachable!() // TODO: Add integer types .
        })
    }

    #[inline(always)]
    fn simd_le(self, rhs: Self) -> SimdMask<T, F> {
        SimdMask::new(match T::TYPE {
            SimdType::F64 => self.data.cmp_f64_le(rhs.data),
            SimdType::F32 => self.data.cmp_f32_le(rhs.data),
            _ => unreachable!() // TODO: Add integer types .
        })
    }

    // TODO: Handle max for U64/I64.
    fn max(self, rhs: Self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.max_f64(rhs.data),
            SimdType::F32 => self.data.max_f32(rhs.data),
            SimdType::I32 => self.data.max_i32(rhs.data),
            SimdType::I16 => self.data.max_i16(rhs.data),
            SimdType::I8 => self.data.max_i8(rhs.data),
            SimdType::U32 => self.data.max_u32(rhs.data),
            SimdType::U16 => self.data.max_u16(rhs.data),
            SimdType::U8 => self.data.max_u8(rhs.data),
            _ => panic!("Max for U64/I64 not implemented!")
        })
    }

    fn min(self, rhs: Self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.min_f64(rhs.data),
            SimdType::F32 => self.data.min_f32(rhs.data),
            SimdType::I32 => self.data.min_i32(rhs.data),
            SimdType::I16 => self.data.min_i16(rhs.data),
            SimdType::I8 => self.data.min_i8(rhs.data),
            SimdType::U32 => self.data.min_u32(rhs.data),
            SimdType::U16 => self.data.min_u16(rhs.data),
            SimdType::U8 => self.data.min_u8(rhs.data),
            _ => panic!("Min for U64/I64 not implemented!")
        })
    }
}

// === Casts ===

impl<T: SimdFloat, F: SimdFamily> SimdVec<T, F> {
    #[inline(always)]
    pub fn cast_int_trunc(self) -> SimdVec<T::Signed, F> {
        SimdVec::new(self.data.float_to_int_trunc())
    }
    #[inline(always)]
    pub fn cast_int_round(self) -> SimdVec<T::Signed, F> {
        SimdVec::new(self.data.float_to_int_round())
    }

    // TODO: INCORRECT for edge cases.
    #[inline(always)]
    pub fn cast_uint_trunc(self) -> SimdVec<T::Unsigned, F> {
        SimdVec::new(self.data.float_to_int_trunc())
    }
    #[inline(always)]
    pub fn cast_uint_round(self) -> SimdVec<T::Unsigned, F> {
        SimdVec::new(self.data.float_to_int_round())
    }
    // #[inline(always)]
    // pub fn cast_int_raw(self) -> SimdVec<T::Signed, F> {
    //     unsafe { SimdVec::new(transmute_copy(&self.data)) }
    // }
    // #[inline(always)]
    // pub fn cast_uint_raw(self) -> SimdVec<T::Unsigned, F> {
    //     unsafe { SimdVec::new(transmute_copy(&self.data)) }
    // }

    // TODO: Move this into quick-noise later.
    #[inline(always)]
    pub fn quintic_lerp(self) -> Self {
        let six = Self::splat(NumCast::from(6.0).unwrap());
        let ten = Self::splat(NumCast::from(10.0).unwrap());
        let fifteen = Self::splat(NumCast::from(15.0).unwrap());
        let t = self;
        t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten)
    }
}

impl<T: SimdFloat, F: SimdFamily> SimdSqrt for SimdVec<T, F> {
    fn sqrt(self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => self.data.sqrt_f64(),
            SimdType::F32 => self.data.sqrt_f32(),
            _ => unreachable!()
        })
    }
}

impl<T: SimdFloat, F: SimdFamily> SimdVec<T, F> {
    pub fn abs(self) -> Self {
        Self::new(match T::TYPE {
            SimdType::F64 => SimdVec::<u64, F>::splat(T::SIGN_MASK as u64).data.and_not(self.data),
            SimdType::F32 => SimdVec::<u32, F>::splat(T::SIGN_MASK as u32).data.and_not(self.data),
            _ => unreachable!()
        })
    }
}

impl<F: SimdFamily> SimdRecipSqrt for SimdVec<f32, F> {
    fn rsqrt(self) -> Self {
        Self::new(self.data.rsqrt_f32())
    }
}
