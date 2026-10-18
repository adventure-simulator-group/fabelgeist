//! Machine epsilon is a precision parameter, not an absolute tolerance.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{ImplItemFn, Item, ItemImpl, ItemMod, Macro, visit::Visit};

use crate::scan::Finding;

pub const RULE: &str = "machine-epsilon";
const EPSILON_MEMBER: &str = "EPSILON";
const SELF_IMPORT: &str = "self";

struct EpsilonReferences<'a> {
    path: &'a str,
    owners: Vec<String>,
    findings: Vec<Finding>,
    float_modules: HashSet<String>,
}

impl EpsilonReferences<'_> {
    fn record(&mut self, span: Span) {
        self.findings.push(Finding {
            rule: RULE.into(),
            path: self.path.into(),
            item: if self.owners.is_empty() {
                "<module>".into()
            } else {
                self.owners.join("::")
            },
            fingerprint: EPSILON_MEMBER.into(),
            line: span.start().line,
        });
    }

    fn tokens(&mut self, tokens: TokenStream) {
        let mut colon = false;
        for token in tokens {
            match &token {
                TokenTree::Ident(ident) if colon && ident == EPSILON_MEMBER => {
                    self.record(ident.span())
                }
                TokenTree::Group(group) => self.tokens(group.stream()),
                _ => {}
            }
            colon = matches!(&token, TokenTree::Punct(punct) if punct.as_char() == ':');
        }
    }
}

impl<'ast> Visit<'ast> for EpsilonReferences<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        let name = match item {
            Item::Fn(item) => Some(item.sig.ident.to_string()),
            Item::Const(item) => Some(item.ident.to_string()),
            Item::Static(item) => Some(item.ident.to_string()),
            Item::Trait(item) => Some(item.ident.to_string()),
            Item::Type(item) => Some(item.ident.to_string()),
            _ => None,
        };
        if let Some(name) = name {
            self.owners.push(name);
            syn::visit::visit_item(self, item);
            self.owners.pop();
        } else {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        self.owners.push(item.ident.to_string());
        syn::visit::visit_item_mod(self, item);
        self.owners.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        self.owners
            .push(item.self_ty.to_token_stream().to_string().replace(' ', ""));
        syn::visit::visit_item_impl(self, item);
        self.owners.pop();
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.owners.push(item.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, item);
        self.owners.pop();
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        self.owners.push(item.ident.to_string());
        syn::visit::visit_impl_item_const(self, item);
        self.owners.pop();
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        self.owners.push(item.ident.to_string());
        syn::visit::visit_trait_item_const(self, item);
        self.owners.pop();
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        self.owners.push(item.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, item);
        self.owners.pop();
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if expression.qself.is_some()
            && expression.path.segments.len() == 1
            && let Some(segment) = expression.path.segments.first()
            && segment.ident == EPSILON_MEMBER
        {
            self.record(segment.ident.span());
        }
        syn::visit::visit_expr_path(self, expression);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        // Checking the terminal identifier also catches primitive type aliases
        // and aliases of std::f32/std::f64 without needing compiler resolution.
        if path.segments.len() > 1
            && let Some(segment) = path.segments.last()
            && segment.ident == EPSILON_MEMBER
        {
            self.record(segment.ident.span());
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        // An aliased import is reported at its source, before it can hide the
        // precision parameter behind a local tolerance name.
        fn imports(checker: &mut EpsilonReferences<'_>, tree: &syn::UseTree, module: &str) {
            match tree {
                syn::UseTree::Path(path) => imports(checker, &path.tree, &path.ident.to_string()),
                syn::UseTree::Name(name) if name.ident == EPSILON_MEMBER => {
                    checker.record(name.ident.span())
                }
                syn::UseTree::Rename(name) if name.ident == EPSILON_MEMBER => {
                    checker.record(name.ident.span())
                }
                syn::UseTree::Group(group) => group
                    .items
                    .iter()
                    .for_each(|tree| imports(checker, tree, module)),
                syn::UseTree::Glob(glob) if checker.float_modules.contains(module) => {
                    // A float-module glob brings EPSILON into scope, where
                    // ordinary paths and macro tokens can hide its origin.
                    // Reject the import itself, including aliases of it.
                    checker.record(glob.star_token.span);
                }
                _ => {}
            }
        }
        imports(self, &item.tree, "");
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        self.tokens(item.tokens.clone());
    }
}

pub fn check_file(path: &str, syntax: &syn::File) -> Vec<Finding> {
    let mut checker = EpsilonReferences {
        path,
        owners: Vec::new(),
        findings: Vec::new(),
        float_modules: float_module_aliases(syntax),
    };
    checker.visit_file(syntax);
    checker.findings
}

