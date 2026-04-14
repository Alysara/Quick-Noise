use crate::simd::architectures::arch_impl::*;
use std::mem::{transmute, transmute_copy};
use crate::simd::architectures::macros::*;

#[derive(Copy, Clone)]
#[repr(align(8))]
pub struct Scalar<const N: usize>(pub [u8; N]);
#[derive(Copy, Clone)]
pub struct ScalarMask<const N: usize>(pub [bool; N]);

impl<const N: usize> SimdArch for Scalar<N> {}
impl<const N: usize> MaskArch for ScalarMask<N> {}

macro_rules! scalar_token_op {
    ($type:ty, $op:tt, $self:ident, $rhs:ident, $size:expr) => {
        unsafe {
            let mut new = Scalar::<$size>([0; $size]);

            let self_ptr: *const $type = $self.0.as_ptr() as *const $type;
            let rhs_ptr: *const $type = $rhs.0.as_ptr() as *const $type;
            let new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type;
            
            for i in 0..($size / size_of::<$type>()) {
                *new_ptr.add(i) = *self_ptr.add(i) $op *rhs_ptr.add(i);
            }

            new
        }
    }
}

macro_rules! scalar_token_op_usize_rhs {
    ($type:ty, $op:tt, $self:ident, $rhs:ident, $size:expr) => {
        unsafe {
            let mut new = Scalar::<$size>([0; $size]);

            let self_ptr: *const $type = $self.0.as_ptr() as *const $type;
            let rhs_ptr: *const $type = $rhs.0.as_ptr() as *const $type;
            let new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type;
            
            for i in 0..($size / size_of::<$type>()) {
                *new_ptr.add(i) = *self_ptr.add(i) $op (*rhs_ptr.add(i) as usize);
            }

            new
        }
    }
}

macro_rules! scalar_func_op {
    ($type:ty, $op:ident, $self:ident, $rhs:ident, $size:expr) => {
        unsafe {
            let mut new = Scalar::<$size>([0; $size]);

            let self_ptr: *const $type = $self.0.as_ptr() as *const $type;
            let rhs_ptr: *const $type = $rhs.0.as_ptr() as *const $type;
            let new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type;
            
            for i in 0..($size / size_of::<$type>()) {
                *new_ptr.add(i) = (*self_ptr.add(i)).$op(*rhs_ptr.add(i));
            }

            new
        }
    }
}

macro_rules! scalar_self_op {
    ($type:ty, $op:ident, $self:ident, $size:expr) => {
        unsafe {
            let mut new = Scalar::<$size>([0; $size]);

            let self_ptr: *const $type = $self.0.as_ptr() as *const $type;
            let new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type;

            for i in 0..($size / size_of::<$type>()) {
                *new_ptr.add(i) = (*self_ptr.add(i)).$op();
            }
            new
        }
    }
}

macro_rules! scalar_fma_expr_op {
    ($type:ty, $self:ident, $mult:ident, $add:ident, $size:expr, |$a:ident, $b:ident, $c:ident| $op:expr) => {
        unsafe {
            let mut new = Scalar::<$size>([0; $size]);

            let a_ptr: *const $type = $self.0.as_ptr() as *const $type;
            let b_ptr: *const $type = $mult.0.as_ptr() as *const $type;
            let c_ptr: *const $type = $add.0.as_ptr() as *const $type;
            let new_ptr: *mut $type = new.0.as_mut_ptr() as *mut $type;

            for i in 0..($size / size_of::<$type>()) {
                let $a = *a_ptr.add(i);
                let $b = *b_ptr.add(i);
                let $c = *c_ptr.add(i);
                *new_ptr.add(i) = $op;
            }
            new
        }
    }
}

macro_rules! scalar_cmp {
    {$type:ty, $op:tt, $self:ident, $rhs:ident, $size:expr} => {
        unsafe {
            let mut result = ScalarMask::<$size>([false; $size]);

            let self_ptr = $self.0.as_ptr() as *const $type;
            let rhs_ptr = $rhs.0.as_ptr() as *const $type;

            for i in 0..($size / size_of::<$type>()) {
                result.0[i] = *self_ptr.add(i) $op *rhs_ptr.add(i);
            }

            result
        }
    }
}

macro_rules! scalar_splat {
    {$type:ty, $self:ident, $val:expr, $size:expr} => {
        unsafe {
            let mut new = Scalar::<$size>([0; $size]);
            let new_ptr = new.0.as_mut_ptr() as *mut $type;

            for i in 0..(N / size_of::<$type>()) {
                *new_ptr.add(i) = $val;
            }
            
            new
        }
    }
}

