pub mod simd_utils {
    pub mod generator;
    pub mod macros;
}

use quick_noise::simd::{StaticSimd, ScalarSimd, StaticArch, ScalarArch};
use quick_noise::simd::architectures::interface::*;
use quick_noise::simd::array_trait::Array;
use quick_noise::simd::register::Simd;

// === Basic ===
simd_vec_tests!(
    splat_test,
    [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64],
    |x| { x }
);
simd_vec_tests!(
    load_store_test,
    [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64],
    |x| { Simd::from_slice(x.to_array().as_slice()) }
);
// TODO: Add lt for integers for integer partial loads.
simd_vec_tests!(partial_load_test, [f32, f64], |x| {
    Simd::partial_load(x.to_array().as_slice(), 2)
});
simd_vec_tests!(masked_load_test, [f32, u32], |x, y| {
    Simd::masked_load(x.to_array().as_slice(), x.simd_gt(y))
});
simd_vec_tests!(partial_store_test, [f32], |x, y| {
    let mut array = x.to_array();
    y.partial_store(array.as_mut_slice(), 2);
    Simd::from_slice(array.as_slice())
});
simd_vec_tests!(masked_store_test, [f32, u32], |x, y| {
    let mut array = x.to_array();
    y.masked_store(array.as_mut_slice(), x.simd_gt(y));
    Simd::from_slice(array.as_slice())
});
simd_vec_test!(zero_test, || -> u32 { Simd::zero() });

// === Permutes ===
simd_vec_tests!(vblend_32_test, [[f32, f32 -> f32]], |x, y| {
    (x.simd_gt(y)).select(x, y)
});

simd_vec_tests!(blend_32_test_1, [[f32, f32 -> f32]], |x, y| {
    x.blend::<0b0110>(y)
});

simd_vec_tests!(blend_32_test_2, [[f32, f32 -> f32]], |x, y| {
    x.blend::<0b1001>(y)
});

simd_vec_tests!(blend_32_test_3, [[f32, f32 -> f32]], |x, y| {
    x.blend::<0b1010>(y)
});

simd_vec_tests!(to_bits_test, [[f32, f32 -> f32]], |x, y| {
    Simd::splat(x.simd_gt(y).to_bits() as f32)
});

simd_vec_tests!(permute_32_test, [[f32, u32 -> f32], [i32, u32 -> i32], [u32, u32 -> u32]], |x, y| {
    let mask = Simd::<u32, A>::splat((A::SIMD_WIDTH as u32 / 4) - 1);
    x.permute_32(y & mask)
});

simd_vec_tests!(permute_8_test, [[f32, u8 -> f32], [i32, u8 -> i32], [u64, u8 -> u64], [u8, u8 -> u8]], |x, y| {
    let mask = Simd::<u8, A>::splat((A::SIMD_WIDTH as u8) - 1);
    x.permute_8(y & mask)
});

simd_vec_tests!(gather_32_test, [[u32 -> f32]], |x| {
    let array: [f32; 32] = std::array::from_fn(|i| i as f32 * 10.0);
    let mask = Simd::<u32, A>::splat(32 - 1);
    (x & mask).gather(&array)
});
simd_vec_tests!(gather_64_test, [[u64 -> f64]], |x| {
    let array: [f64; 32] = std::array::from_fn(|i| i as f64 * 10.0);
    let mask = Simd::<u64, A>::splat(32 - 1);
    (x & mask).gather(&array)
});

// === Comparisons ===
simd_vec_tests!(lt_test, [f32, f64], |x, y| { x.simd_lt(y).select(x, y) });
simd_vec_tests!(le_test, [f32, f64], |x, y| { x.simd_le(y).select(x, y) });
simd_vec_tests!(gt_test, [f32, f64, u32, u64, i32, i64], |x, y| {
    x.simd_gt(y).select(x, y)
});
simd_vec_tests!(ge_test, [f32, f64], |x, y| { x.simd_ge(y).select(x, y) });
simd_vec_tests!(eq_test, [f32, f64, u32, u64, i32, i64], |x, y| {
    x.simd_eq(y).select(x, y)
});
simd_vec_tests!(neq_test, [f32, f64, u32, u64, i32, i64], |x, y| {
    x.simd_neq(y).select(x, y)
});

