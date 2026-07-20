#![allow(clippy::missing_safety_doc)]
use std::{fmt::Debug, ops::{Index, IndexMut}};

use crate::simd::{SimdElement, array_trait::Array, register::Simd};

pub trait Arch: Clone + Copy + Default {
    const SIMD_WIDTH: usize;
    const NUM_SIMD_REG: usize;
    type Block2<T: SimdElement>: Index<usize, Output = Simd<T, Self>> + IndexMut<usize> + Default;
    type Block4<T: SimdElement>: Index<usize, Output = Simd<T, Self>> + IndexMut<usize> + Default;

    type Vec: SimdArch
        + Copy
        + Clone
        + SimdLoadImpl<MaskType = Self::Mask>
        + SimdStoreImpl<MaskType = Self::Mask>
        + SimdPartialOrdImpl<MaskType = Self::Mask>;
    type Mask: MaskArch + Copy + Clone + SimdVariableBlendImpl<VecType = Self::Vec>;
    type ScalarArch: Arch;

    type Array64<T: Debug + Copy>: Debug + Copy + Array<T>;
    type Array32<T: Debug + Copy>: Debug + Copy + Array<T>;
    type Array16<T: Debug + Copy>: Debug + Copy + Array<T>;
    type Array8<T: Debug + Copy>: Debug + Copy + Array<T>;
}

pub trait SimdArch:
    Copy +
    Clone +
    SimdAddImpl +
    SimdSubImpl +
    SimdMulImpl +
    SimdDivImpl +
    SimdBitwiseImpl +
    SimdShiftImpl +
    SimdLoadImpl +
    SimdStoreImpl +
    SimdZeroImpl +
    SimdFloatCastsImpl +
    SimdIntCastsImpl +
    SimdPermuteImpl +
    SimdMulAddImpl +
    SimdRoundImpl +
    SimdPartialOrdImpl +
    SimdSplatImpl +
    SimdGatherImpl +
    SimdSqrtImpl +
    SimdNegateImpl +
    SimdBlockShiftImpl +
    SimdImmediateBlendImpl +
    SimdLaneShiftImpl +
{}

pub trait MaskArch:
    Copy + Clone + SimdBitwiseImpl + SimdAllBitsImpl + SimdVariableBlendImpl + SimdMaskBitConversion
{
}

// === Arithmetic ===
pub trait SimdAddImpl {
    unsafe fn f64_add(self, rhs: Self) -> Self;
    unsafe fn f32_add(self, rhs: Self) -> Self;
    unsafe fn i64_add(self, rhs: Self) -> Self;
    unsafe fn i32_add(self, rhs: Self) -> Self;
    unsafe fn i16_add(self, rhs: Self) -> Self;
    unsafe fn i8_add(self, rhs: Self) -> Self;
}

pub trait SimdSubImpl {
    unsafe fn f64_sub(self, rhs: Self) -> Self;
    unsafe fn f32_sub(self, rhs: Self) -> Self;
    unsafe fn i64_sub(self, rhs: Self) -> Self;
    unsafe fn i32_sub(self, rhs: Self) -> Self;
    unsafe fn i16_sub(self, rhs: Self) -> Self;
    unsafe fn i8_sub(self, rhs: Self) -> Self;
}

pub trait SimdMulImpl {
    unsafe fn f64_mul(self, rhs: Self) -> Self;
    unsafe fn f32_mul(self, rhs: Self) -> Self;
    unsafe fn i32_mul(self, rhs: Self) -> Self;
    unsafe fn i16_mul(self, rhs: Self) -> Self;
}

pub trait SimdDivImpl {
    unsafe fn f64_div(self, rhs: Self) -> Self;
    unsafe fn f32_div(self, rhs: Self) -> Self;
}

pub trait SimdBitwiseImpl {
    unsafe fn and(self, rhs: Self) -> Self;
    unsafe fn or(self, rhs: Self) -> Self;
    unsafe fn xor(self, rhs: Self) -> Self;
    unsafe fn not(self) -> Self;
    unsafe fn and_not(self, rhs: Self) -> Self;
}

