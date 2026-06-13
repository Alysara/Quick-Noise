use std::arch::x86_64::*;
use std::mem::{transmute, transmute_copy};

use crate::simd::architectures::arch_impl::*;
use crate::simd::architectures::macros::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Avx512Reg(pub __m512i);
impl SimdArch for Avx512Reg {}

#[derive(Copy, Clone)]
pub struct Avx512Mask(pub __mmask64);
impl MaskArch for Avx512Mask {}

impl SimdAddImpl for Avx512Reg {
    #[inline(always)]
    fn f64_add(self, rhs: Self) -> Self {
        self_from_op!(_mm512_add_pd, self, rhs)
    }
    #[inline(always)]
    fn f32_add(self, rhs: Self) -> Self {
        self_from_op!(_mm512_add_ps, self, rhs)
    }
    #[inline(always)]
    fn i64_add(self, rhs: Self) -> Self {
        self_from_op!(_mm512_add_epi64, self, rhs)
    }
    #[inline(always)]
    fn i32_add(self, rhs: Self) -> Self {
        self_from_op!(_mm512_add_epi32, self, rhs)
    }
    #[inline(always)]
    fn i16_add(self, rhs: Self) -> Self {
        self_from_op!(_mm512_add_epi16, self, rhs)
    }
    #[inline(always)]
    fn i8_add(self, rhs: Self) -> Self {
        self_from_op!(_mm512_add_epi8, self, rhs)
    }
}

impl SimdSubImpl for Avx512Reg {
    #[inline(always)]
    fn f64_sub(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sub_pd, self, rhs)
    }
    #[inline(always)]
    fn f32_sub(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sub_ps, self, rhs)
    }
    #[inline(always)]
    fn i64_sub(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sub_epi64, self, rhs)
    }
    #[inline(always)]
    fn i32_sub(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sub_epi32, self, rhs)
    }
    #[inline(always)]
    fn i16_sub(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sub_epi16, self, rhs)
    }
    #[inline(always)]
    fn i8_sub(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sub_epi8, self, rhs)
    }
}

impl SimdMulImpl for Avx512Reg {
    #[inline(always)]
    fn f64_mul(self, rhs: Self) -> Self {
        self_from_op!(_mm512_mul_pd, self, rhs)
    }
    #[inline(always)]
    fn f32_mul(self, rhs: Self) -> Self {
        self_from_op!(_mm512_mul_ps, self, rhs)
    }
    #[inline(always)]
    fn i32_mul(self, rhs: Self) -> Self {
        self_from_op!(_mm512_mullo_epi32, self, rhs)
    }
    #[inline(always)]
    fn i16_mul(self, rhs: Self) -> Self {
        self_from_op!(_mm512_mullo_epi16, self, rhs)
    }
}

impl SimdDivImpl for Avx512Reg {
    #[inline(always)]
    fn f64_div(self, rhs: Self) -> Self {
        self_from_op!(_mm512_div_pd, self, rhs)
    }
    #[inline(always)]
    fn f32_div(self, rhs: Self) -> Self {
        self_from_op!(_mm512_div_ps, self, rhs)
    }
}

impl SimdBitwiseImpl for Avx512Reg {
    #[inline(always)]
    fn and(self, rhs: Self) -> Self {
        self_from_op!(_mm512_and_si512, self, rhs)
    }
    #[inline(always)]
    fn or(self, rhs: Self) -> Self {
        self_from_op!(_mm512_or_si512, self, rhs)
    }
    #[inline(always)]
    fn xor(self, rhs: Self) -> Self {
        self_from_op!(_mm512_xor_si512, self, rhs)
    }
    #[inline(always)]
    fn not(self) -> Self {
        Self(self.xor(Self::splat_32(!0)).0)
    }
    #[inline(always)]
    fn and_not(self, rhs: Self) -> Self {
        self_from_op!(_mm512_andnot_si512, rhs, self)
    }
}

