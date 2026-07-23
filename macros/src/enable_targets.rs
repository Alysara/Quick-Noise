use std::hash::{DefaultHasher, Hash, Hasher};

use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit_mut::{self, VisitMut};
use syn::{
    FnArg, GenericParam, Ident, ImplItem, ItemFn, ItemImpl, PatType, ReturnType, Token,
    WhereClause, parse_macro_input,
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

fn mentions_ident(ty: &syn::Type, ident: &Ident) -> bool {
    let needle = ident.to_string();
    quote!(#ty)
        .to_string()
        .split_whitespace()
        .any(|t| t == needle)
}

fn output_mentions_ident(output: &ReturnType, ident: &Ident) -> bool {
    match output {
        ReturnType::Type(_, ty) => mentions_ident(ty, ident),
        ReturnType::Default => false,
    }
}

struct ReplaceArchType<'a> {
    from: &'a Ident,
    to: TokenStream2,
}

impl VisitMut for ReplaceArchType<'_> {
    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        if let syn::Type::Path(type_path) = ty
            && type_path.qself.is_none()
            && type_path.path.segments.len() == 1
        {
            let seg = &type_path.path.segments[0];
            if seg.ident == *self.from && seg.arguments.is_empty() {
                *ty = syn::parse2(self.to.clone()).expect("valid substituted type");
                return;
            }
        }
        visit_mut::visit_type_mut(self, ty);
    }
}

fn substitute_arch_in_signature(
    inputs: &Punctuated<FnArg, Token![,]>,
    output: &ReturnType,
    arch: &Ident,
    concrete: TokenStream2,
) -> (Punctuated<FnArg, Token![,]>, ReturnType) {
    let mut inputs = inputs.clone();
    let mut output = output.clone();
    let mut replacer = ReplaceArchType { from: arch, to: concrete };

    for arg in inputs.iter_mut() {
        replacer.visit_fn_arg_mut(arg);
    }
    replacer.visit_return_type_mut(&mut output);

    (inputs, output)
}

