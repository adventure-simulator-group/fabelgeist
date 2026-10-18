//! Import provenance for the epsilon rule, including unexpanded macro bodies.

use std::collections::HashSet;

use proc_macro2::{TokenStream, TokenTree};
use syn::{ItemUse, Macro, UseTree, visit::Visit};

const USE_IMPORT: &str = "use";
const SELF_IMPORT: &str = "self";

struct Alias {
    source: String,
    local: String,
}

#[derive(Default)]
struct Imports {
    aliases: Vec<Alias>,
}

impl Imports {
    fn tree(&mut self, tree: &UseTree, module: &str) {
        match tree {
            UseTree::Path(path) => self.tree(&path.tree, &path.ident.to_string()),
            UseTree::Group(group) => group.items.iter().for_each(|tree| self.tree(tree, module)),
            UseTree::Rename(rename) => self.aliases.push(Alias {
                source: if rename.ident == SELF_IMPORT {
                    module.to_string()
                } else {
                    rename.ident.to_string()
                },
                local: rename.rename.to_string(),
            }),
            _ => {}
        }
    }
}

impl<'ast> Visit<'ast> for Imports {
    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.tree(&item.tree, "");
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        for import in token_imports(item.tokens.clone()) {
            self.tree(&import.tree, "");
        }
    }
}

pub(super) fn float_module_aliases<'a>(
    files: impl IntoIterator<Item = &'a syn::File>,
) -> HashSet<String> {
    let mut imports = Imports::default();
    for syntax in files {
        imports.visit_file(syntax);
    }
    let mut modules = HashSet::from(["f32".into(), "f64".into()]);
    // Conservatively track terminal alias names, just as qualified EPSILON
    // detection does. Include every scanned first-party file: a re-export or
    // macro definition must not hide provenance from its consuming file.
    // Import declaration and filesystem iteration order do not affect Rust
    // name binding, so resolve all alias chains to a fixed point.
    loop {
        let previous = modules.len();
        for alias in &imports.aliases {
            if modules.contains(&alias.source) {
                modules.insert(alias.local.clone());
            }
        }
        if modules.len() == previous {
            return modules;
        }
    }
}

pub(super) fn token_imports(stream: TokenStream) -> Vec<ItemUse> {
    let tokens = stream.into_iter().collect::<Vec<_>>();
    let mut imports = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token {
            TokenTree::Ident(ident) if ident == USE_IMPORT => {
                // Groups are opaque here, so the first semicolon terminates
                // this import even when its use tree contains nested groups.
                if let Some(end) = tokens[index..]
                    .iter()
                    .position(|token| matches!(token, TokenTree::Punct(p) if p.as_char() == ';'))
                    && let Ok(import) =
                        syn::parse2(tokens[index..=index + end].iter().cloned().collect())
                {
                    imports.push(import);
                }
            }
            TokenTree::Group(group) => imports.extend(token_imports(group.stream())),
            _ => {}
        }
    }
    imports
}