impl SimdShiftImpl for Avx512Reg {
    #[inline(always)]
    fn sllv_64(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sllv_epi64, self, rhs)
    }
    #[inline(always)]
    fn srlv_64(self, rhs: Self) -> Self {
        self_from_op!(_mm512_srlv_epi64, self, rhs)
    }
    #[inline(always)]
    fn srav_64(self, rhs: Self) -> Self {
        self_from_op!(_mm512_srav_epi64, self, rhs)
    }
    #[inline(always)]
    fn sllv_32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sllv_epi32, self, rhs)
    }
    #[inline(always)]
    fn srlv_32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_srlv_epi32, self, rhs)
    }
    #[inline(always)]
    fn srav_32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_srav_epi32, self, rhs)
    }
    #[inline(always)]
    fn sllv_16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_sllv_epi16, self, rhs)
    }
    #[inline(always)]
    fn srlv_16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_srlv_epi16, self, rhs)
    }
    #[inline(always)]
    fn srav_16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_srav_epi16, self, rhs)
    }
}

impl SimdLoadImpl for Avx512Reg {
    type MaskType = Avx512Mask;
    #[inline(always)]
    fn load_aligned<T>(ptr: *const T) -> Self {
        self_from_op!(_mm512_load_si512, ptr)
    }
    #[inline(always)]
    fn load_unaligned<T>(ptr: *const T) -> Self {
        self_from_op!(_mm512_loadu_si512, ptr)
    }
    #[inline(always)]
    fn masked_load_64<T>(ptr: *const T, mask: Self::MaskType) -> Self {
        self_from_op!(_mm512_mask_load_epi64, Self::zero(), ptr, mask)
    }
    #[inline(always)]
    fn masked_load_32<T>(ptr: *const T, mask: Self::MaskType) -> Self {
        self_from_op!(_mm512_mask_load_epi32, Self::zero(), ptr, mask)
    }
}

impl SimdStoreImpl for Avx512Reg {
    type MaskType = Avx512Mask;
    #[inline(always)]
    fn store_aligned<T>(self, ptr: *mut T) {
        execute_intrinsic!(_mm512_store_si512, ptr, self);
    }
    #[inline(always)]
    fn store_unaligned<T>(self, ptr: *mut T) {
        execute_intrinsic!(_mm512_storeu_si512, ptr, self);
    }
    #[inline(always)]
    fn masked_store_64<T>(self, ptr: *mut T, mask: Self::MaskType) {
        execute_intrinsic!(_mm512_mask_store_epi64, ptr, mask, self);
    }
    #[inline(always)]
    fn masked_store_32<T>(self, ptr: *mut T, mask: Self::MaskType) {
        execute_intrinsic!(_mm512_mask_store_epi32, ptr, mask, self);
    }
}

impl SimdZeroImpl for Avx512Reg {
    #[inline(always)]
    fn zero() -> Self {
        self_from_op!(_mm512_setzero_si512,)
    }
}

impl SimdFloatCastsImpl for Avx512Reg {
    #[inline(always)]
    fn float_to_int_trunc(self) -> Self {
        self_from_op!(_mm512_cvttps_epi32, self)
    }
    #[inline(always)]
    fn float_to_int_round(self) -> Self {
        self_from_op!(_mm512_cvtps_epi32, self)
    }
}

impl SimdIntCastsImpl for Avx512Reg {
    #[inline(always)]
    fn int_to_float(self) -> Self {
        self_from_op!(_mm512_cvtepi32_ps, self)
    }
}

impl SimdPermuteImpl for Avx512Reg {
    // type BlockVec = Sse;
    #[inline(always)]
    fn permute_32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_permutexvar_epi32, self, rhs)
    }
    #[inline(always)]
    fn permute_8(self, rhs: Self) -> Self {
        self_from_op!(_mm512_shuffle_epi8, self, rhs)
    }
}