simd_vec_tests!(float_int_cast_test, [[i32 -> f32]], |x| { x.cast_float() });
// TODO: Add mask tester.
// TODO: Add correct implementation for all_zero.
// simd_vec_tests!(all_false_test, [f32, f64], |x, y| {
//     if x.simd_gt(y).all_false() { x } else { y }
// });

simd_vec_tests!(and_mask_test, [f32, f64], |x, y, z| {
    (x.simd_gt(y) & y.simd_gt(z)).select(y, z)
});
simd_vec_tests!(or_mask_test, [f32, f64], |x, y, z| {
    (x.simd_gt(y) | y.simd_gt(z)).select(y, z)
});
simd_vec_tests!(xor_mask_test, [f32, f64], |x, y, z| {
    (x.simd_gt(y) ^ y.simd_gt(z)).select(y, z)
});
simd_vec_tests!(andnot_mask_test, [f32, f64], |x, y, z| {
    (x.simd_gt(y).andnot(y.simd_gt(z))).select(y, z)
});
simd_vec_tests!(not_mask_test, [f32, f64], |x, y| {
    (!x.simd_gt(y)).select(x, y)
});

// === Min/Max ===
simd_vec_tests!(min_test, [f32, f64, u8, u16, u32, i8, i16, i32], |x, y| {
    x.min(y)
});
simd_vec_tests!(max_test, [f32, f64, u8, u16, u32, i8, i16, i32], |x, y| {
    x.max(y)
});

// === Arithmetic ===
simd_vec_tests!(
    add_test,
    [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64],
    |x, y| { x + y }
);
simd_vec_tests!(
    sub_test,
    [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64],
    |x, y| { x - y }
);
simd_vec_tests!(mul_test, [u16, u32, i16, i32, f32, f64], |x, y| { x * y });
simd_vec_tests!(div_test, [f32, f64], |x, y| { x / y });

simd_vec_tests!(and_test, [u8, u16, u32, u64], |x, y| { x & y });
simd_vec_tests!(or_test, [u8, u16, u32, u64], |x, y| { x | y });
simd_vec_tests!(xor_test, [u8, u16, u32, u64], |x, y| { x ^ y });
simd_vec_tests!(andnot_test, [u8, u16, u32, u64], |x, y| { x.andnot(y) });
simd_vec_tests!(not_test, [u8, u16, u32, u64], |x| { !x });

// TODO: VARIABLE SHIATS DO NOT WORK ON SSE!
simd_vec_tests!(shl_scalar_test, [u32, i32], |x| { x << 10 });
simd_vec_tests!(shr_scalar_test, [u32, i32], |x| { x >> 10 });
simd_vec_tests!(shl_variable_test,
    [[u32, u32 -> u32], [i32, u32 -> i32]],
    |x, y| { x << (y.raw_cast() & Simd::splat(15))
});
simd_vec_tests!(shr_variable_test,
    [[u32, u32 -> u32], [i32, u32 -> i32]],
    |x, y| { x >> (y.raw_cast() & Simd::splat(15))
});

// === Clamp ===
// TODO: Add integer clamps.
simd_vec_tests!(clamp_test, [f32, f64], |x, y, z| { x.clamp(y, z) });

// === Aloating Point Operations ===
simd_vec_tests!(round_test, [f32, f64], |x| { x.round() });
simd_vec_tests!(floor_test, [f32, f64], |x| { x.floor() });
simd_vec_tests!(ceil_test, [f32, f64], |x| { x.ceil() });
simd_vec_tests!(fract_test, [f32, f64], |x| { x.fract() });
simd_vec_tests!(sqrt_test, [f32, f64], |x| { x.sqrt() });
// simd_vec_tests!(rsqrt_test, [f32], |x| { x.rsqrt() });
simd_vec_tests!(quintic_lerp_test, [f32, f64], |x| { x.quintic_lerp() });

