use std::arch::aarch64::*;
use std::mem::{transmute, transmute_copy};
use crate::simd::architectures::arch_impl::*;
use crate::simd::architectures::macros::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Neon(pub float32x4_t);
impl SimdArch for Neon {}
impl MaskArch for Neon {}

impl SimdAddImpl for Neon {
    #[inline(always)] fn f64_add(self, rhs: Self) -> Self { self_from_op!(vaddq_f64, self, rhs) }
    #[inline(always)] fn f32_add(self, rhs: Self) -> Self { self_from_op!(vaddq_f32, self, rhs) }
    #[inline(always)] fn i64_add(self, rhs: Self) -> Self { self_from_op!(vaddq_s64, self, rhs) }
    #[inline(always)] fn i32_add(self, rhs: Self) -> Self { self_from_op!(vaddq_s32, self, rhs) }
    #[inline(always)] fn i16_add(self, rhs: Self) -> Self { self_from_op!(vaddq_s16, self, rhs) }
    #[inline(always)] fn i8_add(self, rhs: Self) -> Self { self_from_op!(vaddq_s8, self, rhs) }
}

impl SimdSubImpl for Neon {
    #[inline(always)] fn f64_sub(self, rhs: Self) -> Self { self_from_op!(vsubq_f64, self, rhs) }
    #[inline(always)] fn f32_sub(self, rhs: Self) -> Self { self_from_op!(vsubq_f32, self, rhs) }
    #[inline(always)] fn i64_sub(self, rhs: Self) -> Self { self_from_op!(vsubq_s64, self, rhs) }
    #[inline(always)] fn i32_sub(self, rhs: Self) -> Self { self_from_op!(vsubq_s32, self, rhs) }
    #[inline(always)] fn i16_sub(self, rhs: Self) -> Self { self_from_op!(vsubq_s16, self, rhs) }
    #[inline(always)] fn i8_sub(self, rhs: Self) -> Self { self_from_op!(vsubq_s8, self, rhs) }
}

impl SimdMulImpl for Neon {
    #[inline(always)] fn f64_mul(self, rhs: Self) -> Self { self_from_op!(vmulq_f64, self, rhs) }
    #[inline(always)] fn f32_mul(self, rhs: Self) -> Self { self_from_op!(vmulq_f32, self, rhs) }
    #[inline(always)] fn i32_mul(self, rhs: Self) -> Self { self_from_op!(vmulq_s32, self, rhs) }
    #[inline(always)] fn i16_mul(self, rhs: Self) -> Self { self_from_op!(vmulq_s16, self, rhs) }
}

impl SimdDivImpl for Neon {
    #[inline(always)] fn f64_div(self, rhs: Self) -> Self { self_from_op!(vdivq_f64, self, rhs) }
    #[inline(always)] fn f32_div(self, rhs: Self) -> Self { self_from_op!(vdivq_f32, self, rhs) }
}

impl SimdBitwiseImpl for Neon {
    #[inline(always)] fn and(self, rhs: Self) -> Self { self_from_op!(vandq_u32, self, rhs) }
    #[inline(always)] fn or(self, rhs: Self) -> Self { self_from_op!(vorrq_u32, self, rhs) }
    #[inline(always)] fn xor(self, rhs: Self) -> Self { self_from_op!(veorq_u32, self, rhs) }
    #[inline(always)] fn not(self) -> Self { Self(self.xor(Self::splat_32(!0u32)).0) }
    #[inline(always)] fn and_not(self, rhs: Self) -> Self { self_from_op!(vbicq_u32, self, rhs) }
}