fn float_module_aliases(syntax: &syn::File) -> HashSet<String> {
    #[derive(Default)]
    struct Imports(Vec<(String, String)>);

    impl<'ast> Visit<'ast> for Imports {
        fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
            fn aliases(imports: &mut Imports, tree: &syn::UseTree, module: &str) {
                match tree {
                    syn::UseTree::Path(path) => {
                        aliases(imports, &path.tree, &path.ident.to_string())
                    }
                    syn::UseTree::Group(group) => group
                        .items
                        .iter()
                        .for_each(|tree| aliases(imports, tree, module)),
                    syn::UseTree::Rename(rename) => {
                        let source = if rename.ident == SELF_IMPORT {
                            module.to_string()
                        } else {
                            rename.ident.to_string()
                        };
                        imports.0.push((source, rename.rename.to_string()));
                    }
                    _ => {}
                }
            }
            aliases(self, &item.tree, "");
        }
    }

    let mut imports = Imports::default();
    imports.visit_file(syntax);
    let mut modules = HashSet::from(["f32".into(), "f64".into()]);
    // Rust imports are independent of declaration order. Resolve chained
    // aliases to a fixed point before checking any glob import.
    loop {
        let previous = modules.len();
        for (source, alias) in &imports.0 {
            if modules.contains(source) {
                modules.insert(alias.clone());
            }
        }
        if modules.len() == previous {
            return modules;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_tests_macros_qualified_paths_and_aliases() {
        let syntax = syn::parse_file(
            r#"
            use std::f32::EPSILON as tolerance;
            use core::f64 as number;
            type Float = f32;
            const ROUNDING: f64 = core::f64::EPSILON;
            #[cfg(test)] mod tests {
                #[test] fn boundary() {
                    assert!(value <= f32::EPSILON);
                    let denominator = value.max(number::EPSILON);
                    let alias = Float::EPSILON;
                    let qualified = <f32>::EPSILON;
                }
            }
        "#,
        )
        .unwrap();
        let findings = check_file("crates/example/tests/numeric.rs", &syntax);
        assert_eq!(findings.len(), 6);
        assert_eq!(findings[0].item, "<module>");
        assert_eq!(findings[1].item, "ROUNDING");
        assert_eq!(findings[2].item, "tests::boundary");
        assert!(findings.iter().all(|finding| finding.line > 0));
    }

    #[test]
    fn precision_constants_have_exact_associated_owners() {
        let syntax = syn::parse_file(
            r#"
            struct Frame;
            impl Frame { const ROUNDING: f32 = f32::EPSILON; }
            trait Projection { const ROUNDING: f64 = f64::EPSILON; }
        "#,
        )
        .unwrap();
        let findings = check_file("crates/example/src/lib.rs", &syntax);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].item, "Frame::ROUNDING");
        assert_eq!(findings[1].item, "Projection::ROUNDING");
    }

    #[test]
    fn rejects_float_globs_before_bare_expressions_and_assertions_can_hide_epsilon() {
        let syntax = syn::parse_file(
            r#"
            use std::f32::*;
            fn invalid(distance: f32) -> bool { distance <= EPSILON }
            mod tests {
                use core::f64::*;
                fn assertion(distance: f64) { assert!(distance > EPSILON); }
            }
        "#,
        )
        .unwrap();
        let findings = check_file("crates/example/tests/numeric.rs", &syntax);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].item, "<module>");
        assert_eq!(findings[1].item, "tests");
    }

    #[test]
    fn rejects_globs_through_grouped_and_chained_float_module_aliases() {
        let syntax = syn::parse_file(
            r#"
            use chained::*;
            use number as chained;
            use std::{f32 as number};
            use core::f64::{self as double};
            fn assertion() {
                use double::*;
                assert!(distance <= EPSILON);
            }
            mod unrelated {
                use super::*;
                const EPSILON: f32 = 0.5;
                fn local(distance: f32) -> bool { distance <= EPSILON }
            }
        "#,
        )
        .unwrap();
        let findings = check_file("crates/example/src/lib.rs", &syntax);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[1].item, "assertion");
    }

    #[test]
    fn ignores_comments_strings_and_adjacent_float_boundaries() {
        let syntax = syn::parse_file(
            r#"
            // f32::EPSILON
            const DOCUMENTATION: &str = "f64::EPSILON";
            fn sample(value: f32) -> f32 { value.clamp(0.0_f32.next_up(), 1.0_f32.next_down()) }
        "#,
        )
        .unwrap();
        assert!(check_file("crates/example/src/lib.rs", &syntax).is_empty());
    }
}
