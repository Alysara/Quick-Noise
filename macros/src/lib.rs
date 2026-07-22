use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, Ident, ItemFn, PatType, Token, parse_macro_input};

use crate::dispatch_simd::dispatch_simd_impl;

mod dispatch_simd;
mod enable_targets;

#[proc_macro_attribute]
pub fn dispatch_simd(args: TokenStream, item: TokenStream) -> TokenStream {
    dispatch_simd::dispatch_simd_impl(args, item)
}

#[proc_macro_attribute]
pub fn enable_targets(args: TokenStream, item: TokenStream) -> TokenStream {
    enable_targets::enable_targets_impl(args, item)
}

