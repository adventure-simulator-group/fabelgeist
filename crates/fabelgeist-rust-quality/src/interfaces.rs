//! Raw interfaces are migration debt, independently of literal/fixture scopes.
//!
//! This census includes test helpers and trait methods. Unexpanded macros are
//! reported separately so a clean scalar census cannot conceal generated APIs.
mod aliases;
mod error;
mod fingerprint;
mod scope;
#[cfg(test)]
mod tests;

use std::{fs, path::Path};

use quote::ToTokens;
use syn::{
    FnArg, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemTrait, ReturnType, Signature, TraitItemFn,
    Type, spanned::Spanned, visit::Visit,
};
use walkdir::WalkDir;

use crate::{
    config::Config,
    scan::{Finding, path_matches},
};
use aliases::{AliasIndex, MacroOrigin, RawTypes};
use error::CensusError;
use fingerprint::OpaqueInterfaceFingerprint;
use scope::SourceScope;

pub(super) struct SourceFile {
    path: String,
    syntax: syn::File,
}

pub(super) struct InterfaceItem(String);
pub(super) struct InterfaceOwner {
    label: String,
    ty: Option<Type>,
    generics: syn::Generics,
}

struct InterfaceFunction {
    item: InterfaceItem,
    generics: syn::Generics,
}

enum InterfaceSlot<'a> {
    Input(&'a syn::Pat),
    Return,
    Receiver,
}

impl SourceFile {
    fn load(root: &Path, path: &Path) -> Result<Self, CensusError> {
        let relative = path
            .strip_prefix(root)
            .map_err(|source| CensusError::OutsideRoot {
                path: path.into(),
                source,
            })?;
        let source = fs::read_to_string(path).map_err(|source| CensusError::Read {
            path: path.into(),
            source,
        })?;
        let syntax = syn::parse_file(&source).map_err(|source| CensusError::Parse {
            path: relative.into(),
            source,
        })?;
        Ok(Self {
            path: relative.to_string_lossy().replace('\\', "/"),
            syntax,
        })
    }
}

pub fn census(root: &Path, config: &Config) -> Result<Vec<Finding>, CensusError> {
    let mut sources = Vec::new();
    let entries = WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_entry(|entry| entry.file_name() != "target" && entry.file_name() != "node_modules");
    for entry in entries {
        let entry = entry.map_err(CensusError::Walk)?;
        if !entry.file_type().is_file() || entry.path().extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let source = SourceFile::load(root, entry.path())?;
        if !config
            .excluded_paths
            .iter()
            .any(|pattern| path_matches(pattern, &source.path))
        {
            sources.push(source);
        }
    }
    let aliases = AliasIndex::collect(&sources);
    let mut findings = Vec::new();
    for source in &sources {
        findings.extend(scan_source(source, &aliases));
    }
    findings.sort();
    Ok(findings)
}

fn scan_source(source: &SourceFile, aliases: &AliasIndex) -> Vec<Finding> {
    let mut scanner = InterfaceScanner {
        source,
        aliases,
        scope: SourceScope::from_source(source),
        owner: None,
        function: None,
        findings: Vec::new(),
    };
    scanner.visit_file(&source.syntax);
    scanner.findings
}

struct InterfaceScanner<'a> {
    source: &'a SourceFile,
    aliases: &'a AliasIndex,
    scope: SourceScope,
    owner: Option<InterfaceOwner>,
    function: Option<InterfaceFunction>,
    findings: Vec<Finding>,
}

impl InterfaceScanner<'_> {
    fn signature(&mut self, signature: &Signature) -> Option<InterfaceFunction> {
        let item = self.scope.item(self.owner.as_ref(), &signature.ident);
        let mut generics = self
            .owner
            .as_ref()
            .map_or_else(syn::Generics::default, |owner| owner.generics.clone());
        generics.params.extend(signature.generics.params.clone());
        if let Some(clause) = &signature.generics.where_clause {
            if let Some(owner_clause) = &mut generics.where_clause {
                owner_clause.predicates.extend(clause.predicates.clone());
            } else {
                generics.where_clause = Some(clause.clone());
            }
        }
        let owner = self.owner.as_ref().and_then(|owner| owner.ty.clone());
        for input in &signature.inputs {
            if let FnArg::Typed(input) = input {
                let raw = self
                    .aliases
                    .raw_types(&input.ty, &self.scope, &generics, owner.as_ref());
                self.record(&item, InterfaceSlot::Input(&input.pat), &input.ty, raw);
            } else if let FnArg::Receiver(receiver) = input {
                let raw =
                    self.aliases
                        .raw_types(&receiver.ty, &self.scope, &generics, owner.as_ref());
                self.record(&item, InterfaceSlot::Receiver, &receiver.ty, raw);
            }
        }
        if let ReturnType::Type(_, ty) = &signature.output {
            let raw = self
                .aliases
                .raw_types(ty, &self.scope, &generics, owner.as_ref());
            self.record(&item, InterfaceSlot::Return, ty, raw);
        }
        // Fn(u64), Iterator<Item = u64>, and associated type bounds are part
        // of the interface even when the input parameter is spelled `F`.
        let raw = self.aliases.generic_raw_types(&generics, &self.scope);
        for value in raw.values {
            self.findings.push(Finding {
                rule: value.as_rule_name().into(),
                path: self.source.path.clone(),
                item: item.0.clone(),
                fingerprint: format!("generic-bound:{}", value.spelling),
                line: signature.span().start().line,
            });
        }
        self.function.replace(InterfaceFunction { item, generics })
    }

    fn record(&mut self, item: &InterfaceItem, slot: InterfaceSlot<'_>, ty: &Type, raw: RawTypes) {
        let parameter = match slot {
            InterfaceSlot::Input(pattern) => pattern.to_token_stream().to_string(),
            InterfaceSlot::Return => "return".into(),
            InterfaceSlot::Receiver => "self".into(),
        };
        let spelling = ty.to_token_stream().to_string().replace(' ', "");
        for value in raw.values {
            self.findings.push(Finding {
                rule: value.as_rule_name().into(),
                path: self.source.path.clone(),
                item: item.0.clone(),
                fingerprint: format!("{parameter}:{spelling}:{}", value.spelling),
                line: ty.span().start().line,
            });
        }
    }
}

