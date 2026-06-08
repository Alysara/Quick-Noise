// OPERATIONS WITH NATIVE SUPPORT.
use std::fmt::Debug;

use crate::simd::array_trait::Array;
use crate::simd::traits::*;

pub trait SimdFamily: Clone + Copy {
    const SIMD_WIDTH: usize;
    type Vec: SimdArch
        + Copy
        + Clone
        + SimdLoadImpl<MaskType = Self::Mask>
        + SimdStoreImpl<MaskType = Self::Mask>
        + SimdPartialOrdImpl<MaskType = Self::Mask>;
    // SimdPermuteImpl<BlockVec = <Self::BlockFamily as SimdFamily>::Vec>;
    type Mask: MaskArch + Copy + Clone + SimdVariableBlendImpl<VecType = Self::Vec>;
    type ScalarFamily: SimdFamily;

    type Array64<T: Debug + Copy>: Debug + Copy + Array<T>;
    type Array32<T: Debug + Copy>: Debug + Copy + Array<T>;
    type Array16<T: Debug + Copy>: Debug + Copy + Array<T>;
    type Array8<T: Debug + Copy>: Debug + Copy + Array<T>;
}

// pub trait LaneCounts<T: SimdElement> {
//     const LANES: usize;
//     type ArrayType;
// }

// impl<F: SimdFamily, T: SimdElement> LaneCounts<T> for F {
//     const LANES: usize = F::SIMD_WIDTH / std::mem::size_of::<T>();

//     // What I would like to do:
//     type ArrayType = [T; F::SIMD_WIDTH / std::mem::size_of::<T>()];

//     // OR make this manually for every type for every family, except then
//     // SimdElement doesn't know it has that trait without an explicit bound.
// }

// pub trait FloatType: Config {} // ?
// impl FloatType for f32 {}
// impl FloatType for f64 {}

// pub trait Config {}

// struct Config1;
// struct Config2;

// impl Config for Config1 {}
// impl Config for Config2 {}

// pub trait FloatSecrets {
//     type SecretArray: Default;
// }

// impl FloatSecrets for Config1 { type SecretArray = [f32; 8]; }
// impl FloatSecrets for Config1 { type SecretArray = [f64; 4]; }

// impl FloatSecrets<Config2> for f32 { type SecretArray = [f32; 16]; }
// impl FloatSecrets<Config2> for f64 { type SecretArray = [f64; 8]; }
// trait Array<T> {
//     fn from_fn(f: impl FnMut(usize) -> T) -> Self;
// }

// impl<const N: usize, T> Array<T> for [T; N] {
//     fn from_fn(f: impl FnMut(usize) -> T) -> Self {
//         std::array::from_fn(f)
//     }
// }

// trait FloatType: Default {
//     type Array<C: Config>: Default + Array<Self>;
// }

// trait Config {
//     type F32Array: Default + Array<f32>;
//     type F64Array: Default + Array<f64>;
// }

// impl FloatType for f32 {
//     type Array<C: Config> = <C as Config>::F32Array;
// }

// impl FloatType for f64 {
//     type Array<C: Config> = C::F64Array;
// }

// struct Config1;
// struct Config2;

// impl Config for Config1 {
//     type F32Array = [f32; 8];
//     type F64Array = [f64; 4];
// }

// impl Config for Config2 {
//     type F32Array = [f32; 16];
//     type F64Array = [f64; 8];
// }

// fn example<T: FloatType, C: Config, const N: usize>() -> [T; N]
// where
//   <T as FloatType>::Array<C>: Into<[T; N]>
// {
//     <<T as FloatType>::Array<C> as Array<T>>::from_fn(|i| T::default()).into()
// }

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
    // SimdExtractImpl +
    // SimdInsertImpl +
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
    fn f64_add(self, rhs: Self) -> Self;
    fn f32_add(self, rhs: Self) -> Self;
    fn i64_add(self, rhs: Self) -> Self;
    fn i32_add(self, rhs: Self) -> Self;
    fn i16_add(self, rhs: Self) -> Self;
    fn i8_add(self, rhs: Self) -> Self;
}

pub trait SimdSubImpl {
    fn f64_sub(self, rhs: Self) -> Self;
    fn f32_sub(self, rhs: Self) -> Self;
    fn i64_sub(self, rhs: Self) -> Self;
    fn i32_sub(self, rhs: Self) -> Self;
    fn i16_sub(self, rhs: Self) -> Self;
    fn i8_sub(self, rhs: Self) -> Self;
}

pub trait SimdMulImpl {
    fn f64_mul(self, rhs: Self) -> Self;
    fn f32_mul(self, rhs: Self) -> Self;
    fn i32_mul(self, rhs: Self) -> Self;
    fn i16_mul(self, rhs: Self) -> Self;
}