// TODO: f64 casts.
// simd_vec_tests!(cast_int_round_test, [[f32 -> i32]], |x| { x.clamp(-1e9, 1e9).cast_int_round() });
// simd_vec_tests!(cast_int_trunc_test, [[f32 -> i32]], |x| { x.clamp(-1e9, 1e9).cast_int_trunc() });
simd_vec_tests!(cast_int_raw_test,   [[f32 -> i32]], |x| { x.raw_cast() });

// TODO: f64 casts.
// simd_vec_tests!(cast_uint_round_test, [[f32 -> u32]], |x| { x.abs().clamp_max(2e9).cast_uint_round() });
// simd_vec_tests!(cast_uint_trunc_test, [[f32 -> u32]], |x| { x.abs().clamp_max(2e9).cast_uint_trunc() });
simd_vec_tests!(cast_uint_raw_test,   [[f32 -> u32]], |x| { x.raw_cast() });

simd_vec_tests!(mul_add_test, [f32, f64], |x, y, z| { x.mul_add(y, z) });
simd_vec_tests!(mul_sub_test, [f32, f64], |x, y, z| { x.mul_sub(y, z) });
simd_vec_tests!(negated_mul_add, [f32, f64], |x, y, z| {
    x.negated_mul_add(y, z)
});
simd_vec_tests!(negated_mul_sub, [f32, f64], |x, y, z| {
    x.negated_mul_sub(y, z)
});

// === Lane shifts ===
simd_vec_tests!(block_left_byte_shift_test_1, [u8, f32], |x| {
    x.block_left_byte_shift::<1>()
});
simd_vec_tests!(block_left_byte_shift_test_2, [u8, f32], |x| {
    x.block_left_byte_shift::<2>()
});
simd_vec_tests!(block_left_byte_shift_test_3, [u8, f32], |x| {
    x.block_left_byte_shift::<4>()
});
simd_vec_tests!(block_left_byte_shift_test_4, [u8, f32], |x| {
    x.block_left_byte_shift::<10>()
});
simd_vec_tests!(block_right_byte_shift_test_1, [u8, f32], |x| {
    x.block_right_byte_shift::<1>()
});
simd_vec_tests!(block_right_byte_shift_test_2, [u8, f32], |x| {
    x.block_right_byte_shift::<2>()
});
simd_vec_tests!(block_right_byte_shift_test_3, [u8, f32], |x| {
    x.block_right_byte_shift::<4>()
});
simd_vec_tests!(block_right_byte_shift_test_4, [u8, f32], |x| {
    x.block_right_byte_shift::<10>()
});

simd_vec_tests!(left_lane_shift_2_test_32, [f32, u32, i32], |x| {
    x.left_lane_shift(2)
});
simd_vec_tests!(left_lane_shift_1_test_32, [f32, u32, i32], |x| {
    x.left_lane_shift(1)
});
simd_vec_tests!(left_lane_shift_0_test_32, [f32, u32, i32], |x| {
    x.left_lane_shift(0)
});
simd_vec_tests!(left_lane_shift_99_test_32, [f32, u32, i32], |x| {
    x.left_lane_shift(99)
});

simd_vec_tests!(right_lane_shift_2_test_32, [f32, u32, i32], |x| {
    x.right_lane_shift(2)
});
simd_vec_tests!(right_lane_shift_1_test_32, [f32, u32, i32], |x| {
    x.right_lane_shift(1)
});
simd_vec_tests!(right_lane_shift_0_test_32, [f32, u32, i32], |x| {
    x.right_lane_shift(0)
});
simd_vec_tests!(right_lane_shift_99_test_32, [f32, u32, i32], |x| {
    x.right_lane_shift(99)
});
