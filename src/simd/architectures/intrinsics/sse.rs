use std::arch::x86_64::*;
use crate::simd::{architectures::arch_impl::*, simd_mask::core::SimdMask};
use std::mem::{transmute, transmute_copy};
use crate::simd::architectures::macros::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Sse(pub __m128i);
impl SimdArch for Sse {}
impl MaskArch for Sse {}

impl SimdAddImpl for Sse {
    #[inline(always)] fn f64_add(self, rhs: Self) -> Self { self_from_op!(_mm_add_pd, self, rhs) }
    #[inline(always)] fn f32_add(self, rhs: Self) -> Self { self_from_op!(_mm_add_ps, self, rhs) }
    #[inline(always)] fn i64_add(self, rhs: Self) -> Self { self_from_op!(_mm_add_epi64, self, rhs) }
    #[inline(always)] fn i32_add(self, rhs: Self) -> Self { self_from_op!(_mm_add_epi32, self, rhs) }
    #[inline(always)] fn i16_add(self, rhs: Self) -> Self { self_from_op!(_mm_add_epi16, self, rhs) }
    #[inline(always)] fn i8_add(self, rhs: Self) -> Self { self_from_op!(_mm_add_epi8, self, rhs) }
}

impl SimdSubImpl for Sse {
    #[inline(always)] fn f64_sub(self, rhs: Self) -> Self { self_from_op!(_mm_sub_pd, self, rhs) }
    #[inline(always)] fn f32_sub(self, rhs: Self) -> Self { self_from_op!(_mm_sub_ps, self, rhs) }
    #[inline(always)] fn i64_sub(self, rhs: Self) -> Self { self_from_op!(_mm_sub_epi64, self, rhs) }
    #[inline(always)] fn i32_sub(self, rhs: Self) -> Self { self_from_op!(_mm_sub_epi32, self, rhs) }
    #[inline(always)] fn i16_sub(self, rhs: Self) -> Self { self_from_op!(_mm_sub_epi16, self, rhs) }
    #[inline(always)] fn i8_sub(self, rhs: Self) -> Self { self_from_op!(_mm_sub_epi8, self, rhs) }
}

impl SimdMulImpl for Sse {
    #[inline(always)] fn f64_mul(self, rhs: Self) -> Self { self_from_op!(_mm_mul_pd, self, rhs) }
    #[inline(always)] fn f32_mul(self, rhs: Self) -> Self { self_from_op!(_mm_mul_ps, self, rhs) }
    #[inline(always)] fn i32_mul(self, rhs: Self) -> Self { self_from_op!(_mm_mullo_epi32, self, rhs) }
    #[inline(always)] fn i16_mul(self, rhs: Self) -> Self { self_from_op!(_mm_mullo_epi16, self, rhs) }
}

impl SimdDivImpl for Sse {
    #[inline(always)] fn f64_div(self, rhs: Self) -> Self { self_from_op!(_mm_div_pd, self, rhs) }
    #[inline(always)] fn f32_div(self, rhs: Self) -> Self { self_from_op!(_mm_div_ps, self, rhs) }
}

impl SimdBitwiseImpl for Sse {
    #[inline(always)] fn and(self, rhs: Self) -> Self { self_from_op!(_mm_and_si128, self, rhs) }
    #[inline(always)] fn or(self, rhs: Self) -> Self { self_from_op!(_mm_or_si128, self, rhs) }
    #[inline(always)] fn xor(self, rhs: Self) -> Self { self_from_op!(_mm_xor_si128, self, rhs) }
    #[inline(always)] fn not(self) -> Self { Self(self.xor(Self::splat_32(!0)).0) }
    #[inline(always)] fn and_not(self, rhs: Self) -> Self { self_from_op!(_mm_andnot_si128, rhs, self) }
}

