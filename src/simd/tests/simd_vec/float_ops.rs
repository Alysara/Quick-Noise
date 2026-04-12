use crate::simd::architectures::arch_impl::*;
use crate::simd::simd_vec::core::SimdVec;
use crate::simd::simd_traits::*;
use crate::simd::arch_simd::{ArchSimd, ArchMask, ScalarSimd, ScalarMask};
use crate::simd::tests::simd_vec::generator::{test_vecs};
use crate::simd::tests::simd_vec::macros::{assert_simd_eq, simd_vec_test, simd_vec_tests};
use crate::simd::array_trait::Array;

// === Basic ===
simd_vec_tests!(splat_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x| { x });
simd_vec_tests!(load_store_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x| {
    SimdVec::load(x.to_array().as_slice())
});
// TODO: Add lt for integers for integer partial loads.
simd_vec_tests!(partial_load_test, [f32, f64], |x| {
    SimdVec::partial_load(x.to_array().as_slice(), 2)
});


// === Permutes ===
simd_vec_tests!(blend_32_test, [[f32, f32 -> f32]], |x, y| {
    (x.simd_gt(y)).select(x, y)
});

simd_vec_tests!(permute_32_test, [[f32, u32 -> f32]], |x, y| {
    let mask = SimdVec::<u32, F>::splat((F::SIMD_WIDTH as u32 / 4) - 1);
    x.permute_32(y & mask)
});

simd_vec_tests!(permute_8_test, [u8], |x, y| {
    let mask = SimdVec::<u8, F>::splat((F::SIMD_WIDTH as u8) - 1);
    x.permute_8(y & mask)
});

simd_vec_tests!(gather_32_test, [[u32 -> f32]], |x| {
    let array: [f32; 32] = std::array::from_fn(|i| i as f32 * 10.0);
    let mask = SimdVec::<u32, F>::splat(32 - 1);
    (x & mask).gather(&array)
});

// === Arithmetic ===
simd_vec_tests!(add_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x, y| { x + y });
simd_vec_tests!(sub_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x, y| { x - y });
simd_vec_tests!(mul_test, [u16, u32, i16, i32, f32, f64], |x, y| { x * y });
simd_vec_tests!(div_test, [f32, f64], |x, y| { x / y });

simd_vec_tests!(and_test, [u8, u16, u32, u64], |x, y| { x & y });
simd_vec_tests!(or_test, [u8, u16, u32, u64], |x, y| { x | y });
simd_vec_tests!(xor_test, [u8, u16, u32, u64], |x, y| { x ^ y });
simd_vec_tests!(andnot_test, [u8, u16, u32, u64], |x, y| { x.andnot(y) });

// === Clamp ===
// TODO: Add integer clamps.
simd_vec_tests!(clamp_min_test, [f32, f64], |x| { x.clamp_min(0.) });
simd_vec_tests!(clamp_max_test, [f32, f64], |x| { x.clamp_min(0.) });
simd_vec_tests!(clamp_test, [f32, f64], |x| { x.clamp(-1000., 1000.) });

// === Floating Point Operations ===
simd_vec_tests!(round_test, [f32, f64], |x| { x.round() });
simd_vec_tests!(floor_test, [f32, f64], |x| { x.floor() });
simd_vec_tests!(ceil_test, [f32, f64], |x| { x.ceil() });
simd_vec_tests!(fract_test, [f32, f64], |x| { x.fract() });
simd_vec_tests!(sqrt_test, [f32, f64], |x| { x.sqrt() });
simd_vec_tests!(rsqrt_test, [f32], |x| { x.rsqrt() });
simd_vec_tests!(quintic_lerp_test, [f32, f64], |x| { x.quintic_lerp() });

// TODO: f64 casts.
simd_vec_tests!(cast_int_round_test, [[f32 -> i32]], |x| { x.clamp(-1e9, 1e9).cast_int_round() });
simd_vec_tests!(cast_int_trunc_test, [[f32 -> i32]], |x| { x.clamp(-1e9, 1e9).cast_int_trunc() });
simd_vec_tests!(cast_int_raw_test,   [[f32 -> i32]], |x| { x.raw_cast() });

// TODO: f64 casts.
simd_vec_tests!(cast_uint_round_test, [[f32 -> u32]], |x| { x.abs().clamp_max(2e9).cast_uint_round() });
simd_vec_tests!(cast_uint_trunc_test, [[f32 -> u32]], |x| { x.abs().clamp_max(2e9).cast_uint_trunc() });
simd_vec_tests!(cast_uint_raw_test,   [[f32 -> u32]], |x| { x.raw_cast() });

simd_vec_tests!(max_test, [f32, f64], |x, y| { x.max(y) });
simd_vec_tests!(min_test, [f32, f64], |x, y| { x.min(y) });
simd_vec_tests!(mul_add_test, [f32, f64], |x, y, z| { x.mul_add(y, z) });
simd_vec_tests!(mul_sub_test, [f32, f64], |x, y, z| { x.mul_sub(y, z) });
simd_vec_tests!(negated_mul_add, [f32, f64], |x, y, z| { x.negated_mul_add(y, z) });
simd_vec_tests!(negated_mul_sub, [f32, f64], |x, y, z| { x.negated_mul_sub(y, z) });