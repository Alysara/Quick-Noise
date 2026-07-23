use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, Ident, ItemFn, PatType, Token, parse_macro_input};

use crate::dispatch_simd::dispatch_simd_fn;

mod dispatch_simd;
mod enable_targets;

const FEATURES: &[(&str, &str, &str)] = if cfg!(target_arch = "x86_64") {
    &[
        ("Avx512", "avx512", "avx512f,fma"),
        ("Avx2", "avx2", "avx2,fma"),
        ("Sse", "sse4", "sse4.2"),
        ("Scalar128", "scalar", ""),
    ]
} else if cfg!(target_arch = "aarch64") {
    &[("Neon", "neon", "neon"), ("Scalar128", "scalar", "")]
} else {
    &[("Scalar128", "scalar", "")]
};

#[proc_macro_attribute]
pub fn dispatch_simd(args: TokenStream, item: TokenStream) -> TokenStream {
    dispatch_simd::dispatch_simd_entry(args, item)
}

#[proc_macro_attribute]
pub fn enable_targets(args: TokenStream, item: TokenStream) -> TokenStream {
    // panic!("{}", enable_targets::enable_targets_entry(args, item));
    enable_targets::enable_targets_entry(args, item)
}

pub(crate) fn quick_noise_path() -> TokenStream2 {
    match crate_name("quick-noise") {
        Ok(FoundCrate::Itself) => quote! { quick_noise },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::bongos },
    }
}