pub trait SimdShiftImpl {
    unsafe fn sllv_64(self, shift: Self) -> Self;
    unsafe fn srlv_64(self, shift: Self) -> Self;
    unsafe fn srav_64(self, shift: Self) -> Self;
    unsafe fn sllv_32(self, shift: Self) -> Self;
    unsafe fn srlv_32(self, shift: Self) -> Self;
    unsafe fn srav_32(self, shift: Self) -> Self;
    unsafe fn sllv_16(self, shift: Self) -> Self;
    unsafe fn srlv_16(self, shift: Self) -> Self;
    unsafe fn srav_16(self, shift: Self) -> Self;
}

pub trait SimdLoadImpl {
    type MaskType;
    unsafe fn load_aligned<T>(ptr: *const T) -> Self;
    unsafe fn load_unaligned<T>(ptr: *const T) -> Self;
    unsafe fn masked_load_64<T>(ptr: *const T, mask: Self::MaskType) -> Self;
    unsafe fn masked_load_32<T>(ptr: *const T, mask: Self::MaskType) -> Self;
    // TODO: There are byte and short mask loaders as well.
}

pub trait SimdStoreImpl {
    type MaskType;
    unsafe fn store_aligned<T>(self, ptr: *mut T);
    unsafe fn store_unaligned<T>(self, ptr: *mut T);
    unsafe fn masked_store_64<T>(self, ptr: *mut T, mask: Self::MaskType);
    unsafe fn masked_store_32<T>(self, ptr: *mut T, mask: Self::MaskType);
    // TODO: There are byte and short mask storers as well.
}

// pub trait SimdExtractImpl {
//     unsafe fn extract_64<T, const N: i32>(self) -> T;
//     unsafe fn extract_32<T, const N: i32>(self) -> T;
//     unsafe fn extract_16<T, const N: i32>(self) -> T;
//     unsafe fn extract_8<T, const N: i32>(self) -> T;
// }

// pub trait SimdInsertImpl {
//     unsafe fn insert_64<T, const N: i32>(self, val: T) -> Self;
//     unsafe fn insert_32<T, const N: i32>(self, val: T) -> Self;
//     unsafe fn insert_16<T, const N: i32>(self, val: T) -> Self;
//     unsafe fn insert_8<T, const N: i32>(self, val: T) -> Self;
// }

pub trait SimdZeroImpl {
    unsafe fn zero() -> Self;
}

pub trait SimdFloatCastsImpl {
    unsafe fn float_to_int_trunc(self) -> Self;
    unsafe fn float_to_int_round(self) -> Self;
}

pub trait SimdIntCastsImpl {
    unsafe fn int_to_float(self) -> Self;
}

pub trait SimdPermuteImpl {
    // type BlockVec;
    unsafe fn permute_32(self, rhs: Self) -> Self;
    unsafe fn permute_8(self, rhs: Self) -> Self;
    // unsafe fn imm_permute_64<const M: i32>(self);
    // unsafe fn imm_permute_32_lo<const M: i32>(self);
    // unsafe fn imm_permute_16_lo<const M: i32>(self);
}

