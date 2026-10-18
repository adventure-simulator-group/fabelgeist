//! Import provenance for the epsilon rule, including unexpanded macro bodies.

use std::{
    collections::{BTreeMap, HashSet},
    path::{Component, Path, PathBuf},
};

use proc_macro2::{TokenStream, TokenTree};
use syn::ItemUse;

use super::ParsedFile;

mod collect;
mod includes;
mod scopes;
mod targets;
#[cfg(test)]
mod tests;

pub(super) type FloatGlobs = BTreeMap<String, HashSet<(usize, usize)>>;

const USE_IMPORT: &str = "use";
const SELF_IMPORT: &str = "self";

pub(super) fn float_globs(base: Option<&Path>, files: &[ParsedFile]) -> Result<FloatGlobs, String> {
    collect::float_globs(base, files)
}

pub(super) fn import_at(tokens: &[TokenTree]) -> Option<(ItemUse, usize)> {
    if !matches!(tokens.first(), Some(TokenTree::Ident(ident)) if ident == USE_IMPORT) {
        return None;
    }
    let end = tokens
        .iter()
        .position(|token| matches!(token, TokenTree::Punct(p) if p.as_char() == ';'))?;
    let import = syn::parse2(tokens[..=end].iter().cloned().collect()).ok()?;
    Some((import, end + 1))
}

pub(super) fn token_imports(stream: TokenStream) -> Vec<ItemUse> {
    let tokens = stream.into_iter().collect::<Vec<_>>();
    let mut imports = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if let Some((import, _)) = import_at(&tokens[index..]) {
            imports.push(import);
        } else if let TokenTree::Group(group) = token {
            imports.extend(token_imports(group.stream()));
        }
    }
    imports
}

pub(super) fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(component),
        }
    }
    result
}
