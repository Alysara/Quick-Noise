// use crate::simd::architectures::arch_impl::*;
use crate::simd::traits::*;
// use std::ops::*;
// use num_traits::NumCast;
// use crate::simd::simd_vec::core::SimdVec;
// use crate::simd::simd_traits::*;
// use crate::simd::arch_simd::{ArchSimd, ArchMask, ScalarSimd, ScalarMask};
// use crate::simd::tests::simd_vec::generator::{apply};


pub trait SimdTestEq {
    fn test_eq(self, other: Self) -> bool;
}

fn approx_eq(a: f64, b: f64, rel_eps: f64) -> bool {
    if a == b { return true; } // handles inf == inf, 0.0 == 0.0
    let diff = (a - b).abs();
    let mag = a.abs().max(b.abs());
    diff / mag < rel_eps
}

impl<T: SimdElement> SimdTestEq for T {
    fn test_eq(self, other: Self) -> bool {
        match T::TYPE {
            SimdType::F32 | SimdType::F64 => {
                let a = self.to_f64().unwrap();
                let b = other.to_f64().unwrap();
                (!a.is_finite() && !b.is_finite()) || approx_eq(a, b, 5e-3)
            },
            _ => self == other
        }
    }
}

#[macro_export]
macro_rules! assert_simd_eq {
    (inputs: [$( ($input_name:expr, $input:expr) ),*], output: ($output_simd:expr, $output_scalar:expr)) => {{
        let simd_array = $output_simd.to_array();
        let scalar_array = $output_scalar.to_array();
        assert_eq!(
            simd_array.len(), scalar_array.len(),
            "Length mismatch! {} != {}", simd_array.len(), scalar_array.len()
        );
        for i in 0..simd_array.len() {
            if !crate::simd::tests::simd_vec::macros::SimdTestEq::test_eq(simd_array[i], scalar_array[i]) {
                let input_names = [$($input_name),*];
                let max_len = input_names.iter().map(|s| s.len()).max().unwrap_or(0);
                let label_width = max_len.max("scalar".len());

                let mut msg = String::from("Simd does not match!\nInputs:\n");
                $(
                    msg.push_str(&format!(
                        "  {:>width$}: {:?}\n",
                        $input_name, $input.to_array(),
                        width = label_width
                    ));
                )*
                msg.push_str(&format!(
                    "Output:\n  {:>width$}: {:?}\n  {:>width$}: {:?}",
                    "simd", simd_array,
                    "scalar", scalar_array,
                    width = label_width
                ));
                panic!("{}", msg);
            }
        }
    }};
}
pub use crate::assert_simd_eq;