impl<const N: usize> SimdAddImpl for Scalar<N> {
    #[inline(always)] fn f64_add(self, rhs: Self) -> Self { scalar_token_op!(f64, +, self, rhs, N) }
    #[inline(always)] fn f32_add(self, rhs: Self) -> Self { scalar_token_op!(f32, +, self, rhs, N) }
    #[inline(always)] fn i64_add(self, rhs: Self) -> Self { scalar_func_op!(i64, wrapping_add, self, rhs, N) }
    #[inline(always)] fn i32_add(self, rhs: Self) -> Self { scalar_func_op!(i32, wrapping_add, self, rhs, N) }
    #[inline(always)] fn i16_add(self, rhs: Self) -> Self { scalar_func_op!(i16, wrapping_add, self, rhs, N) }
    #[inline(always)] fn i8_add(self, rhs: Self) -> Self { scalar_func_op!(i8, wrapping_add, self, rhs, N) }
}

impl<const N: usize> SimdSubImpl for Scalar<N> {
    #[inline(always)] fn f64_sub(self, rhs: Self) -> Self { scalar_token_op!(f64, -, self, rhs, N) }
    #[inline(always)] fn f32_sub(self, rhs: Self) -> Self { scalar_token_op!(f32, -, self, rhs, N) }
    #[inline(always)] fn i64_sub(self, rhs: Self) -> Self { scalar_func_op!(i64, wrapping_sub, self, rhs, N) }
    #[inline(always)] fn i32_sub(self, rhs: Self) -> Self { scalar_func_op!(i32, wrapping_sub, self, rhs, N) }
    #[inline(always)] fn i16_sub(self, rhs: Self) -> Self { scalar_func_op!(i16, wrapping_sub, self, rhs, N) }
    #[inline(always)] fn i8_sub(self, rhs: Self) -> Self { scalar_func_op!(i8, wrapping_sub, self, rhs, N) }
}

impl<const N: usize> SimdMulImpl for Scalar<N> {
    #[inline(always)] fn f64_mul(self, rhs: Self) -> Self { scalar_token_op!(f64, *, self, rhs, N) }
    #[inline(always)] fn f32_mul(self, rhs: Self) -> Self { scalar_token_op!(f32, *, self, rhs, N) }
    #[inline(always)] fn i32_mul(self, rhs: Self) -> Self { scalar_func_op!(i32, wrapping_mul, self, rhs, N) }
    #[inline(always)] fn i16_mul(self, rhs: Self) -> Self { scalar_func_op!(i16, wrapping_mul, self, rhs, N) }
}

impl<const N: usize> SimdDivImpl for Scalar<N> {
    #[inline(always)] fn f64_div(self, rhs: Self) -> Self { scalar_token_op!(f64, /, self, rhs, N) }
    #[inline(always)] fn f32_div(self, rhs: Self) -> Self { scalar_token_op!(f32, /, self, rhs, N) }
}

impl<const N: usize> SimdBitwiseImpl for Scalar<N> {
    #[inline(always)] fn and(self, rhs: Self) -> Self { scalar_token_op!(u64, &, self, rhs, N) }
    #[inline(always)] fn or(self, rhs: Self) -> Self { scalar_token_op!(u64, |, self, rhs, N) }
    #[inline(always)] fn xor(self, rhs: Self) -> Self { scalar_token_op!(u64, ^, self, rhs, N) }
    #[inline(always)] fn not(self) -> Self { self.xor(Self([255; N])) }
    #[inline(always)] fn and_not(self, rhs: Self) -> Self { self.and(rhs.not()) }
}

impl<const N: usize> SimdShiftImpl for Scalar<N> {
    #[inline(always)] fn sllv_64(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(u64, <<, self, rhs, N) }
    #[inline(always)] fn srlv_64(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(u64, >>, self, rhs, N) }
    #[inline(always)] fn srav_64(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(i64, >>, self, rhs, N) }
    #[inline(always)] fn sllv_32(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(u32, <<, self, rhs, N) }
    #[inline(always)] fn srlv_32(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(u32, >>, self, rhs, N) }
    #[inline(always)] fn srav_32(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(i32, >>, self, rhs, N) }
    #[inline(always)] fn sllv_16(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(u16, <<, self, rhs, N) }
    #[inline(always)] fn srlv_16(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(u16, >>, self, rhs, N) }
    #[inline(always)] fn srav_16(self, rhs: Self) -> Self { scalar_token_op_usize_rhs!(i16, >>, self, rhs, N) }
}

