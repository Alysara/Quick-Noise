use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    FnArg, GenericParam, Ident, ImplItem, ItemFn, ItemImpl, PatType, Token, parse_macro_input,
};

use crate::FEATURES;

struct DispatchArgs {
    arch: Ident,
    is_impl: bool,
}

impl Parse for DispatchArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arch: Ident = input.parse()?;
        let is_impl = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let kw: Ident = input.parse()?;
            if kw != "associated" {
                return Err(syn::Error::new(
                    kw.span(),
                    "expected `associated`; enable_targets takes an Arch identifier \
                     and an optional `associated` flag",
                ));
            }
            true
        } else {
            false
        };
        Ok(Self { arch, is_impl })
    }
}

/// Returns the path segment to use in place of a hardcoded `quick_noise::...`.
fn quick_noise_path() -> TokenStream2 {
    match crate_name("quick-noise") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
        Err(_) => quote! { ::bongos }, // fallback if lookup fails
    }
}

/// Bare identifier for a type/const generic param (no bounds) -- for call-site turbofishes.
/// Caller must have already filtered out lifetimes.
fn bare_ident(p: &GenericParam) -> TokenStream2 {
    match p {
        GenericParam::Type(t) => {
            let i = &t.ident;
            quote! { #i }
        }
        GenericParam::Const(c) => {
            let i = &c.ident;
            quote! { #i }
        }
        GenericParam::Lifetime(_) => unreachable!("lifetimes must be filtered out first"),
    }
}

/// Entry point: route based on whether the item is a function or an impl block.
pub fn enable_targets_entry(args: TokenStream, item: TokenStream) -> TokenStream {
    if syn::parse::<ItemImpl>(item.clone()).is_ok() {
        enable_targets_impl(args, item)
    } else {
        enable_targets_fn(args, item)
    }
}

// =====================================================================================
// Function 1 -- a single function that already declares `A` as one of its own generics.
// Unlike `dispatch_simd`, nothing is injected: the macro reroutes based on whichever
// `A` is already in scope, resolved via `A::ARCHITECTURE` (compile-time, per
// monomorphization) rather than a runtime `DETECTED_ARCH` check.
// =====================================================================================
pub fn enable_targets_fn(args: TokenStream, item: TokenStream) -> TokenStream {
    let DispatchArgs { arch, mut is_impl } = parse_macro_input!(args as DispatchArgs);
    let crate_path = quick_noise_path();

    let mut func = parse_macro_input!(item as ItemFn);

    if func.sig.constness.is_some() {
        return syn::Error::new_spanned(
            func.sig.constness,
            "dynamic dispatch does not work in a const context! \
             use StaticArch and StaticSimd for static dispatch.",
        )
        .to_compile_error()
        .into();
    }

    is_impl |= func
        .sig
        .inputs
        .iter()
        .any(|a| matches!(a, FnArg::Receiver(_)));
    is_impl |= matches!(&func.sig.output, syn::ReturnType::Type(_, ty) if quote::quote!(#ty).to_string()
        .split_whitespace()
        .any(|t| t == "Self"));

    let fn_name = func.sig.ident.clone();
    let unsafety = func.sig.safety.clone();
    let asyncness = func.sig.asyncness;
    let output = func.sig.output.clone();
    let inputs = func.sig.inputs.clone();
    let body = func.block.clone();
    let (impl_generics, _, where_clause) = func.sig.generics.split_for_impl();

    let lifetimes: Vec<TokenStream2> = func
        .sig
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Lifetime(l) => {
                let lt = &l.lifetime;
                Some(quote! { #lt })
            }
            _ => None,
        })
        .collect();

    // Full type/const params (bounds included), excluding lifetimes and `arch` itself.
    // Used to declare each per-variant wrapper fn's own generics.
    let ty_full: Vec<TokenStream2> = func
        .sig
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, GenericParam::Lifetime(_)))
        .filter(|p| !matches!(p, GenericParam::Type(t) if t.ident == arch))
        .map(|p| quote! { #p })
        .collect();

    // Bare idents only, excluding lifetimes and `arch` -- for call-site turbofishes.
    let ty_generics: Vec<TokenStream2> = func
        .sig
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, GenericParam::Lifetime(_)))
        .filter(|p| !matches!(p, GenericParam::Type(t) if t.ident == arch))
        .map(bare_ident)
        .collect();

    let call_args: Vec<TokenStream2> = inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => quote! { self },
            FnArg::Typed(PatType { pat, .. }) => match &**pat {
                syn::Pat::Ident(pat_ident) => {
                    let ident = &pat_ident.ident;
                    quote! { #ident }
                }
                _ => quote! { #pat },
            },
        })
        .collect();

    let impl_name = format_ident!("__{}_impl", fn_name);

    // Shared generic body. Reuses the user's own generics (including `A: Arch`)
    // as written -- nothing injected here.
    let impl_fn = quote! {
        #[inline(always)]
        #unsafety #asyncness fn #impl_name #impl_generics (#inputs) #output
            #where_clause
        {
            #body
        }
    };

    // Generics to declare on each per-variant wrapper fn: original list minus `A`.
    let wrapper_generics_decl = quote! { <#(#lifetimes,)* #(#ty_full),*> };

    let self_prefix = is_impl.then(|| quote!(Self::));
    let await_suffix = asyncness.is_some().then(|| quote! { .await });
    let turbofish = (!ty_generics.is_empty())
        .then(|| quote! { ::<#(#lifetimes,)* #(#ty_generics),*> });

    let mut variant_fns = Vec::new();
    let mut match_arms = Vec::new();

    for (variant, label, flags) in FEATURES {
        let variant_ident = format_ident!("{variant}");
        let wrapper_name = format_ident!("__{fn_name}_{label}");
        let flags: Option<TokenStream2> = (!flags.is_empty()).then(|| {
            quote! { #[target_feature(enable = #flags)] }
        });

        // Original generics, in order, with `A`'s slot replaced by the concrete
        // arch type for this variant. Used to call __impl_name.
        let call_generics: Vec<TokenStream2> = func
            .sig
            .generics
            .params
            .iter()
            .filter(|p| !matches!(p, GenericParam::Lifetime(_)))
            .map(|p| match p {
                GenericParam::Type(t) if t.ident == arch => {
                    quote! { #crate_path::simd::#variant_ident }
                }
                _ => bare_ident(p),
            })
            .collect();

        variant_fns.push(quote! {
            #flags
            #unsafety #asyncness fn #wrapper_name #wrapper_generics_decl (#inputs) #output #where_clause {
                #self_prefix #impl_name::<#(#lifetimes,)* #(#call_generics),*>(#(#call_args),*) #await_suffix
            }
        });

        match_arms.push(quote! {
            #crate_path::simd::Architecture::#variant_ident => {
                #self_prefix #wrapper_name #turbofish (#(#call_args),*) #await_suffix
            },
        });
    }

    let dispatch_call = quote! {
        unsafe {
            match #arch::ARCHITECTURE {
                #(#match_arms)*
            }
        }
    };

    let result: TokenStream = if is_impl {
        func.block =
            syn::parse2(quote! { { #dispatch_call } }).expect("expected valid dispatch body");

        quote! {
            #impl_fn
            #(#variant_fns)*
            #func
        }
        .into()
    } else {
        func.block = syn::parse2(quote! {
            {
                #impl_fn
                #(#variant_fns)*
                #dispatch_call
            }
        })
        .expect("expected valid dispatch body");

        quote! {
            #func
        }
        .into()
    };

    result
}

// =====================================================================================
// Function 2 -- `#[enable_targets(A)]` applied to an `impl` block.
//
// Mode A: `A` is one of the impl block's own generic params (e.g. `impl<A: Arch> Foo<A>`).
//         Every method in the block is transformed.
// Mode B: `A` is NOT one of the impl block's generics. Only methods that themselves
//         declare `A` as their own generic are transformed; everything else is left
//         untouched.
//
// Each transformed method's body becomes `match A::ARCHITECTURE { ... }` -- resolved
// against whichever `A` is already in lexical scope at that point (the impl block's, in
// mode A, or the method's own, in mode B). The method's original signature (including
// its own generics, if any) is left exactly as written; only its body changes.
//
// The generated `__impl` + per-variant wrapper functions are collected into a single
// appended, separate impl block with the same generics/Self type as the original, minus
// the trait if the original was a trait impl. `__impl` always freshly (re)declares
// `arch` as its own generic parameter -- shadowing the impl block's `A` where
// applicable -- since that's the only way a wrapper pinned to one concrete architecture
// can substitute a concrete type in place of it via turbofish.
// =====================================================================================
pub fn enable_targets_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let DispatchArgs { arch, .. } = parse_macro_input!(args as DispatchArgs);
    let mut item_impl = parse_macro_input!(item as ItemImpl);
    let crate_path = quick_noise_path();

    let arch_on_impl = item_impl
        .generics
        .params
        .iter()
        .any(|p| matches!(p, GenericParam::Type(t) if t.ident == arch));

    let mut appended_items: Vec<TokenStream2> = Vec::new();

    for entry in item_impl.items.iter_mut() {
        let ImplItem::Fn(method) = entry else {
            continue;
        };

        let method_has_arch = method
            .sig
            .generics
            .params
            .iter()
            .any(|p| matches!(p, GenericParam::Type(t) if t.ident == arch));

        // Mode B: skip anything that doesn't declare the generic itself.
        // Mode A: never skips (arch_on_impl is true, so this check is always false).
        if !arch_on_impl && !method_has_arch {
            continue;
        }

        if method.sig.constness.is_some() {
            continue;
        }

        let fn_name = method.sig.ident.clone();
        let unsafety = method.sig.safety.clone();
        let asyncness = method.sig.asyncness;
        let output = method.sig.output.clone();
        let inputs = method.sig.inputs.clone();
        let body = method.block.clone();
        let where_clause = method.sig.generics.where_clause.clone();

        // Method's own generics, split into lifetimes / others, with `arch` excluded
        // from "others" (whether it came from the method itself, mode B, or wasn't
        // present at all, mode A).
        let mut lifetimes = Vec::new();
        let mut other_full = Vec::new();
        let mut other_bare = Vec::new();
        for p in method.sig.generics.params.iter() {
            match p {
                GenericParam::Lifetime(l) => {
                    let lt = &l.lifetime;
                    lifetimes.push(quote! { #lt });
                }
                GenericParam::Type(t) if t.ident == arch => {}
                _ => {
                    other_full.push(quote! { #p });
                    other_bare.push(bare_ident(p));
                }
            }
        }

        let call_args: Vec<TokenStream2> = inputs
            .iter()
            .map(|a| match a {
                FnArg::Receiver(_) => quote! { self },
                FnArg::Typed(PatType { pat, .. }) => match &**pat {
                    syn::Pat::Ident(pi) => {
                        let i = &pi.ident;
                        quote! { #i }
                    }
                    _ => quote! { #pat },
                },
            })
            .collect();

        let impl_name = format_ident!("__{}_impl", fn_name);

        // __impl (re)declares `arch` fresh -- this shadows the impl block's own `A`
        // in mode A, and simply reintroduces the method's own `A` in mode B. Either
        // way it gives each variant wrapper an independent slot to turbofish into.
        appended_items.push(quote! {
            #[inline(always)]
            #unsafety #asyncness fn #impl_name<#(#lifetimes,)* #arch: #crate_path::simd::Arch #(, #other_full)*>(#inputs) #output
                #where_clause
            {
                #body
            }
        });

        let await_suffix = asyncness.is_some().then(|| quote! { .await });

        let wrapper_generics = if lifetimes.is_empty() && other_full.is_empty() {
            quote! {}
        } else {
            quote! { <#(#lifetimes,)* #(#other_full),*> }
        };
        let turbofish = if other_bare.is_empty() {
            quote! {}
        } else {
            quote! { ::<#(#other_bare),*> }
        };

        let mut match_arms = Vec::new();

        for (variant, label, flags) in FEATURES {
            let variant_ident = format_ident!("{variant}");
            let wrapper_name = format_ident!("__{fn_name}_{label}");
            let flags: Option<TokenStream2> = (!flags.is_empty())
                .then(|| quote! { #[target_feature(enable = #flags)] });

            appended_items.push(quote! {
                #flags
                #unsafety #asyncness fn #wrapper_name #wrapper_generics (#inputs) #output #where_clause {
                    Self::#impl_name::<#(#lifetimes,)* #crate_path::simd::#variant_ident #(, #other_bare)*>(#(#call_args),*) #await_suffix
                }
            });

            match_arms.push(quote! {
                #crate_path::simd::Architecture::#variant_ident => {
                    Self::#wrapper_name #turbofish (#(#call_args),*) #await_suffix
                },
            });
        }

        // Reroute based on whichever `A` is already in scope here -- the impl block's
        // (mode A) or the method's own (mode B) -- resolved at compile time, not via
        // runtime feature detection.
        method.block = syn::parse2(quote! {
            {
                unsafe {
                    match #arch::ARCHITECTURE {
                        #(#match_arms)*
                    }
                }
            }
        })
        .expect("expected valid dispatch body");
    }

    let self_ty = &item_impl.self_ty;
    let (impl_g, _, where_c) = item_impl.generics.split_for_impl();

    let appended_block = if appended_items.is_empty() {
        quote! {}
    } else {
        quote! {
            impl #impl_g #self_ty #where_c {
                #(#appended_items)*
            }
        }
    };

    quote! {
        #item_impl
        #appended_block
    }
    .into()
}