#[macro_export]
macro_rules! simd_vec_test {
    // === 1 arg, inferred return type ===
    ($test_name:ident, |$x:ident: $elem_ty:ty| $body:expr) => {
        paste::paste! {
            fn [<$test_name _func>]<F: SimdFamily>($x: SimdVec<$elem_ty, F>) -> SimdVec<$elem_ty, F> $body
            #[test]
            fn $test_name() {
                for (simd, scalar) in itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty)
                ) {
                    let simd_result = [<$test_name _func>](simd);
                    let scalar_result = [<$test_name _func>](scalar);
                    crate::simd::tests::simd_vec::macros::assert_simd_eq!(
                        inputs: [(stringify!($x), simd)],
                        output: (simd_result, scalar_result)
                    );
                }
            }
        }
    };

    // === 2 args, inferred return type ===
    ($test_name:ident, |$x1:ident: $elem_ty1:ty, $x2:ident: $elem_ty2:ty| $body:expr) => {
        paste::paste! {
            fn [<$test_name _func>]<F: SimdFamily>(
                $x1: SimdVec<$elem_ty1, F>,
                $x2: SimdVec<$elem_ty2, F>
            ) -> SimdVec<$elem_ty1, F> $body
            #[test]
            fn $test_name() {
                let pairs1: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty1),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty1)
                ).collect();
                let pairs2: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty2),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty2)
                ).collect();
                for (simd1, scalar1) in &pairs1 {
                    for (simd2, scalar2) in &pairs2 {
                        let simd_result = [<$test_name _func>](*simd1, *simd2);
                        let scalar_result = [<$test_name _func>](*scalar1, *scalar2);
                        crate::simd::tests::simd_vec::macros::assert_simd_eq!(
                            inputs: [
                                (stringify!($x1), *simd1),
                                (stringify!($x2), *simd2)
                            ],
                            output: (simd_result, scalar_result)
                        );
                    }
                }
            }
        }
    };

    // === 3 args, inferred return type ===
    ($test_name:ident, |$x1:ident: $elem_ty1:ty, $x2:ident: $elem_ty2:ty, $x3:ident: $elem_ty3:ty| $body:expr) => {
        paste::paste! {
            fn [<$test_name _func>]<F: SimdFamily>(
                $x1: SimdVec<$elem_ty1, F>,
                $x2: SimdVec<$elem_ty2, F>,
                $x3: SimdVec<$elem_ty3, F>
            ) -> SimdVec<$elem_ty1, F> $body
            #[test]
            fn $test_name() {
                let pairs1: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty1),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty1)
                ).collect();
                let pairs2: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty2),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty2)
                ).collect();
                let pairs3: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty3),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty3)
                ).collect();
                for (simd1, scalar1) in &pairs1 {
                    for (simd2, scalar2) in &pairs2 {
                        for (simd3, scalar3) in &pairs3 {
                            let simd_result = [<$test_name _func>](*simd1, *simd2, *simd3);
                            let scalar_result = [<$test_name _func>](*scalar1, *scalar2, *scalar3);
                            crate::simd::tests::simd_vec::macros::assert_simd_eq!(
                                inputs: [
                                    (stringify!($x1), *simd1),
                                    (stringify!($x2), *simd2),
                                    (stringify!($x3), *simd3)
                                ],
                                output: (simd_result, scalar_result)
                            );
                        }
                    }
                }
            }
        }
    };

    // === 1 arg, explicit return type ===
    ($test_name:ident, |$x:ident: $elem_ty:ty| -> $ret_ty:ty { $body:expr }) => {
        paste::paste! {
            fn [<$test_name _func>]<F: SimdFamily>($x: SimdVec<$elem_ty, F>) -> SimdVec<$ret_ty, F> {
                $body
            }
            #[test]
            fn $test_name() {
                for (simd, scalar) in itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty)
                ) {
                    let simd_result = [<$test_name _func>](simd);
                    let scalar_result = [<$test_name _func>](scalar);
                    crate::simd::tests::simd_vec::macros::assert_simd_eq!(
                        inputs: [(stringify!($x), simd)],
                        output: (simd_result, scalar_result)
                    );
                }
            }
        }
    };

    // === 2 args, explicit return type ===
    ($test_name:ident, |$x1:ident: $elem_ty1:ty, $x2:ident: $elem_ty2:ty| -> $ret_ty:ty { $body:expr }) => {
        paste::paste! {
            fn [<$test_name _func>]<F: SimdFamily>(
                $x1: SimdVec<$elem_ty1, F>,
                $x2: SimdVec<$elem_ty2, F>
            ) -> SimdVec<$ret_ty, F> {
                $body
            }
            #[test]
            fn $test_name() {
                let pairs1: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty1),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty1)
                ).collect();
                let pairs2: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty2),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty2)
                ).collect();
                for (simd1, scalar1) in &pairs1 {
                    for (simd2, scalar2) in &pairs2 {
                        let simd_result = [<$test_name _func>](*simd1, *simd2);
                        let scalar_result = [<$test_name _func>](*scalar1, *scalar2);
                        crate::simd::tests::simd_vec::macros::assert_simd_eq!(
                            inputs: [
                                (stringify!($x1), *simd1),
                                (stringify!($x2), *simd2)
                            ],
                            output: (simd_result, scalar_result)
                        );
                    }
                }
            }
        }
    };

    // === 3 args, explicit return type ===
    ($test_name:ident, |$x1:ident: $elem_ty1:ty, $x2:ident: $elem_ty2:ty, $x3:ident: $elem_ty3:ty| -> $ret_ty:ty { $body:expr }) => {
        paste::paste! {
            fn [<$test_name _func>]<F: SimdFamily>(
                $x1: SimdVec<$elem_ty1, F>,
                $x2: SimdVec<$elem_ty2, F>,
                $x3: SimdVec<$elem_ty3, F>
            ) -> SimdVec<$ret_ty, F> {
                $body
            }
            #[test]
            fn $test_name() {
                let pairs1: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty1),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty1)
                ).collect();
                let pairs2: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty2),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty2)
                ).collect();
                let pairs3: Vec<_> = itertools::izip!(
                    crate::simd::tests::simd_vec::generator::test_vecs!(ArchSimd, $elem_ty3),
                    crate::simd::tests::simd_vec::generator::test_vecs!(ScalarSimd, $elem_ty3)
                ).collect();
                for (simd1, scalar1) in &pairs1 {
                    for (simd2, scalar2) in &pairs2 {
                        for (simd3, scalar3) in &pairs3 {
                            let simd_result = [<$test_name _func>](*simd1, *simd2, *simd3);
                            let scalar_result = [<$test_name _func>](*scalar1, *scalar2, *scalar3);
                            crate::simd::tests::simd_vec::macros::assert_simd_eq!(
                                inputs: [
                                    (stringify!($x1), *simd1),
                                    (stringify!($x2), *simd2),
                                    (stringify!($x3), *simd3)
                                ],
                                output: (simd_result, scalar_result)
                            );
                        }
                    }
                }
            }
        }
    };
}
pub use crate::simd_vec_test;