impl SimdVariableBlendImpl for Avx512Mask {
    type VecType = Avx512Reg;
    #[inline(always)]
    fn vblend_64(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType {
        unsafe {
            Avx512Reg(transmute(execute_intrinsic!(
                _mm512_mask_blend_pd,
                self,
                false_values,
                true_values
            )))
        }
    }
    #[inline(always)]
    fn vblend_32(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType {
        unsafe {
            Avx512Reg(transmute(execute_intrinsic!(
                _mm512_mask_blend_ps,
                self,
                false_values,
                true_values
            )))
        }
    }
    #[inline(always)]
    fn vblend_8(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType {
        unsafe {
            Avx512Reg(transmute(execute_intrinsic!(
                _mm512_mask_blend_epi8,
                self,
                false_values,
                true_values
            )))
        }
    }
}

impl SimdImmediateBlendImpl for Avx512Reg {
    #[inline(always)]
    fn blend_64<const N: i32>(self, false_values: Self) -> Self {
        let mask = Avx512Mask(N as u64);
        mask.vblend_64(self, false_values)
    }
    #[inline(always)]
    fn blend_32<const N: i32>(self, false_values: Self) -> Self {
        let mask = Avx512Mask(N as u64);
        mask.vblend_32(self, false_values)
    }
}

impl SimdMulAddImpl for Avx512Reg {
    #[inline(always)]
    fn mul_add_f64(self, mult: Self, add: Self) -> Self {
        self_from_op!(_mm512_fmadd_pd, self, mult, add)
    }
    #[inline(always)]
    fn mul_sub_f64(self, mult: Self, sub: Self) -> Self {
        self_from_op!(_mm512_fmsub_pd, self, mult, sub)
    }
    #[inline(always)]
    fn negated_mul_add_f64(self, mult: Self, add: Self) -> Self {
        self_from_op!(_mm512_fnmadd_pd, self, mult, add)
    }
    #[inline(always)]
    fn negated_mul_sub_f64(self, mult: Self, sub: Self) -> Self {
        self_from_op!(_mm512_fnmsub_pd, self, mult, sub)
    }
    #[inline(always)]
    fn mul_add_f32(self, mult: Self, add: Self) -> Self {
        self_from_op!(_mm512_fmadd_ps, self, mult, add)
    }
    #[inline(always)]
    fn mul_sub_f32(self, mult: Self, sub: Self) -> Self {
        self_from_op!(_mm512_fmsub_ps, self, mult, sub)
    }
    #[inline(always)]
    fn negated_mul_add_f32(self, mult: Self, add: Self) -> Self {
        self_from_op!(_mm512_fnmadd_ps, self, mult, add)
    }
    #[inline(always)]
    fn negated_mul_sub_f32(self, mult: Self, sub: Self) -> Self {
        self_from_op!(_mm512_fnmsub_ps, self, mult, sub)
    }
}

impl SimdRoundImpl for Avx512Reg {
    #[inline(always)]
    fn round_f64(self) -> Self {
        self_from_const_op!(
            _mm512_roundscale_pd,
            _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC,
            self
        )
    }
    #[inline(always)]
    fn round_f32(self) -> Self {
        self_from_const_op!(
            _mm512_roundscale_ps,
            _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC,
            self
        )
    }
    #[inline(always)]
    fn floor_f64(self) -> Self {
        self_from_const_op!(
            _mm512_roundscale_pd,
            _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC,
            self
        )
    }
    #[inline(always)]
    fn floor_f32(self) -> Self {
        self_from_const_op!(
            _mm512_roundscale_ps,
            _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC,
            self
        )
    }
    #[inline(always)]
    fn ceil_f64(self) -> Self {
        self_from_const_op!(
            _mm512_roundscale_pd,
            _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC,
            self
        )
    }
    #[inline(always)]
    fn ceil_f32(self) -> Self {
        self_from_const_op!(
            _mm512_roundscale_ps,
            _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC,
            self
        )
    }
}

impl SimdPartialOrdImpl for Avx512Reg {
    type MaskType = Avx512Mask;
    #[inline(always)]
    fn cmp_f64_eq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_pd_mask, _CMP_EQ_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f64_lt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_pd_mask, _CMP_LT_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f64_le(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_pd_mask, _CMP_LE_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f64_gt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_pd_mask, _CMP_GT_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f64_ge(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_pd_mask, _CMP_GE_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f64_neq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_pd_mask, _CMP_NEQ_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f32_eq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_ps_mask, _CMP_EQ_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f32_lt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_ps_mask, _CMP_LT_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f32_le(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_ps_mask, _CMP_LE_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f32_gt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_ps_mask, _CMP_GT_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f32_ge(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_ps_mask, _CMP_GE_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_f32_neq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_const_intrinsic!(_mm512_cmp_ps_mask, _CMP_NEQ_OQ, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i64_eq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpeq_epi64_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i64_gt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpgt_epi64_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i32_eq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpeq_epi32_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i32_gt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpgt_epi32_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i16_eq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpeq_epi16_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i16_gt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpgt_epi16_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i8_eq(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpeq_epi8_mask, self, rhs) as u64)
    }
    #[inline(always)]
    fn cmp_i8_gt(self, rhs: Self) -> Self::MaskType {
        Avx512Mask(execute_intrinsic!(_mm512_cmpgt_epi8_mask, self, rhs) as u64)
    }

    #[inline(always)]
    fn max_f64(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_pd, self, rhs)
    }
    #[inline(always)]
    fn min_f64(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_pd, self, rhs)
    }
    #[inline(always)]
    fn max_f32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_ps, self, rhs)
    }
    #[inline(always)]
    fn min_f32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_ps, self, rhs)
    }
    #[inline(always)]
    fn max_i32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_epi32, self, rhs)
    }
    #[inline(always)]
    fn min_i32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_epi32, self, rhs)
    }
    #[inline(always)]
    fn max_i16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_epi16, self, rhs)
    }
    #[inline(always)]
    fn min_i16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_epi16, self, rhs)
    }
    #[inline(always)]
    fn max_i8(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_epi8, self, rhs)
    }
    #[inline(always)]
    fn min_i8(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_epi8, self, rhs)
    }
    #[inline(always)]
    fn max_u32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_epu32, self, rhs)
    }
    #[inline(always)]
    fn min_u32(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_epu32, self, rhs)
    }
    #[inline(always)]
    fn max_u16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_epu16, self, rhs)
    }
    #[inline(always)]
    fn min_u16(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_epu16, self, rhs)
    }
    #[inline(always)]
    fn max_u8(self, rhs: Self) -> Self {
        self_from_op!(_mm512_max_epu8, self, rhs)
    }
    #[inline(always)]
    fn min_u8(self, rhs: Self) -> Self {
        self_from_op!(_mm512_min_epu8, self, rhs)
    }
}