impl<const N: usize> SimdLoadImpl for Scalar<N> {
    type MaskType = ScalarMask<N>;
    #[inline(always)] fn load_aligned<T>(ptr: *const T) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            std::ptr::copy_nonoverlapping(ptr as *const u8, new.0.as_mut_ptr(), N);
            new
        }
    }
    #[inline(always)] fn load_unaligned<T>(ptr: *const T) -> Self { Self::load_aligned::<T>(ptr) }
    #[inline(always)] fn masked_load_64<T>(ptr: *const T, mask: Self::MaskType) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u64;
            let data_ptr = ptr as *const u64;
            for i in 0..(N / 8) {
                *new_ptr.add(i) = if mask.0[i] { *data_ptr.add(i) } else { 0 };
            }
            new
        }
    }
    #[inline(always)] fn masked_load_32<T>(ptr: *const T, mask: Self::MaskType) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u32;
            let data_ptr = ptr as *const u32;
            for i in 0..(N / 4) {
                *new_ptr.add(i) = if mask.0[i] { *data_ptr.add(i) } else { 0 };
            }
            new
        }
    }
}

impl<const N: usize> SimdStoreImpl for Scalar<N> {
    type MaskType = ScalarMask<N>;
    #[inline(always)] fn store_aligned<T>(self, ptr: *mut T) {
        unsafe {
            std::ptr::copy_nonoverlapping(self.0.as_ptr(), ptr as *mut u8, N);
        }
    }
    #[inline(always)] fn store_unaligned<T>(self, ptr: *mut T) { self.store_aligned(ptr); }
    #[inline(always)] fn masked_store_64<T>(self, ptr: *mut T, mask: Self::MaskType) {
        unsafe {
            let self_ptr = self.0.as_ptr() as *const u64;
            let store_ptr = ptr as *mut u64;
            for i in 0..(N / 8) {
                if mask.0[i] {
                    *store_ptr.add(i) = *self_ptr.add(i);
                }
            }
        }
    }
    #[inline(always)] fn masked_store_32<T>(self, ptr: *mut T, mask: Self::MaskType) {
        unsafe {
            let self_ptr = self.0.as_ptr() as *const u32;
            let store_ptr = ptr as *mut u32;
            for i in 0..(N / 4) {
                if mask.0[i] {
                    *store_ptr.add(i) = *self_ptr.add(i);
                }
            }
        }
    }
}

impl<const N: usize> SimdZeroImpl for Scalar<N> {
    #[inline(always)] fn zero() -> Self { Self([0; N]) }
}

impl<const N: usize> SimdFloatCastsImpl for Scalar<N> {
    #[inline(always)] fn float_to_int_trunc(self) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let float_ptr = self.0.as_ptr() as *const f32;
            let int_ptr = new.0.as_mut_ptr() as *mut i32;
            for i in 0..(N / 4) {
                *int_ptr.add(i) = (*float_ptr.add(i)) as i32;
            }
            new
        }
    }
    #[inline(always)] fn float_to_int_round(self) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let float_ptr = self.0.as_ptr() as *const f32;
            let int_ptr = new.0.as_mut_ptr() as *mut i32;
            for i in 0..(N / 4) {
                *int_ptr.add(i) = (*float_ptr.add(i)).round_ties_even() as i32;
            }
            new
        }
    }
}

impl<const N: usize> SimdIntCastsImpl for Scalar<N> {
    #[inline(always)] fn int_to_float(self) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let int_ptr = self.0.as_ptr() as *const i32;
            let float_ptr = new.0.as_mut_ptr() as *mut f32;
            for i in 0..(N / 4) {
                *float_ptr.add(i) = (*int_ptr.add(i)) as f32;
            }
            new
        }
    }
}

impl<const N: usize> SimdPermuteImpl for Scalar<N> {
    #[inline(always)] fn permute_32(self, rhs: Self) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u32;
            let self_ptr = self.0.as_ptr() as *const u32;
            let indices_ptr = rhs.0.as_ptr() as *const u32;
            for i in 0..(N / 4) {
                let index = *indices_ptr.add(i) as usize;
                *new_ptr.add(i) = if index < (N / 4) { *self_ptr.add(index) } else { 0 };
            }
            new
        }
    }
    #[inline(always)] fn permute_8(self, rhs: Self) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let new_ptr = new.0.as_mut_ptr();
            let self_ptr = self.0.as_ptr();
            let indices_ptr = rhs.0.as_ptr();
            for i in 0..N {
                let lane_base = i & !15; // which 16-byte lane we're in (0 or 16)
                let index = (*indices_ptr.add(i) as usize & 15) + lane_base;
                *new_ptr.add(i) = *self_ptr.add(index);
            }
            new
        }
    }
}

