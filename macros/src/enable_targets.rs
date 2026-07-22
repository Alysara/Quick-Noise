use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, Ident, ItemFn, PatType, Token, parse_macro_input};

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

pub fn enable_targets_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let DispatchArgs { arch, mut is_impl } = parse_macro_input!(args as DispatchArgs);

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

    // Check if self is a parameter.
    is_impl |= func
        .sig
        .inputs
        .iter()
        .any(|a| matches!(a, FnArg::Receiver(_)));

    // Check if Self is the return type.
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

    // Lifetimes, unmodified, needed at the front of every generic list we rebuild.
    let lifetimes: Vec<TokenStream2> = func
        .sig
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Lifetime(l) => {
                let lt = &l.lifetime;
                Some(quote! { #lt })
            }
            _ => None,
        })
        .collect();

    // Full type/const params (bounds included), excluding lifetimes and `arch` itself.
    // Used to declare each per-variant wrapper fn's own generics (arch is erased there,
    // since each wrapper is pinned to one concrete architecture).
    let ty_full: Vec<TokenStream2> = func
        .sig
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
        .filter(|p| !matches!(p, syn::GenericParam::Type(t) if t.ident == arch))
        .map(|p| quote! { #p })
        .collect();

    // Bare idents only (no bounds), excluding lifetimes and `arch`.
    // Used at call sites (turbofish position) where bounds aren't legal syntax.
    let ty_generics: Vec<TokenStream2> = func
        .sig
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
        .filter(|p| !matches!(p, syn::GenericParam::Type(t) if t.ident == arch))
        .map(|p| match p {
            syn::GenericParam::Type(t) => {
                let i = &t.ident;
                quote! { #i }
            }
            syn::GenericParam::Const(c) => {
                let i = &c.ident;
                quote! { #i }
            }
            _ => unreachable!("filtered out above"),
        })
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

    let features: &[(&str, &str, &str)] = if cfg!(target_arch = "x86_64") {
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

    // Shared generic body. Reuses the user's own generics (including `A: Arch`)
    // as written -- the macro doesn't inject anything here.
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

    let mut variant_fns = Vec::new();
    let mut match_arms = Vec::new();

    let await_suffix = asyncness.is_some().then(|| {
        quote! { .await }
    });

    for (variant, label, flags) in features {
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
            .filter(|p| !matches!(p, syn::GenericParam::Lifetime(_)))
            .map(|p| match p {
                syn::GenericParam::Type(t) if t.ident == arch => {
                    quote! { quick_noise::simd::#variant_ident }
                }
                syn::GenericParam::Type(t) => {
                    let i = &t.ident;
                    quote! { #i }
                }
                syn::GenericParam::Const(c) => {
                    let i = &c.ident;
                    quote! { #i }
                }
                syn::GenericParam::Lifetime(_) => unreachable!("filtered out above"),
            })
            .collect();

        // Turbofish for calling the wrapper fn (arch already erased from its generics).
        let turbofish = if ty_generics.is_empty() {
            quote! {}
        } else {
            quote! { ::<#(#lifetimes,)* #(#ty_generics),*> }
        };

        let self_prefix = is_impl.then(|| quote!(Self::));

        variant_fns.push(quote! {
            #flags
            #unsafety #asyncness fn #wrapper_name #wrapper_generics_decl (#inputs) #output #where_clause {
                #self_prefix #impl_name::<#(#lifetimes,)* #(#call_generics),*>(#(#call_args),*) #await_suffix
            }
        });

        match_arms.push(quote! {
            quick_noise::simd::Architecture::#variant_ident => {
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
