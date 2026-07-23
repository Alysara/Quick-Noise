use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{
    FnArg, GenericParam, Ident, ImplItem, ItemFn, ItemImpl, PatType, Token, parse_macro_input,
};

use crate::{FEATURES, quick_noise_path};

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
                    "expected `associated`; dispatch_simd takes an Arch identifier \
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

pub fn dispatch_simd_entry(args: TokenStream, item: TokenStream) -> TokenStream {
    if syn::parse::<ItemImpl>(item.clone()).is_ok() {
        dispatch_simd_impl(args, item)
    } else {
        dispatch_simd_fn(args, item)
    }
}

pub fn dispatch_simd_fn(args: TokenStream, item: TokenStream) -> TokenStream {
    let DispatchArgs { arch, mut is_impl } = parse_macro_input!(args as DispatchArgs);
    let mut func = parse_macro_input!(item as ItemFn);
    let crate_path = quick_noise_path();

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
    is_impl |= matches!(&func.sig.output, syn::ReturnType::Type(_, ty)
        if quote!(#ty).to_string().split_whitespace().any(|t| t == "Self"));

    let fn_name = func.sig.ident.clone();
    let unsafety = func.sig.safety.clone();
    let asyncness = func.sig.asyncness;
    let output = func.sig.output.clone();
    let inputs = func.sig.inputs.clone();
    let body = func.block.clone();
    let (impl_generics, _, where_clause) = func.sig.generics.split_for_impl();

    let (lifetimes, non_lifetimes): (Vec<_>, Vec<_>) = func
        .sig
        .generics
        .params
        .iter()
        .partition(|p| matches!(p, GenericParam::Lifetime(_)));

    let ty_full: Vec<TokenStream2> = non_lifetimes.iter().map(|p| quote! { #p }).collect();
    let ty_generics: Vec<TokenStream2> = non_lifetimes.iter().map(|p| bare_ident(p)).collect();

    let call_args: Vec<TokenStream2> = inputs
        .iter()
        .map(|arg| match arg {
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

    let impl_fn = quote! {
        #[inline(always)]
        #unsafety #asyncness fn #impl_name<#(#lifetimes,)* #arch: #crate_path::simd::Arch #(, #ty_full)*>(#inputs) #output
            #where_clause
        {
            #body
        }
    };

    let self_prefix = is_impl.then(|| quote! { Self:: });
    let await_suffix = asyncness.is_some().then(|| quote! { .await });
    let turbofish = (!ty_generics.is_empty()).then(|| quote! { ::<#(#ty_generics),*> });

    let mut variant_fns = Vec::new();
    let mut match_arms = Vec::new();

    for (variant, label, flags) in FEATURES {
        let variant_ident = format_ident!("{variant}");
        let wrapper_name = format_ident!("__{fn_name}_{label}");
        let flags: Option<TokenStream2> = (!flags.is_empty())
            .then(|| quote! { #[target_feature(enable = #flags)] });

        variant_fns.push(quote! {
            #flags
            #unsafety #asyncness fn #wrapper_name #impl_generics (#inputs) #output #where_clause {
                #self_prefix #impl_name::<#crate_path::simd::#variant_ident #(, #ty_generics)*>(#(#call_args),*) #await_suffix
            }
        });

        match_arms.push(quote! {
            #crate_path::simd::Architecture::#variant_ident => {
                std::hint::cold_path();
                #self_prefix #wrapper_name #turbofish (#(#call_args),*) #await_suffix
            },
        });
    }

    let dispatch_call = quote! {
        unsafe {
            match *#crate_path::simd::DETECTED_ARCH {
                #(#match_arms)*
            }
        }
    };

    let result: TokenStream = if is_impl {
        func.block =
            syn::parse2(quote! { { #dispatch_call } }).expect("expected valid dispatch body");
        quote! { #impl_fn #(#variant_fns)* #func }.into()
    } else {
        func.block = syn::parse2(quote! { { #impl_fn #(#variant_fns)* #dispatch_call } })
            .expect("expected valid dispatch body");
        quote! { #func }.into()
    };

    result
}

pub fn dispatch_simd_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    let DispatchArgs { arch, .. } = parse_macro_input!(args as DispatchArgs);
    let mut item_impl = parse_macro_input!(item as ItemImpl);
    let crate_path = quick_noise_path();

    let trait_hash = item_impl.trait_.clone().map_or(String::new(), |t| {
        let mut hasher = DefaultHasher::new();
        t.0.to_token_stream().to_string().hash(&mut hasher);
        format!("_{:012x}", hasher.finish())[..7].to_string()
    });

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

        let (lifetimes, other_full, other_bare): (Vec<TokenStream2>, Vec<TokenStream2>, Vec<TokenStream2>) = {
            let mut lifetimes = Vec::new();
            let mut full = Vec::new();
            let mut bare = Vec::new();
            for p in method.sig.generics.params.iter() {
                match p {
                    GenericParam::Lifetime(l) => {
                        let lt = &l.lifetime;
                        lifetimes.push(quote! { #lt });
                    }
                    GenericParam::Type(t) if t.ident == arch => {}
                    _ => {
                        full.push(quote! { #p });
                        bare.push(bare_ident(p));
                    }
                }
            }
            (lifetimes, full, bare)
        };

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

        let impl_name = format_ident!("__{fn_name}{trait_hash}_impl");

        appended_items.push(quote! {
            #[inline(always)]
            #unsafety #asyncness fn #impl_name<#(#lifetimes,)* #arch: quick_noise::simd::Arch #(, #other_full)*>(#inputs) #output
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
            let wrapper_name = format_ident!("__{fn_name}{trait_hash}_{label}");
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
                    std::hint::cold_path();
                    Self::#wrapper_name #turbofish (#(#call_args),*) #await_suffix
                },
            });
        }

        method.block = syn::parse2(quote! {
            {
                unsafe {
                    match *#crate_path::simd::DETECTED_ARCH {
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
