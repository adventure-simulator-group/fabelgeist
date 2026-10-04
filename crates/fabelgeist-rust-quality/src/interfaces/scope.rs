//! Source/module names used by the syntax census, before compiler expansion.
use syn::Ident;

use super::{InterfaceItem, InterfaceOwner, SourceFile};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Symbol(pub(super) Vec<String>);

impl Symbol {
    pub(super) fn appended(&self, suffix: &Symbol) -> Self {
        Self(self.0.iter().chain(&suffix.0).cloned().collect())
    }
}

#[derive(Clone)]
pub(super) struct SourceScope {
    pub(super) crate_name: String,
    modules: Vec<String>,
}

impl SourceScope {
    pub(super) fn from_source(source: &SourceFile) -> Self {
        let pieces: Vec<_> = source.path.split('/').collect();
        let crate_name = pieces.get(1).unwrap_or(&"unknown").replace('-', "_");
        let mut modules = Vec::new();
        if let Some(src) = pieces.iter().position(|part| *part == "src") {
            modules.extend(
                pieces[src + 1..pieces.len() - 1]
                    .iter()
                    .map(|part| (*part).into()),
            );
            let leaf = pieces.last().unwrap().trim_end_matches(".rs");
            if !matches!(leaf, "lib" | "main" | "mod") {
                modules.push(leaf.into());
            }
        }
        Self {
            crate_name,
            modules,
        }
    }

    pub(super) fn push(&mut self, name: &Ident) {
        self.modules.push(name.to_string());
    }

    pub(super) fn pop(&mut self) {
        self.modules.pop();
    }

    pub(super) fn item(&self, owner: Option<&InterfaceOwner>, name: &Ident) -> InterfaceItem {
        let mut parts = self.modules.clone();
        if let Some(owner) = owner {
            parts.push(owner.label.clone());
        }
        parts.push(name.to_string());
        InterfaceItem(parts.join("::"))
    }

    pub(super) fn module(&self) -> Symbol {
        Symbol(
            std::iter::once(self.crate_name.clone())
                .chain(self.modules.clone())
                .collect(),
        )
    }

    pub(super) fn local_symbol(&self, name: &Ident) -> Symbol {
        self.module().appended(&Symbol(vec![name.to_string()]))
    }

    pub(super) fn candidates(&self, path: &syn::Path) -> Vec<Symbol> {
        let mut parts: Vec<_> = path
            .segments
            .iter()
            .map(|part| part.ident.to_string())
            .collect();
        match parts.first().map(String::as_str) {
            Some("crate") => parts[0] = self.crate_name.clone(),
            Some("self") => {
                parts.remove(0);
                parts.splice(0..0, self.module().0);
            }
            Some("super") => {
                let mut parent = self.modules.clone();
                while parts.first().is_some_and(|part| part == "super") {
                    parts.remove(0);
                    parent.pop();
                }
                parts.splice(0..0, std::iter::once(self.crate_name.clone()).chain(parent));
            }
            _ if path.leading_colon.is_none() => {
                let external = Symbol(parts);
                return vec![self.module().appended(&external), external];
            }
            _ => {}
        }
        vec![Symbol(parts)]
    }
}
