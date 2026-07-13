use num_traits::NumCast;

use crate::simd::architectures::interface::*;
use crate::simd::register::Simd;
use crate::simd::traits::*;

impl<T: SimdFloat, F: SimdFamily> Simd<T, F> {
    #[inline(always)]
    pub fn floor(self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.floor_f64(),
                SimdType::F32 => self.data.floor_f32(),
                _ => unreachable!(),
            })
        }
    }

    #[inline(always)]
    pub fn round(self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.round_f64(),
                SimdType::F32 => self.data.round_f32(),
                _ => unreachable!(),
            })
        }
    }

    #[inline(always)]
    pub fn ceil(self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.ceil_f64(),
                SimdType::F32 => self.data.ceil_f32(),
                _ => unreachable!(),
            })
        }
    }

    #[inline(always)]
    pub fn fract(self) -> Self {
        self - self.floor()
    }

    #[inline(always)]
    pub fn mul_add(self, mult: Self, add: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.mul_add_f64(mult.data, add.data),
                SimdType::F32 => self.data.mul_add_f32(mult.data, add.data),
                _ => unreachable!(),
            })
        }
    }

    #[inline(always)]
    pub fn mul_sub(self, mult: Self, sub: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.mul_sub_f64(mult.data, sub.data),
                SimdType::F32 => self.data.mul_sub_f32(mult.data, sub.data),
                _ => unreachable!(),
            })
        }
    }

    #[inline(always)]
    pub fn negated_mul_add(self, mult: Self, add: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.negated_mul_add_f64(mult.data, add.data),
                SimdType::F32 => self.data.negated_mul_add_f32(mult.data, add.data),
                _ => unreachable!(),
            })
        }
    }

    #[inline(always)]
    pub fn negated_mul_sub(self, mult: Self, sub: Self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.negated_mul_sub_f64(mult.data, sub.data),
                SimdType::F32 => self.data.negated_mul_sub_f32(mult.data, sub.data),
                _ => unreachable!(),
            })
        }
    }

    // === Casts ===

    #[inline(always)]
    pub fn cast_int_trunc(self) -> Simd<T::Signed, F> {
        unsafe { Simd::new(self.data.float_to_int_trunc()) }
    }

    #[inline(always)]
    pub fn cast_int_round(self) -> Simd<T::Signed, F> {
        unsafe { Simd::new(self.data.float_to_int_round()) }
    }

    // TODO: INCORRECT for edge cases.
    #[inline(always)]
    pub fn cast_uint_trunc(self) -> Simd<T::Unsigned, F> {
        unsafe { Simd::new(self.data.float_to_int_trunc()) }
    }

    #[inline(always)]
    pub fn cast_uint_round(self) -> Simd<T::Unsigned, F> {
        unsafe { Simd::new(self.data.float_to_int_round()) }
    }

    // TODO: Move this into quick-noise later.
    #[inline(always)]
    pub fn quintic_lerp(self) -> Self {
        let six = Self::splat(NumCast::from(6.0).unwrap());
        let ten = Self::splat(NumCast::from(10.0).unwrap());
        let fifteen = Self::splat(NumCast::from(15.0).unwrap());
        let t = self;
        t * t * t * t.mul_add(t.mul_sub(six, fifteen), ten)
    }

    #[inline(always)]
    pub fn cubic_lerp(self) -> Self {
        let neg_two = Self::splat(NumCast::from(-2.0).unwrap());
        let three = Self::splat(NumCast::from(3.0).unwrap());
        let t = self;
        t * t * t.mul_add(neg_two, three)
    }

    pub fn sqrt(self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => self.data.sqrt_f64(),
                SimdType::F32 => self.data.sqrt_f32(),
                _ => unreachable!(),
            })
        }
    }

    pub fn abs(self) -> Self {
        unsafe {
            Self::new(match T::TYPE {
                SimdType::F64 => Simd::<u64, F>::splat(T::SIGN_MASK as u64)
                    .data
                    .and(self.data),
                SimdType::F32 => Simd::<u32, F>::splat(T::SIGN_MASK as u32)
                    .data
                    .and(self.data),
                _ => unreachable!(),
            })
        }
    }

    /// Only rsqrt_f32 currently supported.
    pub fn rsqrt(self) -> Self {
        unsafe { Self::new(self.data.rsqrt_f32()) }
    }
}

