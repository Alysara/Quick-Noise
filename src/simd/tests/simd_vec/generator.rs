#[macro_export]
macro_rules! test_vecs {
    ($simd_vec:ident, u8) => {
        vec![
            $simd_vec::<u8>::splat(0),
            $simd_vec::<u8>::splat(1),
            $simd_vec::<u8>::splat(2),
            $simd_vec::<u8>::splat(3),
            $simd_vec::<u8>::splat(7),
            $simd_vec::<u8>::splat(13),
            $simd_vec::<u8>::splat(100),
            $simd_vec::<u8>::splat(200),
            $simd_vec::<u8>::splat(253),
            $simd_vec::<u8>::splat(254),
            $simd_vec::<u8>::splat(255),
            $simd_vec::<u8>::iota(0),
            $simd_vec::<u8>::iota(1),
            $simd_vec::<u8>::iota(24),
            $simd_vec::<u8>::iota(100),
            $simd_vec::<u8>::iota(222),
        ]
    };
    ($simd_vec:ident, u16) => {
        vec![
            $simd_vec::<u16>::splat(0),
            $simd_vec::<u16>::splat(1),
            $simd_vec::<u16>::splat(2),
            $simd_vec::<u16>::splat(10),
            $simd_vec::<u16>::splat(100),
            $simd_vec::<u16>::splat(1000),
            $simd_vec::<u16>::splat(12345),
            $simd_vec::<u16>::splat(60000),
            $simd_vec::<u16>::splat(65534),
            $simd_vec::<u16>::splat(65535),
            $simd_vec::<u16>::iota(0),
            $simd_vec::<u16>::iota(1),
            $simd_vec::<u16>::iota(1000),
            $simd_vec::<u16>::iota(12345),
        ]
    };
    ($simd_vec:ident, u32) => {
        vec![
            $simd_vec::<u32>::splat(0),
            $simd_vec::<u32>::splat(1),
            $simd_vec::<u32>::splat(2),
            $simd_vec::<u32>::splat(10),
            $simd_vec::<u32>::splat(100),
            $simd_vec::<u32>::splat(1_000),
            $simd_vec::<u32>::splat(10_000),
            $simd_vec::<u32>::splat(123_456),
            $simd_vec::<u32>::splat(1_000_000),
            $simd_vec::<u32>::splat(u32::MAX - 1),
            $simd_vec::<u32>::splat(u32::MAX),
            $simd_vec::<u32>::iota(0),
            $simd_vec::<u32>::iota(1),
            $simd_vec::<u32>::iota(100),
            $simd_vec::<u32>::iota(10_000),
            $simd_vec::<u32>::iota(100_000),
        ]
    };
    ($simd_vec:ident, u64) => {
        vec![
            $simd_vec::<u64>::splat(0),
            $simd_vec::<u64>::splat(1),
            $simd_vec::<u64>::splat(2),
            $simd_vec::<u64>::splat(10),
            $simd_vec::<u64>::splat(100),
            $simd_vec::<u64>::splat(1_000),
            $simd_vec::<u64>::splat(1_000_000),
            $simd_vec::<u64>::splat(1_000_000_000),
            $simd_vec::<u64>::splat(u64::MAX - 1),
            $simd_vec::<u64>::splat(u64::MAX),
            $simd_vec::<u64>::iota(0),
            $simd_vec::<u64>::iota(1),
            $simd_vec::<u64>::iota(100),
            $simd_vec::<u64>::iota(1_000_000),
            $simd_vec::<u64>::iota(10_000_000),
        ]
    };
    ($simd_vec:ident, i8) => {
        vec![
            $simd_vec::<i8>::splat(0),
            $simd_vec::<i8>::splat(1),
            $simd_vec::<i8>::splat(-1),
            $simd_vec::<i8>::splat(2),
            $simd_vec::<i8>::splat(-2),
            $simd_vec::<i8>::splat(10),
            $simd_vec::<i8>::splat(-10),
            $simd_vec::<i8>::splat(50),
            $simd_vec::<i8>::splat(-50),
            $simd_vec::<i8>::splat(i8::MIN),
            $simd_vec::<i8>::splat(i8::MAX),
            $simd_vec::<i8>::iota(0),
            $simd_vec::<i8>::iota(-1),
            $simd_vec::<i8>::iota(10),
        ]
    };
    ($simd_vec:ident, i16) => {
        vec![
            $simd_vec::<i16>::splat(0),
            $simd_vec::<i16>::splat(1),
            $simd_vec::<i16>::splat(-1),
            $simd_vec::<i16>::splat(100),
            $simd_vec::<i16>::splat(-100),
            $simd_vec::<i16>::splat(12345),
            $simd_vec::<i16>::splat(-12345),
            $simd_vec::<i16>::splat(i16::MIN),
            $simd_vec::<i16>::splat(i16::MAX),
            $simd_vec::<i16>::iota(0),
            $simd_vec::<i16>::iota(-10),
            $simd_vec::<i16>::iota(1000),
        ]
    };
    ($simd_vec:ident, i32) => {
        vec![
            $simd_vec::<i32>::splat(0),
            $simd_vec::<i32>::splat(1),
            $simd_vec::<i32>::splat(-1),
            $simd_vec::<i32>::splat(2),
            $simd_vec::<i32>::splat(-2),
            $simd_vec::<i32>::splat(10),
            $simd_vec::<i32>::splat(-10),
            $simd_vec::<i32>::splat(100),
            $simd_vec::<i32>::splat(-100),
            $simd_vec::<i32>::splat(1_000),
            $simd_vec::<i32>::splat(-1_000),
            $simd_vec::<i32>::splat(123_456),
            $simd_vec::<i32>::splat(-123_456),
            $simd_vec::<i32>::splat(i32::MAX - 1),
            $simd_vec::<i32>::splat(i32::MAX),
            $simd_vec::<i32>::splat(i32::MIN + 1),
            $simd_vec::<i32>::splat(i32::MIN),
            $simd_vec::<i32>::iota(0),
            $simd_vec::<i32>::iota(-1),
            $simd_vec::<i32>::iota(10),
            $simd_vec::<i32>::iota(1_000),
            $simd_vec::<i32>::iota(100_000),
        ]
    };
    ($simd_vec:ident, i64) => {
        vec![
            $simd_vec::<i64>::splat(0),
            $simd_vec::<i64>::splat(1),
            $simd_vec::<i64>::splat(-1),
            $simd_vec::<i64>::splat(2),
            $simd_vec::<i64>::splat(-2),
            $simd_vec::<i64>::splat(10),
            $simd_vec::<i64>::splat(-10),
            $simd_vec::<i64>::splat(100),
            $simd_vec::<i64>::splat(-100),
            $simd_vec::<i64>::splat(1_000),
            $simd_vec::<i64>::splat(-1_000),
            $simd_vec::<i64>::splat(1_000_000),
            $simd_vec::<i64>::splat(-1_000_000),
            $simd_vec::<i64>::splat(i64::MAX - 1),
            $simd_vec::<i64>::splat(i64::MAX),
            $simd_vec::<i64>::splat(i64::MIN + 1),
            $simd_vec::<i64>::splat(i64::MIN),
            $simd_vec::<i64>::iota(0),
            $simd_vec::<i64>::iota(-1),
            $simd_vec::<i64>::iota(10),
            $simd_vec::<i64>::iota(1_000),
        ]
    };
    ($simd_vec:ident, f32) => {
        vec![
            $simd_vec::<f32>::splat(0.0),
            $simd_vec::<f32>::splat(1.0),
            $simd_vec::<f32>::splat(-1.0),
            $simd_vec::<f32>::splat(0.5),
            $simd_vec::<f32>::splat(-0.5),
            $simd_vec::<f32>::splat(1.1234),
            $simd_vec::<f32>::splat(-2.1234),
            $simd_vec::<f32>::splat(1e-3),
            $simd_vec::<f32>::splat(-1e-3),
            $simd_vec::<f32>::splat(1e6),
            $simd_vec::<f32>::splat(-1e6),
            // $simd_vec::<f32>::splat(f32::MAX),
            // $simd_vec::<f32>::splat(f32::MIN),
            $simd_vec::<f32>::iota(0.0),
            $simd_vec::<f32>::iota(1.0),
            $simd_vec::<f32>::iota(10.0),
        ]
    };
    ($simd_vec:ident, f64) => {
        vec![
            $simd_vec::<f64>::splat(0.0),
            $simd_vec::<f64>::splat(1.0),
            $simd_vec::<f64>::splat(-1.0),
            $simd_vec::<f64>::splat(0.25),
            $simd_vec::<f64>::splat(-0.25),
            $simd_vec::<f64>::splat(1.23456789),
            $simd_vec::<f64>::splat(-9.87654321),
            $simd_vec::<f64>::splat(1e10),
            $simd_vec::<f64>::splat(-1e10),
            // $simd_vec::<f64>::splat(f64::MAX),
            // $simd_vec::<f64>::splat(f64::MIN),
            $simd_vec::<f64>::iota(0.0),
            $simd_vec::<f64>::iota(1.0),
            $simd_vec::<f64>::iota(10.0),
        ]
    };
}
pub use crate::test_vecs;