impl SimdShiftImpl for Neon {
    #[inline(always)] fn sllv_64(self, rhs: Self) -> Self { self_from_op!(vshlq_s64, self, rhs) }
    #[inline(always)] fn srlv_64(self, rhs: Self) -> Self { self_from_op!(vshlq_u64, self, self_from_op!(vnegq_s64, rhs)) }
    #[inline(always)] fn srav_64(self, rhs: Self) -> Self { self_from_op!(vshlq_s64, self, self_from_op!(vnegq_s64, rhs)) }
    #[inline(always)] fn sllv_32(self, rhs: Self) -> Self { self_from_op!(vshlq_s32, self, rhs) }
    #[inline(always)] fn srlv_32(self, rhs: Self) -> Self { self_from_op!(vshlq_u32, self, self_from_op!(vnegq_s32, rhs)) }
    #[inline(always)] fn srav_32(self, rhs: Self) -> Self { self_from_op!(vshlq_s32, self, self_from_op!(vnegq_s32, rhs)) }
    #[inline(always)] fn sllv_16(self, rhs: Self) -> Self { self_from_op!(vshlq_s16, self, rhs) }
    #[inline(always)] fn srlv_16(self, rhs: Self) -> Self { self_from_op!(vshlq_u16, self, self_from_op!(vnegq_s16, rhs)) }
    #[inline(always)] fn srav_16(self, rhs: Self) -> Self { self_from_op!(vshlq_s16, self, self_from_op!(vnegq_s16, rhs)) }
}

impl SimdLoadImpl for Neon {
    type MaskType = Self;
    #[inline(always)] fn load_aligned<T>(ptr: *const T) -> Self { self_from_op!(vld1q_u32, ptr) }
    #[inline(always)] fn load_unaligned<T>(ptr: *const T) -> Self { self_from_op!(vld1q_u32, ptr) }
    #[inline(always)] fn masked_load_64<T>(ptr: *const T, mask: Self::MaskType) -> Self { unsafe { Self(transmute(vandq_u64(transmute(vld1q_u64(ptr as *const u64)), transmute(mask.0)))) } }
    #[inline(always)] fn masked_load_32<T>(ptr: *const T, mask: Self::MaskType) -> Self { unsafe { Self(transmute(vandq_u32(transmute(vld1q_u32(ptr as *const u32)), transmute(mask.0)))) } }
}

impl SimdStoreImpl for Neon {
    type MaskType = Self;
    #[inline(always)] fn store_aligned<T>(self, ptr: *mut T) { execute_intrinsic!(vst1q_u32, ptr, self); }
    #[inline(always)] fn store_unaligned<T>(self, ptr: *mut T) { execute_intrinsic!(vst1q_u32, ptr, self); }
    #[inline(always)] fn masked_store_64<T>(self, ptr: *mut T, mask: Self::MaskType) {
        unsafe {
            let current = vld1q_u64(ptr as *const u64);
            let masked = vbslq_u64(transmute(mask.0), transmute(self.0), current);
            vst1q_u64(ptr as *mut u64, masked);
        }
    }

    #[inline(always)] fn masked_store_32<T>(self, ptr: *mut T, mask: Self::MaskType) {
        unsafe {
            let current = vld1q_u32(ptr as *const u32);
            let masked = vbslq_u32(transmute(mask.0), transmute(self.0), current);
            vst1q_u32(ptr as *mut u32, masked);
        }
    }
}

impl SimdZeroImpl for Neon {
    #[inline(always)] fn zero() -> Self { unsafe { Self(transmute(vdupq_n_u32(0))) } }
}

impl SimdFloatCastsImpl for Neon {
    #[inline(always)] fn float_to_int_trunc(self) -> Self { self_from_op!(vcvtq_s32_f32, self) }
    #[inline(always)] fn float_to_int_round(self) -> Self { self_from_op!(vcvtnq_s32_f32, self) }
}

impl SimdIntCastsImpl for Neon {
    #[inline(always)] fn int_to_float(self) -> Self { self_from_op!(vcvtq_f32_s32, self) }
}

