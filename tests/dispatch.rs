//! Stress test for #[dispatch_simd(A)].
//!
//! Aims to ensure that the macro compiles correctly in
//! a wide variety of conditions. Runtime behavior is not
//! the focus.

#![allow(
    dead_code,
    unused_variables,
    unused_mut,
    clippy::extra_unused_type_parameters,
    clippy::extra_unused_lifetimes,
    unused_parens,
    unused_braces,
    clippy::needless_lifetimes,
    unused_assignments
)]

use std::iter::Zip;
use std::ops::{Index, Mul};

use quick_noise_macros::dispatch_simd;

// ============================================================
// Baseline
// ============================================================

#[dispatch_simd(A)]
fn baseline(val: usize) {}

// ============================================================
// Visibility
// ============================================================

#[dispatch_simd(A)]
pub fn vis_pub(val: usize) {}

#[dispatch_simd(A)]
pub(crate) fn vis_pub_crate(val: usize) {}

mod inner_mod {
    use quick_noise_macros::dispatch_simd;

    #[dispatch_simd(A)]
    pub(super) fn vis_pub_super(val: usize) {}

    #[dispatch_simd(A)]
    pub(crate) fn vis_pub_in_path(val: usize) {}
}

// ============================================================
// Modifiers
// ============================================================

#[dispatch_simd(A)]
async fn mod_async(val: usize) {}

#[dispatch_simd(A)]
unsafe fn mod_unsafe(val: usize) {}

#[dispatch_simd(A)]
extern "C" fn mod_extern_c(val: usize) {}

#[allow(missing_abi)]
#[dispatch_simd(A)]
extern "C" fn mod_extern_implicit(val: usize) {}

// ============================================================
// Modifier combinations
// ============================================================

#[dispatch_simd(A)]
pub unsafe fn combo_pub_unsafe(val: usize) {}

#[dispatch_simd(A)]
pub async fn combo_pub_async(val: usize) {}

#[dispatch_simd(A)]
pub(crate) unsafe extern "C" fn combo_pub_crate_unsafe_extern(val: usize) {}

// ============================================================
// Generics without bounds
// ============================================================

#[dispatch_simd(A)]
fn generic_single<T>(val: T) {}

#[dispatch_simd(A)]
fn generic_multi<T, U>(a: T, b: U) {}

#[dispatch_simd(A)]
fn generic_trailing_comma<T, U>(a: T, b: U) {}

// ============================================================
// Generics with bounds
// ============================================================

#[dispatch_simd(A)]
fn bound_single<T: Clone>(val: T) {}

#[dispatch_simd(A)]
fn bound_multi_plus<T: Clone + Send + 'static>(val: T) {}

#[dispatch_simd(A)]
fn bound_multi_params<T: Clone, U: Default>(a: T, b: U) {}

#[dispatch_simd(A)]
fn bound_nested_generic<T: Into<Vec<u8>>>(val: T) {}

#[dispatch_simd(A)]
fn bound_fn_trait<F: Fn(i32) -> bool>(f: F) {}

// ============================================================
// Lifetimes
// ============================================================

#[dispatch_simd(A)]
fn lifetime_single<'a>(val: &'a str) {}

#[dispatch_simd(A)]
fn lifetime_multi<'a, 'b>(x: &'a str, y: &'b str) {}

#[dispatch_simd(A)]
fn lifetime_plus_type<'a, T>(x: &'a T) {}

#[dispatch_simd(A)]
fn lifetime_bound<'a, T: 'a>(x: &'a T) {}

// ============================================================
// Const generics
// ============================================================

#[dispatch_simd(A)]
fn const_generic<const N: usize>(arr: [f32; N]) {}

#[dispatch_simd(A)]
fn const_generic_mixed<T, const N: usize>(arr: [T; N]) {}

#[dispatch_simd(A)]
fn const_generic_multi<const N: usize, const M: usize>(a: [f32; N], b: [f32; M]) {}

#[dispatch_simd(A)]
fn tuple_generic<T: Index<(f32, u32)>>(weird_struct: T) {}

// ============================================================
// Mixed generics
// ============================================================

#[dispatch_simd(A)]
fn mixed_kitchen_sink<'a, T: Clone + Send, U: Default, const N: usize>(
    x: &'a T,
    y: U,
    arr: [f32; N],
) {
}

#[dispatch_simd(A)]
fn nested_angle_brackets<T: Into<Vec<Box<dyn Fn(i32) -> Option<T>>>>>(val: T) {}

// ============================================================
// Where clauses
// ============================================================

#[dispatch_simd(A)]
fn where_simple<T>(val: T)
where
    T: Clone,
{
}

#[dispatch_simd(A)]
fn where_multi<T, U>(a: T, b: U)
where
    T: Clone + Send,
    U: Default,
{
}

// ============================================================
// Parameter shapes
// ============================================================

#[dispatch_simd(A)]
fn params_none() {}

#[dispatch_simd(A)]
fn params_single(val: usize) {}

#[dispatch_simd(A)]
fn params_multi(a: usize, b: f32, c: bool) {}

#[dispatch_simd(A)]
fn params_trailing_comma(a: usize, b: f32) {}

#[dispatch_simd(A)]
fn params_mut(mut val: usize) {
    val += 1;
}

#[dispatch_simd(A)]
fn params_ref(val: &usize) {}

#[dispatch_simd(A)]
fn params_mut_ref(val: &mut usize) {}