impl SimdShiftImpl for Sse {
    #[inline(always)] fn sllv_64(self, rhs: Self) -> Self { self_from_op!(_mm_sllv_epi64, self, rhs) }
    #[inline(always)] fn srlv_64(self, rhs: Self) -> Self { self_from_op!(_mm_srlv_epi64, self, rhs) }
    #[inline(always)] fn srav_64(self, rhs: Self) -> Self { self_from_op!(_mm_srav_epi64, self, rhs) }
    #[inline(always)] fn sllv_32(self, rhs: Self) -> Self { self_from_op!(_mm_sllv_epi32, self, rhs) }
    #[inline(always)] fn srlv_32(self, rhs: Self) -> Self { self_from_op!(_mm_srlv_epi32, self, rhs) }
    #[inline(always)] fn srav_32(self, rhs: Self) -> Self { self_from_op!(_mm_srav_epi32, self, rhs) }
    #[inline(always)] fn sllv_16(self, rhs: Self) -> Self { self_from_op!(_mm_sllv_epi16, self, rhs) }
    #[inline(always)] fn srlv_16(self, rhs: Self) -> Self { self_from_op!(_mm_srlv_epi16, self, rhs) }
    #[inline(always)] fn srav_16(self, rhs: Self) -> Self { self_from_op!(_mm_srav_epi16, self, rhs) }
}

impl SimdLoadImpl for Sse {
    type MaskType = Self;
    #[inline(always)] fn load_aligned<T>(ptr: *const T) -> Self { self_from_op!(_mm_load_si128, ptr) }
    #[inline(always)] fn load_unaligned<T>(ptr: *const T) -> Self { self_from_op!(_mm_loadu_si128, ptr) }
    #[inline(always)] fn masked_load_64<T>(ptr: *const T, mask: Self::MaskType) -> Self { self_from_op!(_mm_maskload_epi64, ptr, mask) }
    #[inline(always)] fn masked_load_32<T>(ptr: *const T, mask: Self::MaskType) -> Self { self_from_op!(_mm_maskload_epi32, ptr, mask) }
}

impl SimdStoreImpl for Sse {
    type MaskType = Self;
    #[inline(always)] fn store_aligned<T>(self, ptr: *mut T) { execute_intrinsic!(_mm_store_si128, ptr, self); }
    #[inline(always)] fn store_unaligned<T>(self, ptr: *mut T) { execute_intrinsic!(_mm_storeu_si128, ptr, self); }
    #[inline(always)] fn masked_store_64<T>(self, ptr: *mut T, mask: Self::MaskType) { execute_intrinsic!(_mm_maskstore_epi64, ptr, mask, self); }
    #[inline(always)] fn masked_store_32<T>(self, ptr: *mut T, mask: Self::MaskType) { execute_intrinsic!(_mm_maskstore_epi32, ptr, mask, self); }
}

impl SimdZeroImpl for Sse {
    #[inline(always)] fn zero() -> Self { self_from_op!(_mm_setzero_si128,) }
}

impl SimdFloatCastsImpl for Sse {
    #[inline(always)] fn float_to_int_trunc(self) -> Self { self_from_op!(_mm_cvttps_epi32, self) }
    #[inline(always)] fn float_to_int_round(self) -> Self { self_from_op!(_mm_cvtps_epi32, self) }
}

impl SimdIntCastsImpl for Sse {
    #[inline(always)] fn int_to_float(self) -> Self { self_from_op!(_mm_cvtepi32_ps, self) }
}

impl SimdPermuteImpl for Sse {
    #[inline(always)] fn permute_32(self, rhs: Self) -> Self { self_from_op!(_mm_permutevar_ps, self, rhs) }
    #[inline(always)] fn permute_8(self, rhs: Self) -> Self { self_from_op!(_mm_shuffle_epi8, self, rhs) }
}