#[macro_export]
macro_rules! simd_vec_tests {
    // 1 arg
    ($name:ident, [$($ty:tt),+], |$x:ident| $body:block) => {
        paste::paste! {
            $(
                crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty>], |$x: $ty| $body);
            )+
        }
    };
    // 2 args
    ($name:ident, [$($ty:tt),+], |$x1:ident, $x2:ident| $body:block) => {
        paste::paste! {
            $(
                crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty>], |$x1: $ty, $x2: $ty| $body);
            )+
        }
    };
    // 3 args
    ($name:ident, [$($ty:tt),+], |$x1:ident, $x2:ident, $x3:ident| $body:block) => {
        paste::paste! {
            $(
                crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty>], |$x1: $ty, $x2: $ty, $x3: $ty| $body);
            )+
        }
    };
    // 1 arg, explicit return
    ($name:ident, [$([$arg_ty:tt -> $ret_ty:ty]),+], |$x:ident| -> $body:block) => {
        paste::paste! {
            $(
                crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $arg_ty>], |$x: $arg_ty| -> $ret_ty $body);
            )+
        }
    };
    // // 2 args same type, explicit return
    // ($name:ident, [$([$ty:tt -> $ret_ty:ty]),+], |$x1:ident, $x2:ident| -> $body:block) => {
    //     paste::paste! {
    //         $(
    //             crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty>], |$x1: $ty, $x2: $ty| -> $ret_ty $body);
    //         )+
    //     }
    // };
    // // 3 args same type, explicit return
    // ($name:ident, [$([$ty:tt -> $ret_ty:ty]),+], |$x1:ident, $x2:ident, $x3:ident| -> $body:block) => {
    //     paste::paste! {
    //         $(
    //             crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty>], |$x1: $ty, $x2: $ty, $x3: $ty| -> $ret_ty $body);
    //         )+
    //     }
    // };
    // 2 args different types, explicit return
    ($name:ident, [$([$ty1:tt, $ty2:tt -> $ret_ty:ty]),+], |$x1:ident, $x2:ident| $body:block) => {
        paste::paste! {
            $(
                crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty1 _ $ty2>], |$x1: $ty1, $x2: $ty2| -> $ret_ty $body);
            )+
        }
    };
    // 3 args different types, explicit return
    ($name:ident, [$([$ty1:tt, $ty2:tt, $ty3:tt -> $ret_ty:ty]),+], |$x1:ident, $x2:ident, $x3:ident| -> $body:block) => {
        paste::paste! {
            $(
                crate::simd::tests::simd_vec::macros::simd_vec_test!([<$name _ $ty1 _ $ty2 _ $ty3>], |$x1: $ty1, $x2: $ty2, $x3: $ty3| -> $ret_ty $body);
            )+
        }
    };
}
pub use crate::simd_vec_tests;
