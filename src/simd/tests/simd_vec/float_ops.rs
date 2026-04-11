use crate::simd::architectures::arch_impl::*;
use crate::simd::simd_vec::core::SimdVec;
use crate::simd::simd_traits::*;
use crate::simd::arch_simd::{ArchSimd, ArchMask, ScalarSimd, ScalarMask};
use crate::simd::tests::simd_vec::generator::{test_vecs};
use crate::simd::tests::simd_vec::macros::{assert_simd_eq, simd_vec_test, simd_vec_tests};


// === Basic ===
simd_vec_tests!(splat_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x| x);

// === Permutes ===
fn blend_32_test_func<F: SimdFamily>(x: SimdVec<f32, F>, y: SimdVec<f32, F>) -> SimdVec<f32, F>
where
    SimdVec<f32, F>: SimdBlend,
    <F::Vec as SimdVariableBlendImpl>::MaskType: From<F::Mask>,
{
    x.blend_32(y, x.simd_gt(y))
}

#[test]
fn blend_32_test() {
    for (simd1, scalar1, simd2, scalar2) in itertools::izip!(
        test_vecs!(ArchSimd, f32),
        test_vecs!(ScalarSimd, f32),
        test_vecs!(ArchSimd, f32),
        test_vecs!(ScalarSimd, f32)
    ) {
        let simd_result = blend_32_test_func(simd1, simd2);
        let scalar_result = blend_32_test_func(scalar1, scalar2);
        assert_simd_eq!(
            inputs: [(stringify!(x), simd1), (stringify!(y), simd2)],
            output: (simd_result, scalar_result)
        );
    }
}

fn permute_32_test_func<F: SimdFamily>(x: SimdVec<f32, F>, y: SimdVec<u32, F>) -> SimdVec<f32, F> {
    let mask = SimdVec::<u32, F>::splat((F::SIMD_WIDTH as u32 / 4) - 1);
    x.permute_32(y & mask)
}

#[test]
fn permute_32_test() {
    for (simd1, scalar1, simd2, scalar2) in itertools::izip!(
        test_vecs!(ArchSimd, f32),
        test_vecs!(ScalarSimd, f32),
        test_vecs!(ArchSimd, u32),
        test_vecs!(ScalarSimd, u32)
    ) {
        let simd_result = permute_32_test_func(simd1, simd2);
        let scalar_result = permute_32_test_func(scalar1, scalar2);
        assert_simd_eq!(
            inputs: [(stringify!(x), simd1), (stringify!(y), simd2)],
            output: (simd_result, scalar_result)
        );
    }
}

fn permute_8_test_func<F: SimdFamily>(x: SimdVec<u8, F>, y: SimdVec<u8, F>) -> SimdVec<u8, F> {
    let mask = SimdVec::<u8, F>::splat((F::SIMD_WIDTH as u8) - 1);
    x.permute_8(y & mask)
}

#[test]
fn permute_8_test() {
    for (simd1, scalar1, simd2, scalar2) in itertools::izip!(
        test_vecs!(ArchSimd, u8),
        test_vecs!(ScalarSimd, u8),
        test_vecs!(ArchSimd, u8),
        test_vecs!(ScalarSimd, u8)
    ) {
        let simd_result = permute_8_test_func(simd1, simd2);
        let scalar_result = permute_8_test_func(scalar1, scalar2);
        assert_simd_eq!(
            inputs: [(stringify!(x), simd1), (stringify!(y), simd2)],
            output: (simd_result, scalar_result)
        );
    }
}

fn gather_32_test_func<F: SimdFamily>(x: SimdVec<u32, F>) -> SimdVec<f32, F> {
    let array: [f32; 32] = std::array::from_fn(|i| i as f32 * 10.0);
    let mask = SimdVec::<u32, F>::splat(32 - 1);
    (x & mask).gather(&array)
}