impl<const N: usize> SimdVariableBlendImpl for ScalarMask<N> {
    type VecType = Scalar<N>;
    #[inline(always)] fn vblend_64(self, true_values: Self::VecType, false_values: Self::VecType) -> Scalar<N> {
        unsafe {
            let mut new = Scalar::<N>([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u64;
            let false_ptr = false_values.0.as_ptr() as *const u64;
            let true_ptr = true_values.0.as_ptr() as *const u64;
            for i in 0..(N / 8) {
                *new_ptr.add(i) = if self.0[i] { *true_ptr.add(i) } else { *false_ptr.add(i) };
            }
            new
        }
    }
    #[inline(always)] fn vblend_32(self, true_values: Self::VecType, false_values: Self::VecType) -> Scalar<N> {
        unsafe {
            let mut new = Scalar::<N>([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u32;
            let false_ptr = false_values.0.as_ptr() as *const u32;
            let true_ptr = true_values.0.as_ptr() as *const u32;
            for i in 0..(N / 4) {
                *new_ptr.add(i) = if self.0[i] { *true_ptr.add(i) } else { *false_ptr.add(i) };
            }
            new
        }
    }
    #[inline(always)] fn vblend_8(self, true_values: Self::VecType, false_values: Self::VecType) -> Scalar<N> {
        unsafe {
            let mut new = Scalar::<N>([0; N]);
            let new_ptr = new.0.as_mut_ptr();
            let false_ptr = false_values.0.as_ptr();
            let true_ptr = true_values.0.as_ptr();
            for i in 0..N {
                *new_ptr.add(i) = if self.0[i] { *true_ptr.add(i) } else { *false_ptr.add(i) };
            }
            new
        }
    }
}

impl<const N: usize> SimdMulAddImpl for Scalar<N> {
    #[inline(always)] fn mul_add_f64(self, mult: Self, add: Self) -> Self { scalar_fma_expr_op!(f64, self, mult, add, N, |a, b, c| f64::mul_add(a, b, c)) }
    #[inline(always)] fn mul_sub_f64(self, mult: Self, sub: Self) -> Self { scalar_fma_expr_op!(f64, self, mult, sub, N, |a, b, c| f64::mul_add(a, b, -c)) }
    #[inline(always)] fn negated_mul_add_f64(self, mult: Self, add: Self) -> Self { scalar_fma_expr_op!(f64, self, mult, add, N, |a, b, c| f64::mul_add(-a, b, c)) }
    #[inline(always)] fn negated_mul_sub_f64(self, mult: Self, sub: Self) -> Self { scalar_fma_expr_op!(f64, self, mult, sub, N, |a, b, c| f64::mul_add(-a, b, -c)) }
    #[inline(always)] fn mul_add_f32(self, mult: Self, add: Self) -> Self { scalar_fma_expr_op!(f32, self, mult, add, N, |a, b, c| f32::mul_add(a, b, c)) }
    #[inline(always)] fn mul_sub_f32(self, mult: Self, sub: Self) -> Self { scalar_fma_expr_op!(f32, self, mult, sub, N, |a, b, c| f32::mul_add(a, b, -c)) }
    #[inline(always)] fn negated_mul_add_f32(self, mult: Self, add: Self) -> Self { scalar_fma_expr_op!(f32, self, mult, add, N, |a, b, c| f32::mul_add(-a, b, c)) }
    #[inline(always)] fn negated_mul_sub_f32(self, mult: Self, sub: Self) -> Self { scalar_fma_expr_op!(f32, self, mult, sub, N, |a, b, c| f32::mul_add(-a, b, -c)) }
}

impl<const N: usize> SimdRoundImpl for Scalar<N> {
    #[inline(always)] fn round_f64(self) -> Self { scalar_self_op!(f64, round_ties_even, self, N) }
    #[inline(always)] fn round_f32(self) -> Self { scalar_self_op!(f32, round_ties_even, self, N) }
    #[inline(always)] fn floor_f64(self) -> Self { scalar_self_op!(f64, floor, self, N) }
    #[inline(always)] fn floor_f32(self) -> Self { scalar_self_op!(f32, floor, self, N) }
    #[inline(always)] fn ceil_f64(self) -> Self { scalar_self_op!(f64, ceil, self, N) }
    #[inline(always)] fn ceil_f32(self) -> Self { scalar_self_op!(f32, ceil, self, N) }
}

impl<const N: usize> SimdPartialOrdImpl for Scalar<N> {
    type MaskType = ScalarMask<N>;
    #[inline(always)] fn cmp_f64_eq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f64, ==, self, rhs, N) }
    #[inline(always)] fn cmp_f64_lt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f64, <, self, rhs, N) }
    #[inline(always)] fn cmp_f64_le(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f64, <=, self, rhs, N) }
    #[inline(always)] fn cmp_f64_gt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f64, >, self, rhs, N) }
    #[inline(always)] fn cmp_f64_ge(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f64, >=, self, rhs, N) }
    #[inline(always)] fn cmp_f64_neq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f64, !=, self, rhs, N) }
    #[inline(always)] fn cmp_f32_eq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f32, ==, self, rhs, N) }
    #[inline(always)] fn cmp_f32_lt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f32, <, self, rhs, N) }
    #[inline(always)] fn cmp_f32_le(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f32, <=, self, rhs, N) }
    #[inline(always)] fn cmp_f32_gt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f32, >, self, rhs, N) }
    #[inline(always)] fn cmp_f32_ge(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f32, >=, self, rhs, N) }
    #[inline(always)] fn cmp_f32_neq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(f32, !=, self, rhs, N) }
    #[inline(always)] fn cmp_i64_eq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i64, ==, self, rhs, N) }
    #[inline(always)] fn cmp_i64_gt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i64, >, self, rhs, N) }
    #[inline(always)] fn cmp_i32_eq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i32, ==, self, rhs, N) }
    #[inline(always)] fn cmp_i32_gt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i32, >, self, rhs, N) }
    #[inline(always)] fn cmp_i16_eq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i16, ==, self, rhs, N) }
    #[inline(always)] fn cmp_i16_gt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i16, >, self, rhs, N) }
    #[inline(always)] fn cmp_i8_eq(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i8, ==, self, rhs, N) }
    #[inline(always)] fn cmp_i8_gt(self, rhs: Self) -> Self::MaskType { scalar_cmp!(i8, >, self, rhs, N) }