impl SimdPermuteImpl for Neon {
    #[inline(always)] fn permute_32(self, rhs: Self) -> Self {
        let mult = Self::splat_32(0x04040404);
        let add = Self::splat_32(0x03020100);
        let byte_indices = rhs.i32_mul(mult).i32_add(add);
        self_from_op!(vqtbl1q_u8, self, byte_indices)
    }
    #[inline(always)] fn permute_8(self, rhs: Self) -> Self { self_from_op!(vqtbl1q_u8, self, rhs) }
}

impl SimdVariableBlendImpl for Neon {
    type VecType = Self;
    #[inline(always)] fn vblend_64(self, true_values: Self, false_values: Self) -> Self { self_from_op!(vbslq_f64, self, true_values, false_values) }
    #[inline(always)] fn vblend_32(self, true_values: Self, false_values: Self) -> Self { self_from_op!(vbslq_f32, self, true_values, false_values) }
    #[inline(always)] fn vblend_8(self, true_values: Self, false_values: Self) -> Self { self_from_op!(vbslq_u8, self, true_values, false_values) }
}

impl SimdImmediateBlendImpl for Neon {
    #[inline(always)]
    fn blend_64<const N: i32>(self, false_values: Self) -> Self {
        const { assert!(N < 4, "N must be less than 4"); }
        
        const MASKS: [[u64; 2]; 4] = [
            [0, 0],
            [0xFFFFFFFFFFFFFFFF, 0],
            [0, 0xFFFFFFFFFFFFFFFF],
            [0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF],
        ];

        let mask = self_from_op!(vld1q_u64, MASKS[N as usize].as_ptr());
        mask.vblend_64(self, false_values)
    }

    #[inline(always)]
    fn blend_32<const N: i32>(self, false_values: Self) -> Self {
        const { assert!(N < 16, "N must be less than 16"); }

        const MASKS: [[u32; 4]; 16] = [
            [0, 0, 0, 0],
            [0xFFFFFFFF, 0, 0, 0],
            [0, 0xFFFFFFFF, 0, 0],
            [0xFFFFFFFF, 0xFFFFFFFF, 0, 0],
            [0, 0, 0xFFFFFFFF, 0],
            [0xFFFFFFFF, 0, 0xFFFFFFFF, 0],
            [0, 0xFFFFFFFF, 0xFFFFFFFF, 0],
            [0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0],
            [0, 0, 0, 0xFFFFFFFF],
            [0xFFFFFFFF, 0, 0, 0xFFFFFFFF],
            [0, 0xFFFFFFFF, 0, 0xFFFFFFFF],
            [0xFFFFFFFF, 0xFFFFFFFF, 0, 0xFFFFFFFF],
            [0, 0, 0xFFFFFFFF, 0xFFFFFFFF],
            [0xFFFFFFFF, 0, 0xFFFFFFFF, 0xFFFFFFFF],
            [0, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF],
            [0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF, 0xFFFFFFFF],
        ];

        let mask = self_from_op!(vld1q_u32, MASKS[N as usize].as_ptr());
        mask.vblend_32(self, false_values)
    }
}

impl SimdMulAddImpl for Neon {
    #[inline(always)] fn mul_add_f64(self, mult: Self, add: Self) -> Self { self_from_op!(vfmaq_f64, add, self, mult) }
    #[inline(always)] fn mul_sub_f64(self, mult: Self, sub: Self) -> Self { self_from_op!(vfmsq_f64, sub, self, mult).negate_f64() }
    #[inline(always)] fn negated_mul_add_f64(self, mult: Self, add: Self) -> Self { self_from_op!(vfmsq_f64, add, self, mult) }
    #[inline(always)] fn negated_mul_sub_f64(self, mult: Self, sub: Self) -> Self { self_from_op!(vfmaq_f64, sub, self, mult).negate_f64() }
    #[inline(always)] fn mul_add_f32(self, mult: Self, add: Self) -> Self { self_from_op!(vfmaq_f32, add, self, mult) }
    #[inline(always)] fn mul_sub_f32(self, mult: Self, sub: Self) -> Self { self_from_op!(vfmsq_f32, sub, self, mult).negate_f32() }
    #[inline(always)] fn negated_mul_add_f32(self, mult: Self, add: Self) -> Self { self_from_op!(vfmsq_f32, add, self, mult) }
    #[inline(always)] fn negated_mul_sub_f32(self, mult: Self, sub: Self) -> Self { self_from_op!(vfmaq_f32, sub, self, mult).negate_f32() }
}