pub trait SimdVariableBlendImpl {
    type VecType;
    unsafe fn vblend_64(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType;
    unsafe fn vblend_32(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType;
    unsafe fn vblend_8(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType;
}

pub trait SimdImmediateBlendImpl {
    unsafe fn blend_64<const N: i32>(self, false_values: Self) -> Self;
    unsafe fn blend_32<const N: i32>(self, false_values: Self) -> Self;
}
pub trait SimdMulAddImpl {
    unsafe fn mul_add_f64(self, mult: Self, add: Self) -> Self;
    unsafe fn mul_sub_f64(self, mult: Self, sub: Self) -> Self;
    unsafe fn negated_mul_add_f64(self, mult: Self, add: Self) -> Self;
    unsafe fn negated_mul_sub_f64(self, mult: Self, sub: Self) -> Self;
    unsafe fn mul_add_f32(self, mult: Self, add: Self) -> Self;
    unsafe fn mul_sub_f32(self, mult: Self, sub: Self) -> Self;
    unsafe fn negated_mul_add_f32(self, mult: Self, add: Self) -> Self;
    unsafe fn negated_mul_sub_f32(self, mult: Self, sub: Self) -> Self;
}

pub trait SimdRoundImpl {
    unsafe fn round_f64(self) -> Self;
    unsafe fn round_f32(self) -> Self;
    unsafe fn floor_f64(self) -> Self;
    unsafe fn floor_f32(self) -> Self;
    unsafe fn ceil_f64(self) -> Self;
    unsafe fn ceil_f32(self) -> Self;
}

pub trait SimdPartialOrdImpl {
    type MaskType;
    unsafe fn cmp_f64_eq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f64_lt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f64_le(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f64_gt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f64_ge(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f64_neq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f32_eq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f32_lt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f32_le(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f32_gt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f32_ge(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_f32_neq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i64_eq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i64_gt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i32_eq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i32_gt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i16_eq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i16_gt(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i8_eq(self, rhs: Self) -> Self::MaskType;
    unsafe fn cmp_i8_gt(self, rhs: Self) -> Self::MaskType;

    unsafe fn max_f64(self, rhs: Self) -> Self;
    unsafe fn min_f64(self, rhs: Self) -> Self;
    unsafe fn max_f32(self, rhs: Self) -> Self;
    unsafe fn min_f32(self, rhs: Self) -> Self;
    unsafe fn max_i32(self, rhs: Self) -> Self;
    unsafe fn min_i32(self, rhs: Self) -> Self;
    unsafe fn max_i16(self, rhs: Self) -> Self;
    unsafe fn min_i16(self, rhs: Self) -> Self;
    unsafe fn max_i8(self, rhs: Self) -> Self;
    unsafe fn min_i8(self, rhs: Self) -> Self;
    unsafe fn max_u32(self, rhs: Self) -> Self;
    unsafe fn min_u32(self, rhs: Self) -> Self;
    unsafe fn max_u16(self, rhs: Self) -> Self;
    unsafe fn min_u16(self, rhs: Self) -> Self;
    unsafe fn max_u8(self, rhs: Self) -> Self;
    unsafe fn min_u8(self, rhs: Self) -> Self;
}

pub trait SimdSplatImpl {
    unsafe fn splat_64<T>(val: T) -> Self;
    unsafe fn splat_32<T>(val: T) -> Self;
    unsafe fn splat_16<T>(val: T) -> Self;
    unsafe fn splat_8<T>(val: T) -> Self;
}

pub trait SimdGatherImpl {
    unsafe fn gather_32_from_32<T, const B: i32>(self, ptr: *const T) -> Self;
    // unsafe fn gather_64_from_32<T, const B: i32>(ptr: *const T, indicies: Self) -> Self;
    // unsafe fn gather_32_from_64<T, const B: i32>(ptr: *const T, indicies: Self) -> Self;
    unsafe fn gather_64_from_64<T, const B: i32>(self, ptr: *const T) -> Self;
}

pub trait SimdSqrtImpl {
    unsafe fn sqrt_f64(self) -> Self;
    unsafe fn sqrt_f32(self) -> Self;
    // unsafe fn rsqrt_f64(self) -> Self;
    unsafe fn rsqrt_f32(self) -> Self;
}

pub trait SimdAllBitsImpl {
    unsafe fn all_zero(self) -> bool;
}

pub trait SimdNegateImpl {
    unsafe fn negate_f64(self) -> Self;
    unsafe fn negate_f32(self) -> Self;
}

pub trait SimdBlockShiftImpl {
    unsafe fn block_right_byte_shift<const N: i32>(self) -> Self;
    unsafe fn block_left_byte_shift<const N: i32>(self) -> Self;
}

// TODO: Add to_bits_16 and from_bits.
pub trait SimdMaskBitConversion {
    unsafe fn to_bits_64(self) -> u64;
    unsafe fn to_bits_32(self) -> u64;
    // unsafe fn to_bits_16(self) -> u64;
    unsafe fn to_bits_8(self) -> u64;
    unsafe fn from_bits_64(bitmask: u64) -> Self;
    unsafe fn from_bits_32(bitmask: u64) -> Self;
    unsafe fn from_bits_16(bitmask: u64) -> Self;
    unsafe fn from_bits_8(bitmask: u64) -> Self;
}

pub trait SimdLaneShiftImpl {
    unsafe fn right_lane_shift_32(self, n: u32) -> Self;
    unsafe fn left_lane_shift_32(self, n: u32) -> Self;
}