impl<'ast> Visit<'ast> for InterfaceScanner<'_> {
    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        if let Some((_, items)) = &node.content {
            self.scope.push(&node.ident);
            for item in items {
                self.visit_item(item);
            }
            self.scope.pop();
        }
    }

    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let previous = self.owner.replace(InterfaceOwner {
            label: node.self_ty.to_token_stream().to_string().replace(' ', ""),
            ty: Some((*node.self_ty).clone()),
            generics: node.generics.clone(),
        });
        syn::visit::visit_item_impl(self, node);
        self.owner = previous;
    }

    fn visit_item_trait(&mut self, node: &'ast ItemTrait) {
        let previous = self.owner.replace(InterfaceOwner {
            label: node.ident.to_string(),
            ty: None,
            generics: node.generics.clone(),
        });
        syn::visit::visit_item_trait(self, node);
        self.owner = previous;
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        let previous = self.signature(&node.sig);
        syn::visit::visit_block(self, &node.block);
        self.function = previous;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        let previous = self.signature(&node.sig);
        syn::visit::visit_block(self, &node.block);
        self.function = previous;
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        let previous = self.signature(&node.sig);
        if let Some(body) = &node.default {
            self.visit_block(body);
        }
        self.function = previous;
    }

    fn visit_expr_closure(&mut self, node: &'ast syn::ExprClosure) {
        let generics = self
            .function
            .as_ref()
            .map_or_else(syn::Generics::default, |function| function.generics.clone());
        let item = InterfaceItem(format!(
            "{}::closure",
            self.function
                .as_ref()
                .map_or("module", |function| function.item.0.as_str())
        ));
        for input in &node.inputs {
            if let syn::Pat::Type(input) = input {
                let raw = self.aliases.raw_types(
                    &input.ty,
                    &self.scope,
                    &generics,
                    self.owner.as_ref().and_then(|owner| owner.ty.as_ref()),
                );
                self.record(&item, InterfaceSlot::Input(&input.pat), &input.ty, raw);
            } else {
                self.findings.push(Finding {
                    rule: "unresolved-interface".into(),
                    path: self.source.path.clone(),
                    item: item.0.clone(),
                    fingerprint: format!(
                        "inferred-closure:{}:{}",
                        input.to_token_stream(),
                        OpaqueInterfaceFingerprint::from_tokens(&node.body)
                    ),
                    line: input.span().start().line,
                });
            }
        }
        if let ReturnType::Type(_, ty) = &node.output {
            let raw = self.aliases.raw_types(
                ty,
                &self.scope,
                &generics,
                self.owner.as_ref().and_then(|owner| owner.ty.as_ref()),
            );
            self.record(&item, InterfaceSlot::Return, ty, raw);
        }
        if matches!(node.output, ReturnType::Default) {
            self.findings.push(Finding {
                rule: "unresolved-interface".into(),
                path: self.source.path.clone(),
                item: item.0.clone(),
                fingerprint: format!(
                    "inferred-closure-return:{}",
                    OpaqueInterfaceFingerprint::from_tokens(&node.body)
                ),
                line: node.span().start().line,
            });
        }
        syn::visit::visit_expr_closure(self, node);
    }

    fn visit_foreign_item_fn(&mut self, node: &'ast syn::ForeignItemFn) {
        let previous = self.signature(&node.sig);
        self.function = previous;
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        let name = node
            .ident
            .as_ref()
            .unwrap_or(&node.mac.path.segments.last().unwrap().ident);
        self.findings.push(Finding {
            rule: "unexpanded-interface-macro".into(),
            path: self.source.path.clone(),
            item: self.scope.item(self.owner.as_ref(), name).0,
            fingerprint: format!(
                "{}:{}",
                node.mac.path.to_token_stream().to_string().replace(' ', ""),
                OpaqueInterfaceFingerprint::from_tokens(&node.mac.tokens)
            ),
            line: node.span().start().line,
        });
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let tokens = node.tokens.to_token_stream().to_string();
        if tokens.split_whitespace().any(|token| token == "fn")
            || matches!(
                self.aliases.macro_origin(&node.path, &self.scope),
                MacroOrigin::Handwritten
            )
        {
            self.findings.push(Finding {
                rule: "unexpanded-interface-macro".into(),
                path: self.source.path.clone(),
                item: self
                    .scope
                    .item(
                        self.owner.as_ref(),
                        &node.path.segments.last().unwrap().ident,
                    )
                    .0,
                fingerprint: format!(
                    "{}:{}",
                    node.path.to_token_stream().to_string().replace(' ', ""),
                    OpaqueInterfaceFingerprint::from_tokens(&node.tokens)
                ),
                line: node.span().start().line,
            });
        }
    }
}