impl SimdRoundImpl for Neon {
    #[inline(always)] fn round_f64(self) -> Self { self_from_op!(vrndnq_f64, self) }
    #[inline(always)] fn round_f32(self) -> Self { self_from_op!(vrndnq_f32, self) }
    #[inline(always)] fn floor_f64(self) -> Self { self_from_op!(vrndmq_f64, self) }
    #[inline(always)] fn floor_f32(self) -> Self { self_from_op!(vrndmq_f32, self) }
    #[inline(always)] fn ceil_f64(self) -> Self { self_from_op!(vrndpq_f64, self) }
    #[inline(always)] fn ceil_f32(self) -> Self { self_from_op!(vrndpq_f32, self) }
}

impl SimdPartialOrdImpl for Neon {
    type MaskType = Self;
    #[inline(always)] fn cmp_f64_eq(self, rhs: Self) -> Self { self_from_op!(vceqq_f64, self, rhs) }
    #[inline(always)] fn cmp_f64_lt(self, rhs: Self) -> Self { self_from_op!(vcltq_f64, self, rhs) }
    #[inline(always)] fn cmp_f64_le(self, rhs: Self) -> Self { self_from_op!(vcleq_f64, self, rhs) }
    #[inline(always)] fn cmp_f64_gt(self, rhs: Self) -> Self { self_from_op!(vcgtq_f64, self, rhs) }
    #[inline(always)] fn cmp_f64_ge(self, rhs: Self) -> Self { self_from_op!(vcgeq_f64, self, rhs) }
    #[inline(always)] fn cmp_f64_neq(self, rhs: Self) -> Self { Self(self.cmp_f64_eq(rhs).not().0) }
    #[inline(always)] fn cmp_f32_eq(self, rhs: Self) -> Self { self_from_op!(vceqq_f32, self, rhs) }
    #[inline(always)] fn cmp_f32_lt(self, rhs: Self) -> Self { self_from_op!(vcltq_f32, self, rhs) }
    #[inline(always)] fn cmp_f32_le(self, rhs: Self) -> Self { self_from_op!(vcleq_f32, self, rhs) }
    #[inline(always)] fn cmp_f32_gt(self, rhs: Self) -> Self { self_from_op!(vcgtq_f32, self, rhs) }
    #[inline(always)] fn cmp_f32_ge(self, rhs: Self) -> Self { self_from_op!(vcgeq_f32, self, rhs) }
    #[inline(always)] fn cmp_f32_neq(self, rhs: Self) -> Self { Self(self.cmp_f32_eq(rhs).not().0) }
    #[inline(always)] fn cmp_i64_eq(self, rhs: Self) -> Self { self_from_op!(vceqq_s64, self, rhs) }
    #[inline(always)] fn cmp_i64_gt(self, rhs: Self) -> Self { self_from_op!(vcgtq_s64, self, rhs) }
    #[inline(always)] fn cmp_i32_eq(self, rhs: Self) -> Self { self_from_op!(vceqq_s32, self, rhs) }
    #[inline(always)] fn cmp_i32_gt(self, rhs: Self) -> Self { self_from_op!(vcgtq_s32, self, rhs) }
    #[inline(always)] fn cmp_i16_eq(self, rhs: Self) -> Self { self_from_op!(vceqq_s16, self, rhs) }
    #[inline(always)] fn cmp_i16_gt(self, rhs: Self) -> Self { self_from_op!(vcgtq_s16, self, rhs) }
    #[inline(always)] fn cmp_i8_eq(self, rhs: Self) -> Self { self_from_op!(vceqq_s8, self, rhs) }
    #[inline(always)] fn cmp_i8_gt(self, rhs: Self) -> Self { self_from_op!(vcgtq_s8, self, rhs) }