    #[inline(always)] fn max_f64(self, rhs: Self) -> Self { scalar_func_op!(f64, max, self, rhs, N) }
    #[inline(always)] fn min_f64(self, rhs: Self) -> Self { scalar_func_op!(f64, min, self, rhs, N) }
    #[inline(always)] fn max_f32(self, rhs: Self) -> Self { scalar_func_op!(f32, max, self, rhs, N) }
    #[inline(always)] fn min_f32(self, rhs: Self) -> Self { scalar_func_op!(f32, min, self, rhs, N) }
    #[inline(always)] fn max_i32(self, rhs: Self) -> Self { scalar_func_op!(i32, max, self, rhs, N) }
    #[inline(always)] fn min_i32(self, rhs: Self) -> Self { scalar_func_op!(i32, min, self, rhs, N) }
    #[inline(always)] fn max_i16(self, rhs: Self) -> Self { scalar_func_op!(i16, max, self, rhs, N) }
    #[inline(always)] fn min_i16(self, rhs: Self) -> Self { scalar_func_op!(i16, min, self, rhs, N) }
    #[inline(always)] fn max_i8(self, rhs: Self) -> Self { scalar_func_op!(i8, max, self, rhs, N) }
    #[inline(always)] fn min_i8(self, rhs: Self) -> Self { scalar_func_op!(i8, min, self, rhs, N) }
    #[inline(always)] fn max_u32(self, rhs: Self) -> Self { scalar_func_op!(u32, max, self, rhs, N) }
    #[inline(always)] fn min_u32(self, rhs: Self) -> Self { scalar_func_op!(u32, min, self, rhs, N) }
    #[inline(always)] fn max_u16(self, rhs: Self) -> Self { scalar_func_op!(u16, max, self, rhs, N) }
    #[inline(always)] fn min_u16(self, rhs: Self) -> Self { scalar_func_op!(u16, min, self, rhs, N) }
    #[inline(always)] fn max_u8(self, rhs: Self) -> Self { scalar_func_op!(u8, max, self, rhs, N) }
    #[inline(always)] fn min_u8(self, rhs: Self) -> Self { scalar_func_op!(u8, min, self, rhs, N) }
}

// TODO: Make a custom trait for handling this transmutation into i*.
impl<const N: usize> SimdSplatImpl for Scalar<N> {
    #[inline(always)] fn splat_64<T>(val: T) -> Self { scalar_splat!(u64, self, transmute_copy(&val), N) }
    #[inline(always)] fn splat_32<T>(val: T) -> Self { scalar_splat!(u32, self, transmute_copy(&val), N) }
    #[inline(always)] fn splat_16<T>(val: T) -> Self { scalar_splat!(u16, self, transmute_copy(&val), N) }
    #[inline(always)] fn splat_8<T>(val: T) -> Self { scalar_splat!(u8, self, transmute_copy(&val), N) }
}

impl<const N: usize> SimdGatherImpl for Scalar<N> {
    #[inline(always)] fn gather_32_from_32<T, const B: i32>(self, ptr: *const T) -> Self { 
        unsafe {
            let mut new = Self([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u32;
            let indices_ptr = self.0.as_ptr() as *const u32;
            let data_ptr = ptr as *const u32;
            for i in 0..(N / 4) {
                let index = *indices_ptr.add(i) as usize;
                *new_ptr.add(i) = *data_ptr.add(index);
            }
            new
        }
    }
    #[inline(always)] fn gather_64_from_64<T, const B: i32>(self, ptr: *const T) -> Self {
        unsafe {
            let mut new = Self([0; N]);
            let new_ptr = new.0.as_mut_ptr() as *mut u64;
            let indices_ptr = self.0.as_ptr() as *const u64;
            let data_ptr = ptr as *const u64;
            for i in 0..(N / 8) {
                let index = *indices_ptr.add(i) as usize;
                *new_ptr.add(i) = *data_ptr.add(index);
            }
            new
        }
    }
}

impl<const N: usize> SimdSqrtImpl for Scalar<N> {
    #[inline(always)] fn sqrt_f64(self) -> Self { scalar_self_op!(f64, sqrt, self, N) }
    #[inline(always)] fn sqrt_f32(self) -> Self { scalar_self_op!(f32, sqrt, self, N) }
    #[inline(always)] fn rsqrt_f32(self) -> Self { Self::splat_32(1.0f32).f32_div(self.sqrt_f32()) }
}

impl<const N: usize> SimdAllBitsImpl for ScalarMask<N> {
    #[inline(always)] fn all_zero(self) -> bool {
        self.0.iter().any(|&x| !x)
    }
}

