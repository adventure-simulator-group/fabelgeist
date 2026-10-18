//! Machine epsilon is a precision parameter, not an absolute tolerance.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{ImplItemFn, Item, ItemImpl, ItemMod, Macro, visit::Visit};

use crate::scan::Finding;

mod imports;

pub const RULE: &str = "machine-epsilon";
const EPSILON_MEMBER: &str = "EPSILON";

pub struct ParsedFile {
    pub path: String,
    pub syntax: syn::File,
}

struct EpsilonReferences<'a> {
    path: &'a str,
    owners: Vec<String>,
    findings: Vec<Finding>,
    float_modules: &'a HashSet<String>,
    locations: HashSet<(usize, usize)>,
}

impl EpsilonReferences<'_> {
    fn record(&mut self, span: Span) {
        let location = span.start();
        // Macro imports are inspected as syntax and as tokens. Their source
        // identifier still represents one occurrence, even in a grouped use.
        if !self.locations.insert((location.line, location.column)) {
            return;
        }
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

    fn import(&mut self, tree: &syn::UseTree, module: &str) {
        // Report imports before they can hide precision behind a local name.
        match tree {
            syn::UseTree::Path(path) => self.import(&path.tree, &path.ident.to_string()),
            syn::UseTree::Name(name) if name.ident == EPSILON_MEMBER => {
                self.record(name.ident.span())
            }
            syn::UseTree::Rename(name) if name.ident == EPSILON_MEMBER => {
                self.record(name.ident.span())
            }
            syn::UseTree::Group(group) => group
                .items
                .iter()
                .for_each(|tree| self.import(tree, module)),
            syn::UseTree::Glob(glob) if self.float_modules.contains(module) => {
                // A float-module glob can conceal bare EPSILON in expressions
                // and macros. Reject the import itself, including its aliases.
                self.record(glob.star_token.span);
            }
            _ => {}
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
            Item::Macro(item) => item.ident.as_ref().map(ToString::to_string),
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
        self.import(&item.tree, "");
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        for import in imports::token_imports(item.tokens.clone()) {
            self.import(&import.tree, "");
        }
        self.tokens(item.tokens.clone());
    }
}

pub fn check_files(files: &[ParsedFile]) -> Vec<Finding> {
    let float_modules = imports::float_module_aliases(files.iter().map(|file| &file.syntax));
    files
        .iter()
        .flat_map(|file| check_with_modules(&file.path, &file.syntax, &float_modules))
        .collect()
}

#[cfg(test)]
pub fn check_file(path: &str, syntax: &syn::File) -> Vec<Finding> {
    let float_modules = imports::float_module_aliases([syntax]);
    check_with_modules(path, syntax, &float_modules)
}

fn check_with_modules(
    path: &str,
    syntax: &syn::File,
    float_modules: &HashSet<String>,
) -> Vec<Finding> {
    let mut checker = EpsilonReferences {
        path,
        owners: Vec::new(),
        findings: Vec::new(),
        float_modules,
        locations: HashSet::new(),
    };
    checker.visit_file(syntax);
    checker.findings
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
    fn checks_imports_in_macro_definitions_and_assertion_arguments() {
        let syntax = syn::parse_file(
            r#"
            macro_rules! glob {
                ($x:expr) => {{ use std::f32::*; $x <= EPSILON }};
            }
            macro_rules! grouped {
                ($x:expr) => {{ use core::f64::{EPSILON}; $x <= EPSILON }};
            }
            macro_rules! renamed {
                ($x:expr) => {{ use std::{f32::{EPSILON as error}}; $x <= error }};
            }
            macro_rules! qualified {
                ($x:expr) => {{ use std::f32::EPSILON as error; $x <= error }};
            }
            macro_rules! aliased_module {
                ($x:expr) => {{ use core::f32 as number; use number::*; $x <= EPSILON }};
            }
            fn assertion(x: f32) {
                assert!({ use std::f32::{EPSILON}; x > EPSILON });
            }
        "#,
        )
        .unwrap();
        let findings = check_file("crates/example/src/lib.rs", &syntax);
        assert_eq!(findings.len(), 6, "{findings:#?}");
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.item.as_str())
                .collect::<Vec<_>>(),
            [
                "glob",
                "grouped",
                "renamed",
                "qualified",
                "aliased_module",
                "assertion"
            ]
        );
    }

    #[test]
    fn macro_import_occurrences_count_source_identifiers_once() {
        let syntax = syn::parse_file(
            r#"
            macro_rules! direct {
                () => {{ use std::f32::EPSILON; EPSILON }};
            }
            macro_rules! grouped {
                () => {{ use std::f32::{EPSILON, EPSILON as second}; EPSILON + second }};
            }
            macro_rules! unrelated {
                () => {{ const EPSILON: f32 = 0.5; EPSILON }};
            }
        "#,
        )
        .unwrap();
        let findings = check_file("crates/example/src/lib.rs", &syntax);
        assert_eq!(findings.len(), 3, "{findings:#?}");
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.item == "direct")
                .count(),
            1
        );
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.item == "grouped")
                .count(),
            2
        );
    }

    #[test]
    fn ignores_comments_strings_and_adjacent_float_boundaries() {
        let syntax = syn::parse_file(
            r#"
            // f32::EPSILON
            const DOCUMENTATION: &str = "f64::EPSILON";
            macro_rules! documentation { () => { "use std::f32::*; EPSILON" }; }
            fn sample(value: f32) -> f32 { value.clamp(0.0_f32.next_up(), 1.0_f32.next_down()) }
        "#,
        )
        .unwrap();
        assert!(check_file("crates/example/src/lib.rs", &syntax).is_empty());
    }
}
