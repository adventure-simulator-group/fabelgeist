//! Recognize literal source fragments without expanding arbitrary macros.
//!
//! `include!` shares its caller's scope and resolves paths from the source file:
//! https://doc.rust-lang.org/std/macro.include.html

use proc_macro2::TokenTree;
use syn::{LitStr, Macro, Token, parse::Parser, punctuated::Punctuated};

const INCLUDE: &str = "include";

pub(super) fn literal_path(invocation: &Macro) -> Option<LitStr> {
    let segments = invocation
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let recognized = match segments.as_slice() {
        [name] => invocation.path.leading_colon.is_none() && name == INCLUDE,
        [root, name] => matches!(root.as_str(), "std" | "core") && name == INCLUDE,
        _ => false,
    };
    if !recognized {
        return None;
    }
    let paths = Punctuated::<LitStr, Token![,]>::parse_terminated
        .parse2(invocation.tokens.clone())
        .ok()?;
    (paths.len() == 1).then(|| paths[0].clone())
}

pub(super) fn at(tokens: &[TokenTree], start: usize) -> Option<(LitStr, usize)> {
    if start > 0
        && matches!(&tokens[start - 1], TokenTree::Punct(punctuation) if punctuation.as_char() == ':')
    {
        return None;
    }
    for (index, token) in tokens[start..].iter().enumerate() {
        match token {
            TokenTree::Ident(_) => {}
            TokenTree::Punct(punctuation) if matches!(punctuation.as_char(), ':' | '!') => {}
            TokenTree::Group(_) => {
                let invocation =
                    syn::parse2(tokens[start..=start + index].iter().cloned().collect()).ok()?;
                return literal_path(&invocation).map(|path| (path, index + 1));
            }
            _ => return None,
        }
    }
    None
}