pub trait SimdDivImpl {
    fn f64_div(self, rhs: Self) -> Self;
    fn f32_div(self, rhs: Self) -> Self;
}

pub trait SimdBitwiseImpl {
    fn and(self, rhs: Self) -> Self;
    fn or(self, rhs: Self) -> Self;
    fn xor(self, rhs: Self) -> Self;
    fn not(self) -> Self;
    fn and_not(self, rhs: Self) -> Self;
}

pub trait SimdShiftImpl {
    fn sllv_64(self, shift: Self) -> Self;
    fn srlv_64(self, shift: Self) -> Self;
    fn srav_64(self, shift: Self) -> Self;
    fn sllv_32(self, shift: Self) -> Self;
    fn srlv_32(self, shift: Self) -> Self;
    fn srav_32(self, shift: Self) -> Self;
    fn sllv_16(self, shift: Self) -> Self;
    fn srlv_16(self, shift: Self) -> Self;
    fn srav_16(self, shift: Self) -> Self;
}

pub trait SimdLoadImpl {
    type MaskType;
    fn load_aligned<T>(ptr: *const T) -> Self;
    fn load_unaligned<T>(ptr: *const T) -> Self;
    fn masked_load_64<T>(ptr: *const T, mask: Self::MaskType) -> Self;
    fn masked_load_32<T>(ptr: *const T, mask: Self::MaskType) -> Self;
    // TODO: There are byte and short mask loaders as well.
}

pub trait SimdStoreImpl {
    type MaskType;
    fn store_aligned<T>(self, ptr: *mut T);
    fn store_unaligned<T>(self, ptr: *mut T);
    fn masked_store_64<T>(self, ptr: *mut T, mask: Self::MaskType);
    fn masked_store_32<T>(self, ptr: *mut T, mask: Self::MaskType);
    // TODO: There are byte and short mask storers as well.
}

// pub trait SimdExtractImpl {
//     fn extract_64<T, const N: i32>(self) -> T;
//     fn extract_32<T, const N: i32>(self) -> T;
//     fn extract_16<T, const N: i32>(self) -> T;
//     fn extract_8<T, const N: i32>(self) -> T;
// }

// pub trait SimdInsertImpl {
//     fn insert_64<T, const N: i32>(self, val: T) -> Self;
//     fn insert_32<T, const N: i32>(self, val: T) -> Self;
//     fn insert_16<T, const N: i32>(self, val: T) -> Self;
//     fn insert_8<T, const N: i32>(self, val: T) -> Self;
// }

pub trait SimdZeroImpl {
    fn zero() -> Self;
}

pub trait SimdFloatCastsImpl {
    fn float_to_int_trunc(self) -> Self;
    fn float_to_int_round(self) -> Self;
}

pub trait SimdIntCastsImpl {
    fn int_to_float(self) -> Self;
}

pub trait SimdPermuteImpl {
    // type BlockVec;
    fn permute_32(self, rhs: Self) -> Self;
    fn permute_8(self, rhs: Self) -> Self;
    // fn imm_permute_64<const M: i32>(self);
    // fn imm_permute_32_lo<const M: i32>(self);
    // fn imm_permute_16_lo<const M: i32>(self);
}