fn substitute_arch_in_generic_param(
    param: &GenericParam,
    arch: &Ident,
    concrete: TokenStream2,
) -> TokenStream2 {
    let mut p = param.clone();
    let mut replacer = ReplaceArchType { from: arch, to: concrete };
    replacer.visit_generic_param_mut(&mut p);
    quote! { #p }
}

fn substitute_arch_in_where_clause(
    where_clause: &Option<WhereClause>,
    arch: &Ident,
    concrete: TokenStream2,
) -> Option<TokenStream2> {
    where_clause.as_ref().map(|wc| {
        let mut wc = wc.clone();
        let mut replacer = ReplaceArchType { from: arch, to: concrete };
        replacer.visit_where_clause_mut(&mut wc);
        quote! { #wc }
    })
}

fn dispatch_call_args(inputs: &Punctuated<FnArg, Token![,]>, arch: &Ident) -> Vec<TokenStream2> {
    inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => quote! { self },
            FnArg::Typed(PatType { pat, ty, .. }) => {
                let ident_expr = match &**pat {
                    syn::Pat::Ident(pi) => {
                        let i = &pi.ident;
                        quote! { #i }
                    }
                    _ => quote! { #pat },
                };
                if mentions_ident(ty, arch) {
                    quote! { ::core::mem::transmute_copy(&#ident_expr) }
                } else {
                    ident_expr
                }
            }
        })
        .collect()
}

pub fn enable_targets_entry(args: TokenStream, item: TokenStream) -> TokenStream {
    if syn::parse::<ItemImpl>(item.clone()).is_ok() {
        enable_targets_impl(args, item)
    } else {
        enable_targets_fn(args, item)
    }
}

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

    let where_clause_owned = func.sig.generics.where_clause.clone();

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

    let other_params_raw: Vec<GenericParam> = func
        .sig
        .generics
        .params
        .iter()
        .filter(|p| !matches!(p, GenericParam::Lifetime(_)))
        .filter(|p| !matches!(p, GenericParam::Type(t) if t.ident == arch))
        .cloned()
        .collect();

    let ty_generics: Vec<TokenStream2> = other_params_raw.iter().map(bare_ident).collect();

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

    let dispatch_args = dispatch_call_args(&inputs, &arch);
    let output_needs_transmute = output_mentions_ident(&output, &arch);

    let impl_name = format_ident!("__{}_impl", fn_name);

    let impl_fn = quote! {
        #[inline(always)]
        #unsafety #asyncness fn #impl_name #impl_generics (#inputs) #output
            #where_clause
        {
            #body
        }
    };

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
        let concrete = quote! { #crate_path::simd::#variant_ident };

        let wrapper_ty_full: Vec<TokenStream2> = other_params_raw
            .iter()
            .map(|p| substitute_arch_in_generic_param(p, &arch, concrete.clone()))
            .collect();
        let wrapper_generics_decl = if lifetimes.is_empty() && wrapper_ty_full.is_empty() {
            quote! {}
        } else {
            quote! { <#(#lifetimes,)* #(#wrapper_ty_full),*> }
        };
        let wrapper_where = substitute_arch_in_where_clause(&where_clause_owned, &arch, concrete.clone());

        let call_generics: Vec<TokenStream2> = func
            .sig
            .generics
            .params
            .iter()
            .filter(|p| !matches!(p, GenericParam::Lifetime(_)))
            .map(|p| match p {
                GenericParam::Type(t) if t.ident == arch => concrete.clone(),
                _ => bare_ident(p),
            })
            .collect();

        let (wrapper_inputs, wrapper_output) =
            substitute_arch_in_signature(&inputs, &output, &arch, concrete.clone());

        variant_fns.push(quote! {
            #flags
            #unsafety #asyncness fn #wrapper_name #wrapper_generics_decl (#wrapper_inputs) #wrapper_output #wrapper_where {
                #self_prefix #impl_name::<#(#lifetimes,)* #(#call_generics),*>(#(#call_args),*) #await_suffix
            }
        });

        let call_expr = quote! { #self_prefix #wrapper_name #turbofish (#(#dispatch_args),*) #await_suffix };
        let call_expr = if output_needs_transmute {
            quote! { ::core::mem::transmute_copy(&(#call_expr)) }
        } else {
            call_expr
        };

        match_arms.push(quote! {
            #crate_path::simd::Architecture::#variant_ident => {
                #call_expr
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

pub fn enable_targets_impl(args: TokenStream, item: TokenStream) -> TokenStream {
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
        let where_clause_raw = method.sig.generics.where_clause.clone();

        let mut lifetimes = Vec::new();
        let mut other_params_raw: Vec<GenericParam> = Vec::new();
        let mut other_bare = Vec::new();
        for p in method.sig.generics.params.iter() {
            match p {
                GenericParam::Lifetime(l) => {
                    let lt = &l.lifetime;
                    lifetimes.push(quote! { #lt });
                }
                GenericParam::Type(t) if t.ident == arch => {}
                _ => {
                    other_bare.push(bare_ident(p));
                    other_params_raw.push(p.clone());
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

        let dispatch_args = dispatch_call_args(&inputs, &arch);
        let output_needs_transmute = output_mentions_ident(&output, &arch);

        let impl_name = format_ident!("__{fn_name}{trait_hash}_impl");

        let impl_other_full: Vec<TokenStream2> =
            other_params_raw.iter().map(|p| quote! { #p }).collect();
        appended_items.push(quote! {
            #[inline(always)]
            #unsafety #asyncness fn #impl_name<#(#lifetimes,)* #arch: #crate_path::simd::Arch #(, #impl_other_full)*>(#inputs) #output
                #where_clause_raw
            {
                #body
            }
        });

        let await_suffix = asyncness.is_some().then(|| quote! { .await });
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
            let concrete = quote! { #crate_path::simd::#variant_ident };

            let wrapper_other_full: Vec<TokenStream2> = other_params_raw
                .iter()
                .map(|p| substitute_arch_in_generic_param(p, &arch, concrete.clone()))
                .collect();
            let wrapper_generics = if lifetimes.is_empty() && wrapper_other_full.is_empty() {
                quote! {}
            } else {
                quote! { <#(#lifetimes,)* #(#wrapper_other_full),*> }
            };
            let wrapper_where =
                substitute_arch_in_where_clause(&where_clause_raw, &arch, concrete.clone());
            let (wrapper_inputs, wrapper_output) =
                substitute_arch_in_signature(&inputs, &output, &arch, concrete.clone());

            appended_items.push(quote! {
                #flags
                #unsafety #asyncness fn #wrapper_name #wrapper_generics (#wrapper_inputs) #wrapper_output #wrapper_where {
                    Self::#impl_name::<#(#lifetimes,)* #concrete #(, #other_bare)*>(#(#call_args),*) #await_suffix
                }
            });

            let call_expr = quote! { Self::#wrapper_name #turbofish (#(#dispatch_args),*) #await_suffix };
            let call_expr = if output_needs_transmute {
                quote! { ::core::mem::transmute_copy(&(#call_expr)) }
            } else {
                call_expr
            };

            match_arms.push(quote! {
                #crate_path::simd::Architecture::#variant_ident => {
                    #call_expr
                },
            });
        }

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