// TODO: Make a custom trait for handling this transmutation into i*.
impl SimdSplatImpl for Avx512Reg {
    #[inline(always)]
    fn splat_64<T>(val: T) -> Self {
        self_from_op!(_mm512_set1_epi64, val)
    }
    #[inline(always)]
    fn splat_32<T>(val: T) -> Self {
        self_from_op!(_mm512_set1_epi32, val)
    }
    #[inline(always)]
    fn splat_16<T>(val: T) -> Self {
        self_from_op!(_mm512_set1_epi16, val)
    }
    #[inline(always)]
    fn splat_8<T>(val: T) -> Self {
        self_from_op!(_mm512_set1_epi8, val)
    }
}

impl SimdBitwiseImpl for Avx512Mask {
    #[inline(always)]
    fn and(self, rhs: Self) -> Self {
        self_from_op!(_kand_mask64, self, rhs)
    }
    #[inline(always)]
    fn or(self, rhs: Self) -> Self {
        self_from_op!(_kor_mask64, self, rhs)
    }
    #[inline(always)]
    fn xor(self, rhs: Self) -> Self {
        self_from_op!(_kxor_mask64, self, rhs)
    }
    #[inline(always)]
    fn not(self) -> Self {
        self_from_op!(_knot_mask64, self)
    }
    #[inline(always)]
    fn and_not(self, rhs: Self) -> Self {
        self_from_op!(_kandn_mask64, rhs, self)
    }
}