impl SimdVariableBlendImpl for Sse {
    type VecType = Self;
    #[inline(always)] fn vblend_64(self, true_values: Self::VecType, false_values: Self::VecType) -> Self { self_from_op!(_mm_blendv_pd, false_values, true_values, self) }
    #[inline(always)] fn vblend_32(self, true_values: Self::VecType, false_values: Self::VecType) -> Self { self_from_op!(_mm_blendv_ps, false_values, true_values, self) }
    #[inline(always)] fn vblend_8(self, true_values: Self::VecType, false_values: Self::VecType) -> Self { self_from_op!(_mm_blendv_epi8, false_values, true_values, self) }
}

impl SimdImmediateBlendImpl for Sse {
    #[inline(always)] fn blend_64<const N: i32>(self, false_values: Self) -> Self { self_from_const_op!(_mm_blend_pd, N, false_values, self) }
    #[inline(always)] fn blend_32<const N: i32>(self, false_values: Self) -> Self { self_from_const_op!(_mm_blend_ps, N, false_values, self) }
}

impl SimdMulAddImpl for Sse {
    #[inline(always)] fn mul_add_f64(self, mult: Self, add: Self) -> Self { self_from_op!(_mm_fmadd_pd, self, mult, add) }
    #[inline(always)] fn mul_sub_f64(self, mult: Self, sub: Self) -> Self { self_from_op!(_mm_fmsub_pd, self, mult, sub) }
    #[inline(always)] fn negated_mul_add_f64(self, mult: Self, add: Self) -> Self { self_from_op!(_mm_fnmadd_pd, self, mult, add) }
    #[inline(always)] fn negated_mul_sub_f64(self, mult: Self, sub: Self) -> Self { self_from_op!(_mm_fnmsub_pd, self, mult, sub) }
    #[inline(always)] fn mul_add_f32(self, mult: Self, add: Self) -> Self { self_from_op!(_mm_fmadd_ps, self, mult, add) }
    #[inline(always)] fn mul_sub_f32(self, mult: Self, sub: Self) -> Self { self_from_op!(_mm_fmsub_ps, self, mult, sub) }
    #[inline(always)] fn negated_mul_add_f32(self, mult: Self, add: Self) -> Self { self_from_op!(_mm_fnmadd_ps, self, mult, add) }
    #[inline(always)] fn negated_mul_sub_f32(self, mult: Self, sub: Self) -> Self { self_from_op!(_mm_fnmsub_ps, self, mult, sub) }
}

impl SimdRoundImpl for Sse {
    #[inline(always)] fn round_f64(self) -> Self { self_from_const_op!(_mm_round_pd, _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC, self) }
    #[inline(always)] fn round_f32(self) -> Self { self_from_const_op!(_mm_round_ps, _MM_FROUND_TO_NEAREST_INT | _MM_FROUND_NO_EXC, self) }
    #[inline(always)] fn floor_f64(self) -> Self { self_from_const_op!(_mm_round_pd, _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC, self) }
    #[inline(always)] fn floor_f32(self) -> Self { self_from_const_op!(_mm_round_ps, _MM_FROUND_TO_NEG_INF | _MM_FROUND_NO_EXC, self) }
    #[inline(always)] fn ceil_f64(self) -> Self { self_from_const_op!(_mm_round_pd, _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC, self) }
    #[inline(always)] fn ceil_f32(self) -> Self { self_from_const_op!(_mm_round_ps, _MM_FROUND_TO_POS_INF | _MM_FROUND_NO_EXC, self) }
}

