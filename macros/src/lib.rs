use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree};

// TODO: Simplify this logic.

#[proc_macro_attribute]
pub fn dispatch_simd(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut outer = TokenStream::new();
    let mut inner_func = TokenStream::new();
    let mut inner_caller = TokenStream::new();

    // Parse arch identifier.
    let mut arg_iter = args.into_iter();
    let arch_ident_token = arg_iter
        .next()
        .expect("Dispatch simd macro requires an Arch identifier! Example: #[dispatch_simd(A)]");

    let arch_ident = match arch_ident_token {
        TokenTree::Ident(i) => i,
        _ => panic!(
            "Dispatch simd macro requires a valid Arch identifier! Example: #[dispatch_simd(A)]"
        ),
    };
    let arch_token = TokenTree::Ident(arch_ident);

    if arg_iter.next().is_some() {
        panic!("Dispatch simd macro only takes one Arch identifier! Example: #[dispatch_simd(A)]");
    }

    // Parse function modifiers.
    let mut item_iter = item.into_iter();

    let mut is_unsafe = false;
    let mut is_async = false;
    let mut cur_item = item_iter.next();

    loop {
        match cur_item {
            Some(tt) => {
                if let TokenTree::Ident(i) = &tt {
                    match i.to_string().as_str() {
                        "unsafe" => {
                            is_unsafe = true;
                            inner_func.extend(std::iter::once(TokenTree::Ident(i.clone())));
                        }
                        "async" => {
                            is_async = true;
                            inner_func.extend(std::iter::once(TokenTree::Ident(i.clone())));
                        }
                        "const" => {
                            panic!("Dynamic dispatch does not work in a const context! Use StaticArch and StaticSimd for static dispatch.")
                        }
                        "fn" => {
                            outer.extend(std::iter::once(TokenTree::Ident(i.clone())));
                            inner_func.extend(std::iter::once(TokenTree::Ident(i.clone())));
                            break;
                        }
                        _ => {}
                    }
                }
                outer.extend(std::iter::once(tt));
                cur_item = item_iter.next();
            }
            None => panic!("Dispatch simd macro expected a valid function declaration!"),
        }
    }

    // Parse function name.
    let fn_ident = match item_iter.next() {
        Some(TokenTree::Ident(i)) => i,
        _ => panic!("Dispatch simd macro expected a named function declaration!"),
    };
    let dispatch_fn_ident = Ident::new(&format!("{}_{}", fn_ident, "dispatched"), fn_ident.span());
    let dispatch_fn_token = TokenTree::Ident(dispatch_fn_ident);

    outer.extend(std::iter::once(TokenTree::Ident(fn_ident)));
    inner_func.extend(std::iter::once(dispatch_fn_token.clone()));
    inner_caller.extend(std::iter::once(dispatch_fn_token.clone()));

    // Generics parsing.
    let comma = TokenTree::Punct(Punct::new(',', Spacing::Alone));
    let opening_bracket = TokenTree::Punct(Punct::new('<', Spacing::Alone));
    let arch_bound: TokenStream = ": quick_noise::simd::Arch".parse().unwrap();
    inner_func.extend(std::iter::once(opening_bracket.clone()));

    let mut reached_ty = false;
    let mut inner_func_tys = TokenStream::new();
    let mut inner_func_lifetimes = TokenStream::new();
    let mut inner_caller_tys = TokenStream::new();
    let mut inner_caller_lifetimes = TokenStream::new();

    inner_func_tys.extend(std::iter::once(arch_token.clone()));
    inner_func_tys.extend(arch_bound);
    inner_func_tys.extend(std::iter::once(comma.clone()));

    cur_item = item_iter.next();
    let mut next_token = cur_item
        .clone()
        .expect("Dispatch simd macro expected a function body!");

    let last_bracket = TokenTree::Punct(Punct::new('>', Spacing::Alone));
    let mut prev = next_token.clone();
    if matches!(next_token, TokenTree::Punct(ref p) if p.as_char() == '<') {
        outer.extend(std::iter::once(next_token.clone()));


        let mut bracket_layer = 0;

        'outer: loop {
            cur_item = item_iter.next();
            let mut token = cur_item
                .clone()
                .expect("Dispatch simd macro expected valid generics!");
            outer.extend(std::iter::once(token.clone()));
            // inner_func.extend(std::iter::once(token.clone()));

            match token {
                TokenTree::Ident(ref i) if i.to_string() == "const" => {
                    inner_func_tys.extend(std::iter::once(token.clone()));
                    cur_item = item_iter.next();
                    token = cur_item.expect("Dispatch simd macro expected valid generics!");
                    outer.extend(std::iter::once(token.clone()));
                    inner_func_tys.extend(std::iter::once(token.clone()));
                    inner_caller_tys.extend(std::iter::once(token.clone()));
                    reached_ty = true;
                }
                TokenTree::Punct(ref p) if p.as_char() == '\'' => {
                    inner_func_lifetimes.extend(std::iter::once(token.clone()));
                    cur_item = item_iter.next();
                    token = cur_item.expect("Dispatch simd macro expected valid generics!");
                    outer.extend(std::iter::once(token.clone()));
                    inner_func_lifetimes.extend(std::iter::once(token.clone()));
                    inner_func_lifetimes.extend(std::iter::once(comma.clone()));
                }
                TokenTree::Punct(ref p) if p.as_char() == '>' => break,
                TokenTree::Ident(_) => {
                    inner_func_tys.extend(std::iter::once(token.clone()));
                    inner_caller_tys.extend(std::iter::once(token.clone()));
                    reached_ty = true;
                }
                _ => panic!("Dispatch macro expected valid generics!"),
            }

            loop {
                cur_item = item_iter.next();
                token = cur_item
                    .clone()
                    .expect("Dispatch simd macro expected valid generics!");

                // inner_func.extend(std::iter::once(token.clone()));
                outer.extend(std::iter::once(token.clone()));
                let prev_is_dash = matches!(prev, TokenTree::Punct(ref p) if p.as_char() == '-');
                match token {
                    TokenTree::Punct(ref p) if p.as_char() == ',' && bracket_layer == 0 => {
                        if reached_ty {
                            inner_caller_tys.extend(std::iter::once(comma.clone()));
                            inner_func_tys.extend(std::iter::once(comma.clone()));
                        }
                        continue 'outer;
                    }
                    TokenTree::Punct(ref p) if p.as_char() == '<' => bracket_layer += 1,
                    TokenTree::Punct(ref p) if p.as_char() == '>' && !prev_is_dash => {
                        match bracket_layer == 0 {
                            true => break 'outer,
                            false => bracket_layer -= 1,
                        }
                    }
                    _ => {}
                }
                inner_func_tys.extend(std::iter::once(token.clone()));
                prev = token;
            }
        }
        next_token = item_iter
            .next()
            .expect("Dispatch simd macro expected parameters!");

        if reached_ty {
            let caller_gen_start: TokenStream = "::<".parse().unwrap();
            inner_caller.extend(caller_gen_start);
            inner_caller.extend(inner_caller_tys);
            inner_caller.extend(std::iter::once(last_bracket.clone()));
        }
    }

    inner_func.extend(inner_func_lifetimes);
    inner_func.extend(inner_func_tys);
    inner_func.extend(std::iter::once(last_bracket.clone()));

    // Parse parameters.
    outer.extend(std::iter::once(next_token.clone()));
    inner_func.extend(std::iter::once(next_token.clone()));
    fill_caller_params(&mut inner_caller, next_token.clone());

    let mut layer = 0;

    let mut prev = cur_item.clone();
    for tt in item_iter {
        // Skip nested <> to exclude <{CONST}> patterns.
        match tt {
            TokenTree::Group(ref g) if layer == 0 && g.delimiter() == Delimiter::Brace => {
                inner_func.extend(std::iter::once(tt.clone()));
                break;
            }
            TokenTree::Punct(ref p) if p.as_char() == '<' => layer += 1,
            TokenTree::Punct(ref p) if p.as_char() == '>' => {
                // Exception for ->.
                match prev {
                    Some(TokenTree::Punct(ref p)) if p.as_char() == '-' => {}
                    _ => layer -= 1,
                }
            }
            _ => {}
        }

        // Write tokens as is to inner and outer.
        inner_func.extend(std::iter::once(tt.clone()));
        outer.extend(std::iter::once(tt.clone()));

        prev = Some(tt);
    }

    let dispatch_path = match is_async {
        true => "quick_noise::simd::dispatch::dispatch_async!",
        false => "quick_noise::simd::dispatch::dispatch!",
    };

    let mut dispatch: TokenStream = dispatch_path.parse().unwrap();
    let caller_group = TokenTree::Group(Group::new(Delimiter::Parenthesis, inner_caller));
    dispatch.extend(std::iter::once(caller_group));
    // dispatch.extend(std::iter::once(semicolon));
    // if is_async {
    //     let await_modifier: TokenStream = ".await".parse().unwrap();
    //     dispatch.extend(await_modifier);
    // }

    if is_unsafe {
        let unsafe_group = TokenTree::Group(Group::new(Delimiter::Brace, dispatch));
        dispatch = "unsafe".parse().unwrap();
        dispatch.extend(std::iter::once(unsafe_group));
    }

    inner_func.extend(dispatch);

    let new_body = Group::new(Delimiter::Brace, inner_func);
    outer.extend(std::iter::once(new_body));

    // panic!("Test: {}", outer);
    outer
}