#[test]
fn gather_32_test() {
    for (simd, scalar) in itertools::izip!(
        test_vecs!(ArchSimd, u32),
        test_vecs!(ScalarSimd, u32),
    ) {
        let simd_result = gather_32_test_func(simd);
        let scalar_result = gather_32_test_func(scalar);
        assert_simd_eq!(
            inputs: [(stringify!(x), simd)],
            output: (simd_result, scalar_result)
        );
    }
}

// fn gather_64_test_func<F: SimdFamily>(x: SimdVec<u64, F>) -> SimdVec<f64, F> {
//     let array: [f64; 32] = std::array::from_fn(|i| i as f64 * 10.0);
//     let mask = SimdVec::<u64, F>::splat(32 - 1);
//     (x & mask).gather(&array)
// }

// #[test]
// fn gather_64_test() {
//     for (simd, scalar) in itertools::izip!(
//         test_vecs!(ArchSimd, u64),
//         test_vecs!(ScalarSimd, u64),
//     ) {
//         let simd_result = gather_64_test_func(simd);
//         let scalar_result = gather_64_test_func(scalar);
//         assert_simd_eq!(
//             inputs: [(stringify!(x), simd)],
//             output: (simd_result, scalar_result)
//         );
//     }
// }

// === Arithmetic ===
simd_vec_tests!(add_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x, y| x + y);
simd_vec_tests!(sub_test, [u8, u16, u32, u64, i8, i16, i32, i64, f32, f64], |x, y| x - y);
simd_vec_tests!(mul_test, [u16, u32, i16, i32, f32, f64], |x, y| x * y);
simd_vec_tests!(div_test, [f32, f64], |x, y| x / y);

simd_vec_tests!(and_test, [u8, u16, u32, u64], |x, y| x & y);
simd_vec_tests!(or_test, [u8, u16, u32, u64], |x, y| x | y);
simd_vec_tests!(xor_test, [u8, u16, u32, u64], |x, y| x ^ y);
simd_vec_tests!(andnot_test, [u8, u16, u32, u64], |x, y| x.andnot(y));

// === Floating Point Operations ===
simd_vec_tests!(round_test, [f32, f64], |x| x.round());
simd_vec_tests!(floor_test, [f32, f64], |x| x.floor());
simd_vec_tests!(ceil_test, [f32, f64], |x| x.ceil());
simd_vec_tests!(fract_test, [f32, f64], |x| x.fract());
simd_vec_tests!(sqrt_test, [f32, f64], |x| x.sqrt());
simd_vec_tests!(rsqrt_test, [f32], |x| x.rsqrt());
simd_vec_tests!(quintic_lerp_test, [f32, f64], |x| x.quintic_lerp());

simd_vec_tests!(cast_int_round_test, [[f32; i32], [f64; i64]], |x| -> { x.cast_int_round() });
simd_vec_tests!(cast_int_trunc_test, [[f32; i32], [f64; i64]], |x| -> { x.cast_int_trunc() });
simd_vec_tests!(cast_int_raw_test,   [[f32; i32], [f64; i64]], |x| -> { x.raw_cast() });

simd_vec_tests!(cast_uint_round_test, [[f32; u32], [f64; u64]], |x| -> { x.cast_uint_round() });
simd_vec_tests!(cast_uint_trunc_test, [[f32; u32], [f64; u64]], |x| -> { x.cast_uint_trunc() });
simd_vec_tests!(cast_uint_raw_test,   [[f32; u32], [f64; u64]], |x| -> { x.raw_cast() });

simd_vec_tests!(max_test, [f32, f64], |x, y| x.max(y));
simd_vec_tests!(min_test, [f32, f64], |x, y| x.min(y));
simd_vec_tests!(mul_add_test, [f32, f64], |x, y, z| x.mul_add(y, z));
simd_vec_tests!(mul_sub_test, [f32, f64], |x, y, z| x.mul_sub(y, z));
simd_vec_tests!(negated_mul_add, [f32, f64], |x, y, z| x.negated_mul_add(y, z));
simd_vec_tests!(negated_mul_sub, [f32, f64], |x, y, z| x.negated_mul_sub(y, z));