impl SimdPartialOrdImpl for Sse {
    type MaskType = Self;
    #[inline(always)] fn cmp_f64_eq(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_pd, _CMP_EQ_OQ, self, rhs) }
    #[inline(always)] fn cmp_f64_lt(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_pd, _CMP_LT_OQ, self, rhs) }
    #[inline(always)] fn cmp_f64_le(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_pd, _CMP_LE_OQ, self, rhs) }
    #[inline(always)] fn cmp_f64_gt(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_pd, _CMP_GT_OQ, self, rhs) }
    #[inline(always)] fn cmp_f64_ge(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_pd, _CMP_GE_OQ, self, rhs) }
    #[inline(always)] fn cmp_f64_neq(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_pd, _CMP_NEQ_OQ, self, rhs) }
    #[inline(always)] fn cmp_f32_eq(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_ps, _CMP_EQ_OQ, self, rhs) }
    #[inline(always)] fn cmp_f32_lt(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_ps, _CMP_LT_OQ, self, rhs) }
    #[inline(always)] fn cmp_f32_le(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_ps, _CMP_LE_OQ, self, rhs) }
    #[inline(always)] fn cmp_f32_gt(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_ps, _CMP_GT_OQ, self, rhs) }
    #[inline(always)] fn cmp_f32_ge(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_ps, _CMP_GE_OQ, self, rhs) }
    #[inline(always)] fn cmp_f32_neq(self, rhs: Self) -> Self { self_from_const_op!(_mm_cmp_ps, _CMP_NEQ_OQ, self, rhs) }
    #[inline(always)] fn cmp_i64_eq(self, rhs: Self) -> Self { self_from_op!(_mm_cmpeq_epi64, self, rhs) }
    #[inline(always)] fn cmp_i64_gt(self, rhs: Self) -> Self { self_from_op!(_mm_cmpgt_epi64, self, rhs) }
    #[inline(always)] fn cmp_i32_eq(self, rhs: Self) -> Self { self_from_op!(_mm_cmpeq_epi32, self, rhs) }
    #[inline(always)] fn cmp_i32_gt(self, rhs: Self) -> Self { self_from_op!(_mm_cmpgt_epi32, self, rhs) }
    #[inline(always)] fn cmp_i16_eq(self, rhs: Self) -> Self { self_from_op!(_mm_cmpeq_epi16, self, rhs) }
    #[inline(always)] fn cmp_i16_gt(self, rhs: Self) -> Self { self_from_op!(_mm_cmpgt_epi16, self, rhs) }
    #[inline(always)] fn cmp_i8_eq(self, rhs: Self) -> Self { self_from_op!(_mm_cmpeq_epi8, self, rhs) }
    #[inline(always)] fn cmp_i8_gt(self, rhs: Self) -> Self { self_from_op!(_mm_cmpgt_epi8, self, rhs) }

    #[inline(always)] fn max_f64(self, rhs: Self) -> Self { self_from_op!(_mm_max_pd, self, rhs) }
    #[inline(always)] fn min_f64(self, rhs: Self) -> Self { self_from_op!(_mm_min_pd, self, rhs) }
    #[inline(always)] fn max_f32(self, rhs: Self) -> Self { self_from_op!(_mm_max_ps, self, rhs) }
    #[inline(always)] fn min_f32(self, rhs: Self) -> Self { self_from_op!(_mm_min_ps, self, rhs) }
    #[inline(always)] fn max_i32(self, rhs: Self) -> Self { self_from_op!(_mm_max_epi32, self, rhs) }
    #[inline(always)] fn min_i32(self, rhs: Self) -> Self { self_from_op!(_mm_min_epi32, self, rhs) }
    #[inline(always)] fn max_i16(self, rhs: Self) -> Self { self_from_op!(_mm_max_epi16, self, rhs) }
    #[inline(always)] fn min_i16(self, rhs: Self) -> Self { self_from_op!(_mm_min_epi16, self, rhs) }
    #[inline(always)] fn max_i8(self, rhs: Self) -> Self { self_from_op!(_mm_max_epi8, self, rhs) }
    #[inline(always)] fn min_i8(self, rhs: Self) -> Self { self_from_op!(_mm_min_epi8, self, rhs) }
    #[inline(always)] fn max_u32(self, rhs: Self) -> Self { self_from_op!(_mm_max_epu32, self, rhs) }
    #[inline(always)] fn min_u32(self, rhs: Self) -> Self { self_from_op!(_mm_min_epu32, self, rhs) }
    #[inline(always)] fn max_u16(self, rhs: Self) -> Self { self_from_op!(_mm_max_epu16, self, rhs) }
    #[inline(always)] fn min_u16(self, rhs: Self) -> Self { self_from_op!(_mm_min_epu16, self, rhs) }
    #[inline(always)] fn max_u8(self, rhs: Self) -> Self { self_from_op!(_mm_max_epu8, self, rhs) }
    #[inline(always)] fn min_u8(self, rhs: Self) -> Self { self_from_op!(_mm_min_epu8, self, rhs) }
}