fn fill_caller_params(caller: &mut TokenStream, param_token: TokenTree) {
    let mut param_stream = TokenStream::new();

    let stream = match param_token {
        TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => g.stream(),
        _ => panic!("Dispatch simd macro expected parameters!"),
    };

    let mut params_iter = stream.into_iter();
    let mut cur_param = handle_self_and_next(&mut param_stream, &mut params_iter);

    let comma = TokenTree::Punct(Punct::new(',', Spacing::Alone));
    'outer: loop {
        let Some(mut tt) = cur_param.clone() else {
            break;
        };

        if matches!(tt, TokenTree::Ident(ref i) if i.to_string() == "mut") {
            let cur_param = params_iter.next();
            let Some(new_tt) = cur_param.clone() else {
                break;
            };
            tt = new_tt;
        }

        if !matches!(tt, TokenTree::Ident(_)) {
            panic!(
                "Dispatch simd macro expected valid parameter syntax! {:?}",
                tt
            );
        }

        param_stream.extend(std::iter::once(tt));
        let mut bracket_layer = 0;

        loop {
            cur_param = params_iter.next();
            let Some(token) = cur_param.clone() else {
                break 'outer;
            };

            match token {
                TokenTree::Punct(ref p) if p.as_char() == ',' && bracket_layer == 0 => {
                    param_stream.extend(std::iter::once(comma.clone()));
                    cur_param = params_iter.next();
                    continue 'outer;
                }
                TokenTree::Punct(ref p) if p.as_char() == '<' => bracket_layer += 1,
                TokenTree::Punct(ref p) if p.as_char() == '>' => bracket_layer -= 1,
                _ => {}
            }
        }
    }

    let params = TokenTree::Group(Group::new(Delimiter::Parenthesis, param_stream));
    caller.extend(std::iter::once(params));
}

fn handle_self_and_next(
    param_stream: &mut TokenStream,
    iter: &mut impl Iterator<Item = TokenTree>,
) -> Option<TokenTree> {
    let mut cur = iter.next();

    if let Some(tt) = cur.clone()
        && let TokenTree::Punct(ref p) = tt
        && p.as_char() == '&'
    {
        param_stream.extend(std::iter::once(tt));
    } else {
        return cur;
    }

    cur = iter.next();
    if let Some(tt) = cur.clone()
        && let TokenTree::Ident(ref i) = tt
        && i.to_string() == "mut"
    {
        param_stream.extend(std::iter::once(tt));
        cur = iter.next();
    }

    if let Some(tt) = cur
        && let TokenTree::Ident(ref i) = tt
        && i.to_string() == "self"
    {
        param_stream.extend(std::iter::once(tt));
    } else {
        panic!("Dispatch simd macro expected valid parameter syntax! 2");
    }

    let next = iter.next();
    if next.is_some() {
        let comma = TokenTree::Punct(Punct::new(',', Spacing::Alone));
        param_stream.extend(std::iter::once(comma));
    }

    next
}