    #[inline(always)] fn max_f64(self, rhs: Self) -> Self { self_from_op!(vmaxq_f64, self, rhs) }
    #[inline(always)] fn min_f64(self, rhs: Self) -> Self { self_from_op!(vminq_f64, self, rhs) }
    #[inline(always)] fn max_f32(self, rhs: Self) -> Self { self_from_op!(vmaxq_f32, self, rhs) }
    #[inline(always)] fn min_f32(self, rhs: Self) -> Self { self_from_op!(vminq_f32, self, rhs) }
    #[inline(always)] fn max_i32(self, rhs: Self) -> Self { self_from_op!(vmaxq_s32, self, rhs) }
    #[inline(always)] fn min_i32(self, rhs: Self) -> Self { self_from_op!(vminq_s32, self, rhs) }
    #[inline(always)] fn max_i16(self, rhs: Self) -> Self { self_from_op!(vmaxq_s16, self, rhs) }
    #[inline(always)] fn min_i16(self, rhs: Self) -> Self { self_from_op!(vminq_s16, self, rhs) }
    #[inline(always)] fn max_i8(self, rhs: Self) -> Self { self_from_op!(vmaxq_s8, self, rhs) }
    #[inline(always)] fn min_i8(self, rhs: Self) -> Self { self_from_op!(vminq_s8, self, rhs) }
    #[inline(always)] fn max_u32(self, rhs: Self) -> Self { self_from_op!(vmaxq_u32, self, rhs) }
    #[inline(always)] fn min_u32(self, rhs: Self) -> Self { self_from_op!(vminq_u32, self, rhs) }
    #[inline(always)] fn max_u16(self, rhs: Self) -> Self { self_from_op!(vmaxq_u16, self, rhs) }
    #[inline(always)] fn min_u16(self, rhs: Self) -> Self { self_from_op!(vminq_u16, self, rhs) }
    #[inline(always)] fn max_u8(self, rhs: Self) -> Self { self_from_op!(vmaxq_u8, self, rhs) }
    #[inline(always)] fn min_u8(self, rhs: Self) -> Self { self_from_op!(vminq_u8, self, rhs) }
}

impl SimdSplatImpl for Neon {
    #[inline(always)] fn splat_64<T>(val: T) -> Self { self_from_op!(vdupq_n_s64, val) }
    #[inline(always)] fn splat_32<T>(val: T) -> Self { self_from_op!(vdupq_n_s32, val) }
    #[inline(always)] fn splat_16<T>(val: T) -> Self { self_from_op!(vdupq_n_s16, val) }
    #[inline(always)] fn splat_8<T>(val: T) -> Self { self_from_op!(vdupq_n_s8, val) }
}

impl SimdGatherImpl for Neon {
    fn gather_32_from_32<T, const B: i32>(self, ptr: *const T) -> Self {
        unsafe {
            let ptr_32 = ptr as *const u32;
            let mut temp = [0u32; 4];
            self.store_unaligned(temp.as_mut_ptr());
            for i in 0..4 {
                temp[i] = *ptr_32.add(temp[i] as usize);
            }
            Self::load_unaligned(temp.as_ptr())
        }
    }

    #[inline(always)] fn gather_64_from_64<T, const B: i32>(self, ptr: *const T) -> Self { 
        unsafe {
            let ptr_64 = ptr as *const u64;
            let mut temp = [0u64; 2];
            self.store_unaligned(temp.as_mut_ptr());
            for i in 0..2 {
                temp[i] = *ptr_64.add(temp[i] as usize);
            }
            Self::load_unaligned(temp.as_ptr())
        }
    }
}