pub trait SimdVariableBlendImpl {
    type VecType;
    fn vblend_64(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType;
    fn vblend_32(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType;
    fn vblend_8(self, true_values: Self::VecType, false_values: Self::VecType) -> Self::VecType;
}

pub trait SimdImmediateBlendImpl {
    fn blend_64<const N: i32>(self, false_values: Self) -> Self;
    fn blend_32<const N: i32>(self, false_values: Self) -> Self;
}
pub trait SimdMulAddImpl {
    fn mul_add_f64(self, mult: Self, add: Self) -> Self;
    fn mul_sub_f64(self, mult: Self, sub: Self) -> Self;
    fn negated_mul_add_f64(self, mult: Self, add: Self) -> Self;
    fn negated_mul_sub_f64(self, mult: Self, sub: Self) -> Self;
    fn mul_add_f32(self, mult: Self, add: Self) -> Self;
    fn mul_sub_f32(self, mult: Self, sub: Self) -> Self;
    fn negated_mul_add_f32(self, mult: Self, add: Self) -> Self;
    fn negated_mul_sub_f32(self, mult: Self, sub: Self) -> Self;
}

pub trait SimdRoundImpl {
    fn round_f64(self) -> Self;
    fn round_f32(self) -> Self;
    fn floor_f64(self) -> Self;
    fn floor_f32(self) -> Self;
    fn ceil_f64(self) -> Self;
    fn ceil_f32(self) -> Self;
}

pub trait SimdPartialOrdImpl {
    type MaskType;
    fn cmp_f64_eq(self, rhs: Self) -> Self::MaskType;
    fn cmp_f64_lt(self, rhs: Self) -> Self::MaskType;
    fn cmp_f64_le(self, rhs: Self) -> Self::MaskType;
    fn cmp_f64_gt(self, rhs: Self) -> Self::MaskType;
    fn cmp_f64_ge(self, rhs: Self) -> Self::MaskType;
    fn cmp_f64_neq(self, rhs: Self) -> Self::MaskType;
    fn cmp_f32_eq(self, rhs: Self) -> Self::MaskType;
    fn cmp_f32_lt(self, rhs: Self) -> Self::MaskType;
    fn cmp_f32_le(self, rhs: Self) -> Self::MaskType;
    fn cmp_f32_gt(self, rhs: Self) -> Self::MaskType;
    fn cmp_f32_ge(self, rhs: Self) -> Self::MaskType;
    fn cmp_f32_neq(self, rhs: Self) -> Self::MaskType;
    fn cmp_i64_eq(self, rhs: Self) -> Self::MaskType;
    fn cmp_i64_gt(self, rhs: Self) -> Self::MaskType;
    fn cmp_i32_eq(self, rhs: Self) -> Self::MaskType;
    fn cmp_i32_gt(self, rhs: Self) -> Self::MaskType;
    fn cmp_i16_eq(self, rhs: Self) -> Self::MaskType;
    fn cmp_i16_gt(self, rhs: Self) -> Self::MaskType;
    fn cmp_i8_eq(self, rhs: Self) -> Self::MaskType;
    fn cmp_i8_gt(self, rhs: Self) -> Self::MaskType;

    fn max_f64(self, rhs: Self) -> Self;
    fn min_f64(self, rhs: Self) -> Self;
    fn max_f32(self, rhs: Self) -> Self;
    fn min_f32(self, rhs: Self) -> Self;
    fn max_i32(self, rhs: Self) -> Self;
    fn min_i32(self, rhs: Self) -> Self;
    fn max_i16(self, rhs: Self) -> Self;
    fn min_i16(self, rhs: Self) -> Self;
    fn max_i8(self, rhs: Self) -> Self;
    fn min_i8(self, rhs: Self) -> Self;
    fn max_u32(self, rhs: Self) -> Self;
    fn min_u32(self, rhs: Self) -> Self;
    fn max_u16(self, rhs: Self) -> Self;
    fn min_u16(self, rhs: Self) -> Self;
    fn max_u8(self, rhs: Self) -> Self;
    fn min_u8(self, rhs: Self) -> Self;
}

pub trait SimdSplatImpl {
    fn splat_64<T>(val: T) -> Self;
    fn splat_32<T>(val: T) -> Self;
    fn splat_16<T>(val: T) -> Self;
    fn splat_8<T>(val: T) -> Self;
}

pub trait SimdGatherImpl {
    fn gather_32_from_32<T, const B: i32>(self, ptr: *const T) -> Self;
    // fn gather_64_from_32<T, const B: i32>(ptr: *const T, indicies: Self) -> Self;
    // fn gather_32_from_64<T, const B: i32>(ptr: *const T, indicies: Self) -> Self;
    fn gather_64_from_64<T, const B: i32>(self, ptr: *const T) -> Self;
}

pub trait SimdSqrtImpl {
    fn sqrt_f64(self) -> Self;
    fn sqrt_f32(self) -> Self;
    // fn rsqrt_f64(self) -> Self;
    fn rsqrt_f32(self) -> Self;
}

pub trait SimdAllBitsImpl {
    fn all_zero(self) -> bool;
}

pub trait SimdNegateImpl {
    fn negate_f64(self) -> Self;
    fn negate_f32(self) -> Self;
}

pub trait SimdBlockShiftImpl {
    fn block_right_byte_shift<const N: i32>(self) -> Self;
    fn block_left_byte_shift<const N: i32>(self) -> Self;
}

// TODO: Add to_bits_16 and from_bits.
pub trait SimdMaskBitConversion {
    fn to_bits_64(self) -> u64;
    fn to_bits_32(self) -> u64;
    // fn to_bits_16(self) -> u64;
    fn to_bits_8(self) -> u64;
    fn from_bits_64(bitmask: u64) -> Self;
    fn from_bits_32(bitmask: u64) -> Self;
    fn from_bits_16(bitmask: u64) -> Self;
    fn from_bits_8(bitmask: u64) -> Self;
}

pub trait SimdLaneShiftImpl {
    fn right_lane_shift_32(self, n: u32) -> Self;
    fn left_lane_shift_32(self, n: u32) -> Self;
}