impl SimdGatherImpl for Avx512Reg {
    #[inline(always)]
    fn gather_32_from_32<T, const B: i32>(self, ptr: *const T) -> Self {
        self_from_const_op!(_mm512_i32gather_epi32, B, self, ptr)
    }
    // #[inline(always)] fn gather_64_from_32<T, const B: i32>(self, ptr: *const T) -> Self { self_from_const_op!(_mm512_i32gather_epi64, B, ptr, self) }
    // #[inline(always)] fn gather_32_from_64<T, const B: i32>(self, ptr: *const T) -> Self { self_from_const_op!(_mm512_i64gather_epi32, B, ptr, self) }
    #[inline(always)]
    fn gather_64_from_64<T, const B: i32>(self, ptr: *const T) -> Self {
        self_from_const_op!(_mm512_i64gather_epi64, B, self, ptr)
    }
}

impl SimdSqrtImpl for Avx512Reg {
    #[inline(always)]
    fn sqrt_f64(self) -> Self {
        self_from_op!(_mm512_sqrt_pd, self)
    }
    #[inline(always)]
    fn sqrt_f32(self) -> Self {
        self_from_op!(_mm512_sqrt_ps, self)
    }
    #[inline(always)]
    fn rsqrt_f32(self) -> Self {
        self_from_op!(_mm512_rsqrt14_ps, self)
    }
}

impl SimdAllBitsImpl for Avx512Mask {
    #[inline(always)]
    fn all_zero(self) -> bool {
        self.0 == 0
    }
}

impl SimdNegateImpl for Avx512Reg {
    #[inline(always)]
    fn negate_f64(self) -> Self {
        Self::splat_64(-0.0f64).xor(self)
    }
    #[inline(always)]
    fn negate_f32(self) -> Self {
        Self::splat_32(-0.0f64).xor(self)
    }
}

impl SimdBlockShiftImpl for Avx512Reg {
    #[inline(always)]
    fn block_left_byte_shift<const N: i32>(self) -> Self {
        self_from_const_op!(_mm512_bslli_epi128, N, self)
    }
    #[inline(always)]
    fn block_right_byte_shift<const N: i32>(self) -> Self {
        self_from_const_op!(_mm512_bsrli_epi128, N, self)
    }
}

impl SimdMaskBitConversion for Avx512Mask {
    #[inline(always)]
    fn to_bits_64(self) -> u64 {
        self.0 as u64
    }
    #[inline(always)]
    fn to_bits_32(self) -> u64 {
        self.0 as u64
    }
    #[inline(always)]
    fn to_bits_8(self) -> u64 {
        self.0 as u64
    }
    #[inline(always)]
    fn from_bits_64(bitmask: u64) -> Self {
        unsafe { transmute(bitmask) }
    }
    #[inline(always)]
    fn from_bits_32(bitmask: u64) -> Self {
        unsafe { transmute(bitmask) }
    }
    #[inline(always)]
    fn from_bits_16(bitmask: u64) -> Self {
        unsafe { transmute(bitmask) }
    }
    #[inline(always)]
    fn from_bits_8(bitmask: u64) -> Self {
        unsafe { transmute(bitmask) }
    }
}

impl SimdLaneShiftImpl for Avx512Reg {
    #[inline(always)]
    fn left_lane_shift_32(self, n: u32) -> Self {
        match n {
            0 => self,
            1 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 1) }),
            2 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 2) }),
            3 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 3) }),
            4 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 4) }),
            5 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 5) }),
            6 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 6) }),
            7 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 7) }),
            8 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 8) }),
            9 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 9) }),
            10 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 10) }),
            11 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 11) }),
            12 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 12) }),
            13 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 13) }),
            14 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 14) }),
            15 => Self(unsafe { _mm512_alignr_epi32(self.0, _mm512_setzero_si512(), 15) }),
            _ => Self::zero(),
        }
    }

    #[inline(always)]
    fn right_lane_shift_32(self, n: u32) -> Self {
        match n {
            0 => self,
            1 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 1) }),
            2 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 2) }),
            3 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 3) }),
            4 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 4) }),
            5 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 5) }),
            6 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 6) }),
            7 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 7) }),
            8 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 8) }),
            9 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 9) }),
            10 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 10) }),
            11 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 11) }),
            12 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 12) }),
            13 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 13) }),
            14 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 14) }),
            15 => Self(unsafe { _mm512_alignr_epi32(_mm512_setzero_si512(), self.0, 15) }),
            _ => Self::zero(),
        }
    }
}