impl SimdSqrtImpl for Neon {
    #[inline(always)] fn sqrt_f64(self) -> Self { self_from_op!(vsqrtq_f64, self) }
    #[inline(always)] fn sqrt_f32(self) -> Self { self_from_op!(vsqrtq_f32, self) }
    #[inline(always)] fn rsqrt_f32(self) -> Self { self_from_op!(vrsqrteq_f32, self) }
}

impl SimdAllBitsImpl for Neon {
    #[inline(always)]
    fn all_zero(self) -> bool { unsafe {
        vmaxvq_u8(transmute_copy(&self.0)) == 0
    }}
}

impl SimdNegateImpl for Neon {
    #[inline(always)] fn negate_f64(self) -> Self { self_from_op!(vnegq_f64, self) }
    #[inline(always)] fn negate_f32(self) -> Self { self_from_op!(vnegq_f32, self) }
}

impl SimdBlockShiftImpl for Neon {
    // Stabilize const generic expr plssssss.
    #[inline(always)]
    fn block_left_byte_shift<const N: i32>(self) -> Self {
        match N {
            0 => self,
            1 => self_from_const_op!(vextq_u8, 15, Self::zero(), self),
            2 => self_from_const_op!(vextq_u8, 14, Self::zero(), self),
            3 => self_from_const_op!(vextq_u8, 13, Self::zero(), self),
            4 => self_from_const_op!(vextq_u8, 12, Self::zero(), self),
            5 => self_from_const_op!(vextq_u8, 11, Self::zero(), self),
            6 => self_from_const_op!(vextq_u8, 10, Self::zero(), self),
            7 => self_from_const_op!(vextq_u8, 9, Self::zero(), self),
            8 => self_from_const_op!(vextq_u8, 8, Self::zero(), self),
            9 => self_from_const_op!(vextq_u8, 7, Self::zero(), self),
            10 => self_from_const_op!(vextq_u8, 6, Self::zero(), self),
            11 => self_from_const_op!(vextq_u8, 5, Self::zero(), self),
            12 => self_from_const_op!(vextq_u8, 4, Self::zero(), self),
            13 => self_from_const_op!(vextq_u8, 3, Self::zero(), self),
            14 => self_from_const_op!(vextq_u8, 2, Self::zero(), self),
            15 => self_from_const_op!(vextq_u8, 1, Self::zero(), self),
            16 => Self::zero(),
            _ => unreachable!(),
        }
    }
    #[inline(always)] fn block_right_byte_shift<const N: i32>(self) -> Self {
        self_from_const_op!(vextq_u8, N, self, Self::zero())
    }
}

// TODO: Add to_bits for other bit sizes for NEON.
impl SimdMaskBitConversion for Neon {
    #[inline(always)] fn to_bits_64(self) -> u64 {
        0u64 
    }
    #[inline(always)] fn to_bits_32(self) -> u64 {
        unsafe {
            let single_bits = vshrq_n_u32::<31>(transmute(self.0));
            let iota = vld1q_u32([0, 1, 2, 3].as_ptr());
            let shifted = vshlq_u32(single_bits, transmute(iota));
            vaddvq_u32(shifted).into()
        }
    }
    #[inline(always)] fn to_bits_8(self) -> u64 {
        0u64
    }
}

impl SimdLaneShiftImpl for Neon {
    #[inline(always)] fn left_lane_shift_32<const N: i32>(self) -> Self {
        match N {
            0 => self,
            1 => self_from_const_op!(vextq_u8, 4, Self::zero(), self),
            2 => self_from_const_op!(vextq_u8, 8, Self::zero(), self),
            3 => self_from_const_op!(vextq_u8, 12, Self::zero(), self),
            _ => Self::zero()
        }
    }
    #[inline(always)] fn right_lane_shift_32<const N: i32>(self) -> Self {
        match N {
            0 => self,
            1 => self_from_const_op!(vextq_u8, 12, self, Self::zero()),
            2 => self_from_const_op!(vextq_u8, 8, self, Self::zero()),
            3 => self_from_const_op!(vextq_u8, 4, self, Self::zero()),
            _ => Self::zero()
        }
    }
}