#[dispatch_simd(A)]
fn params_ref_with_lifetime<'a>(val: &'a usize) {}

#[dispatch_simd(A)]
fn params_nested_generic_type(val: std::collections::HashMap<String, Vec<u8>>) {}

#[dispatch_simd(A)]
fn params_fn_pointer(f: fn(i32) -> i32) {}

#[dispatch_simd(A)]
fn params_impl_trait(f: impl Fn(i32) -> bool) {}

#[dispatch_simd(A)]
fn params_array_type(arr: [f32; 16]) {}

#[dispatch_simd(A)]
fn params_slice_type(arr: &[f32]) {}

#[dispatch_simd(A)]
fn params_tuple_type(pair: (i32, f32)) {}

// ============================================================
// Return types
// ============================================================

#[dispatch_simd(A)]
fn ret_simple(val: usize) -> usize {
    val
}

#[dispatch_simd(A)]
fn ret_generic<T: Default>(val: T) -> T {
    val
}

#[dispatch_simd(A)]
fn ret_complex(val: usize) -> Result<Vec<u8>, std::io::Error> {
    Ok(Vec::new())
}

#[dispatch_simd(A)]
fn ret_tuple(val: usize) -> (usize, usize) {
    (val, val)
}

#[dispatch_simd(A)]
fn ret_reference<'a>(val: &'a str) -> &'a str {
    val
}

// ============================================================
// Body edge cases
// ============================================================

#[dispatch_simd(A)]
fn body_nested_braces(val: usize) {
    {
        {
            let _ = val;
        }
    }
}

#[dispatch_simd(A)]
fn body_closure_with_braces(val: usize) {
    let f = |x: usize| -> usize { x + 1 };
    let _ = f(val);
}

#[dispatch_simd(A)]
fn body_string_with_braces(val: usize) {
    let _s = "this has { braces } and < angle > brackets inside a string";
    let _c = '{';
}

#[dispatch_simd(A)]
fn body_generic_turbofish_call(val: usize) {
    let v = Vec::<u8>::new();
}

#[dispatch_simd(A)]
fn body_comparison_ops(val: usize) {
    let _ = val < 10 && val > 0;
}

// ============================================================
// Doc comments / attributes on the function itself
// ============================================================

/// A documented function.
#[dispatch_simd(A)]
fn with_doc_comment(val: usize) {}

#[allow(dead_code)]
#[dispatch_simd(A)]
fn with_other_attribute_before(val: usize) {}

#[dispatch_simd(A)]
#[allow(dead_code)]
fn with_other_attribute_after(val: usize) {}

// TODO: Work for methods with self.
struct Kernel;

impl Kernel {
    #[dispatch_simd(A)]
    fn method_ref_self(&self, val: usize) -> usize {
        val
    }

    #[dispatch_simd(A)]
    fn method_mut_self(&mut self, val: usize) {}

    #[dispatch_simd(A)]
    fn method_owned_self(self, val: usize) -> usize {
        val
    }

    #[dispatch_simd(A)]
    fn assoc_fn_no_self(val: usize) -> usize {
        val
    }
}

struct ComplexKernel<
    'a,
    'b,
    const N: usize,
    T: Clone + Iterator<Item = (f32, f32, (u32, usize))>,
    U,
> where
    U: Clone + Default + Mul<Output = Self>,
{
    ref_a: &'a f32,
    ref_b: &'b f32,
    it: T,
    val: U,
}

#[repr(C)]
struct ArrayWrapper<const N: usize> {
    array: [f32; N],
}

impl<const N: usize> ArrayWrapper<N> {
    #[dispatch_simd(A)]
    pub fn new(val: f32) -> Self {
        Self { array: [val; N] }
    }
}

impl<'a, 'b, const N: usize, T: Clone + Iterator<Item = (f32, f32, (u32, usize))>, U>
    ComplexKernel<'a, 'b, N, T, U>
where
    U: Clone + Default + Mul<Output = Self>,
{
    #[dispatch_simd(A)]
    pub(crate) unsafe fn simple(val: usize, vec: Vec<(f32, u32)>) -> (((f32))) {
        1.0
    }

    #[dispatch_simd(A)]
    pub(crate) unsafe extern "C" fn extern_example(&self, val: usize) -> ArrayWrapper<{ 6 }> {
        ArrayWrapper { array: [0.0; 6] }
    }

    #[dispatch_simd(A)]
    async fn async_generics<'c, 'd, 'e, L: Clone + Default, M>(
        mut self,
        a: L,
        b: M,
    ) -> Vec<Box<(u32, f32)>>
    where
        M: Mul<Output = Self>,
        Zip<L, M>: Default,
    {
        Vec::new()
    }

    #[dispatch_simd(A, associated)]
    fn impl_gen_ret() -> U {
        U::default()
    }
}

fn deeply_nested<T>(val: Option<Result<Vec<Vec<T>>, ()>>) {}

#[dispatch_simd(A)]
fn higher_ranked<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
}

fn main() {
    baseline(1);
    vis_pub(1);
    vis_pub_crate(1);
    unsafe { mod_unsafe(1) };
    generic_single(1usize);
    bound_single(1usize);
    lifetime_single("x");
    const_generic([0.0; 4]);
    where_simple(1usize);
    params_none();
    params_single(1);
    params_mut(1);
    params_ref(&1);
    ret_simple(1);
}