// TODO: Make a custom trait for handling this transmutation into i*.
impl SimdSplatImpl for Sse {
    #[inline(always)] fn splat_64<T>(val: T) -> Self { self_from_op!(_mm_set1_epi64x, val) }
    #[inline(always)] fn splat_32<T>(val: T) -> Self { self_from_op!(_mm_set1_epi32, val) }
    #[inline(always)] fn splat_16<T>(val: T) -> Self { self_from_op!(_mm_set1_epi16, val) }
    #[inline(always)] fn splat_8<T>(val: T) -> Self { self_from_op!(_mm_set1_epi8, val) }
}

impl SimdGatherImpl for Sse {
    #[inline(always)] fn gather_32_from_32<T, const B: i32>(self, ptr: *const T) -> Self { self_from_const_op!(_mm_i32gather_epi32, B, ptr, self) }
    // #[inline(always)] fn gather_64_from_32<T, const B: i32>(self, ptr: *const T) -> Self { self_from_const_op!(_mm_i32gather_epi64, B, ptr, self) }
    // #[inline(always)] fn gather_32_from_64<T, const B: i32>(self, ptr: *const T) -> Self { self_from_const_op!(_mm_i64gather_epi32, B, ptr, self) }
    #[inline(always)] fn gather_64_from_64<T, const B: i32>(self, ptr: *const T) -> Self { self_from_const_op!(_mm_i64gather_epi64, B, ptr, self) }
}

impl SimdSqrtImpl for Sse {
    #[inline(always)] fn sqrt_f64(self) -> Self { self_from_op!(_mm_sqrt_pd, self) }
    #[inline(always)] fn sqrt_f32(self) -> Self { self_from_op!(_mm_sqrt_ps, self) }
    #[inline(always)] fn rsqrt_f32(self) -> Self { self_from_op!(_mm_rsqrt_ps, self) }
}

impl SimdAllBitsImpl for Sse {
    #[inline(always)] fn all_zero(self) -> bool { execute_intrinsic!(_mm_testz_si128, self, self) == 0 }
}

impl SimdNegateImpl for Sse {
    #[inline(always)] fn negate_f64(self) -> Self { Self::splat_64(-0.0f64).xor(self) }
    #[inline(always)] fn negate_f32(self) -> Self { Self::splat_32(-0.0f64).xor(self) }
}

impl SimdBlockShiftImpl for Sse {
    #[inline(always)] fn block_left_byte_shift<const N: i32>(self) -> Self { self_from_const_op!(_mm_bslli_si128, N, self) }
    #[inline(always)] fn block_right_byte_shift<const N: i32>(self) -> Self { self_from_const_op!(_mm_bsrli_si128, N, self) }
}

impl SimdMaskBitConversion for Sse {
    #[inline(always)] fn to_bits_64(self) -> u64 { execute_intrinsic!(_mm_movemask_pd, self) as u64 }
    #[inline(always)] fn to_bits_32(self) -> u64 { execute_intrinsic!(_mm_movemask_ps, self) as u64 }
    #[inline(always)] fn to_bits_8(self) -> u64 { execute_intrinsic!(_mm_movemask_epi8, self) as u64 }
}

// impl SimdLaneShiftImpl for Sse {
//     #[inline(always)] fn left_lane_shift<const N: i32>(self) -> Self { self_from_const_op!(_mm_bslli_si128, N, self) }
//     #[inline(always)] fn right_lane_shift<const N: i32>(self) -> Self { self_from_const_op!(_mm_bsrli_si128, N, self) }
// }