impl<const N: usize> SimdBitwiseImpl for ScalarMask<N> {
    #[inline(always)] fn and(self, rhs: Self) -> Self { Self(std::array::from_fn(|i| self.0[i] & rhs.0[i])) }
    #[inline(always)] fn or(self, rhs: Self) -> Self { Self(std::array::from_fn(|i| self.0[i] | rhs.0[i])) }
    #[inline(always)] fn xor(self, rhs: Self) -> Self { Self(std::array::from_fn(|i| self.0[i] ^ rhs.0[i])) }
    #[inline(always)] fn not(self) -> Self { Self(std::array::from_fn(|i| !self.0[i])) }
    #[inline(always)] fn and_not(self, rhs: Self) -> Self { self.and(rhs.not()) }
}

impl<const N: usize> SimdNegateImpl for Scalar<N> {
    #[inline(always)] fn negate_f64(self) -> Self { Self::splat_64(-0.0f64).xor(self) }
    #[inline(always)] fn negate_f32(self) -> Self { Self::splat_32(-0.0f64).xor(self) }
}

impl<const N: usize> SimdBlockByteShiftImpl for Scalar<N> {
    #[inline(always)] fn block_left_byte_shift<const M: i32>(self) -> Self {
        let mut new = Self::splat_8(0);
        for block_start in (0..N).step_by(16) {
            let block_end = block_start + 16;
            for i in (block_start + M as usize)..block_end {
                new.0[i] = self.0[i - M as usize];
            }
        }
        new
    }
    #[inline(always)] fn block_right_byte_shift<const M: i32>(self) -> Self {
        let mut new = Self::splat_8(0);
        for block_start in (0..N).step_by(16) {
            let block_end = block_start + 16;
            for i in block_start..(block_end - M as usize) {
                new.0[i] = self.0[i + M as usize];
            }
        }
        new
    }
}