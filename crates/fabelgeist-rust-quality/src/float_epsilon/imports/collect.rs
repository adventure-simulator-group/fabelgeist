//! Follow module trees and included fragments, then isolate unreached sources.
//!
//! File lookup follows the Rust Reference's module-source rules:
//! https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute

use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    path::{Path, PathBuf},
};

use proc_macro2::{Delimiter, Span, TokenStream, TokenTree};
use syn::{Item, ItemMod, ItemUse, Macro, UseTree, visit::Visit};

use super::super::ParsedFile;
use super::{
    FloatGlobs, SELF_IMPORT, import_at, includes, normalize,
    scopes::{Graph, ImportPath, ScopeId, ScopeKind},
    targets,
};

const MODULE: &str = "mod";
const MOD_SOURCE: &str = "mod.rs";
const PATH_ATTRIBUTE: &str = "path";

#[derive(Clone)]
struct Context {
    file: usize,
    scope: ScopeId,
    module_dir: PathBuf,
    attribute_dir: PathBuf,
}

struct Glob {
    file: usize,
    scope: ScopeId,
    path: ImportPath,
    span: Span,
}

struct Collector<'a> {
    files: &'a [ParsedFile],
    paths: BTreeMap<PathBuf, usize>,
    graph: Graph,
    context: Context,
    active: BTreeSet<usize>,
    reached: BTreeSet<usize>,
    globs: Vec<Glob>,
}

#[derive(Clone, Copy)]
enum FileKind {
    CrateRoot,
    Module,
    ExplicitPath,
    Include,
}

impl Collector<'_> {
    fn file(&mut self, file: usize, scope: ScopeId, kind: FileKind) {
        if !self.active.insert(file) {
            return;
        }
        self.reached.insert(file);
        let path = Path::new(&self.files[file].path);
        let parent = path.parent().unwrap_or(Path::new(""));
        let context = Context {
            file,
            scope,
            module_dir: if matches!(
                kind,
                FileKind::CrateRoot | FileKind::ExplicitPath | FileKind::Include
            ) || path.file_name().is_some_and(|name| name == MOD_SOURCE)
            {
                parent.into()
            } else {
                path.with_extension("")
            },
            attribute_dir: parent.into(),
        };
        let previous = std::mem::replace(&mut self.context, context);
        let files = self.files;
        self.visit_file(&files[file].syntax);
        self.context = previous;
        self.active.remove(&file);
    }

    fn include(&mut self, path: &syn::LitStr) {
        let source = Path::new(&self.files[self.context.file].path);
        let directory = source.parent().unwrap_or(Path::new(""));
        let path = normalize(&directory.join(path.value()));
        // Only follow audited first-party source. Generated and excluded files
        // are absent from this inventory, even when an include points at them.
        if let Some(file) = self.paths.get(&path).copied() {
            self.file(file, self.context.scope, FileKind::Include);
        }
    }

    fn import(&mut self, item: &ItemUse) {
        self.use_tree(
            &item.tree,
            ImportPath {
                segments: Vec::new(),
                absolute: item.leading_colon.is_some(),
            },
            &item.vis,
        );
    }

    fn use_tree(&mut self, tree: &UseTree, mut path: ImportPath, visibility: &syn::Visibility) {
        match tree {
            UseTree::Path(part) => {
                path.segments.push(part.ident.to_string());
                self.use_tree(&part.tree, path, visibility);
            }
            UseTree::Group(group) => {
                for tree in &group.items {
                    self.use_tree(tree, path.clone(), visibility);
                }
            }
            UseTree::Name(name) => {
                let local = if name.ident == SELF_IMPORT {
                    path.segments.last().cloned()
                } else {
                    path.segments.push(name.ident.to_string());
                    Some(name.ident.to_string())
                };
                if let Some(local) = local {
                    self.graph
                        .import(self.context.scope, local, path, visibility);
                }
            }
            UseTree::Rename(rename) => {
                if rename.ident != SELF_IMPORT {
                    path.segments.push(rename.ident.to_string());
                }
                self.graph.import(
                    self.context.scope,
                    rename.rename.to_string(),
                    path,
                    visibility,
                );
            }
            UseTree::Glob(glob) => {
                self.graph
                    .glob(self.context.scope, path.clone(), visibility);
                self.globs.push(Glob {
                    file: self.context.file,
                    scope: self.context.scope,
                    path,
                    span: glob.star_token.span,
                });
            }
        }
    }

    fn tokens(&mut self, stream: TokenStream) {
        let tokens = stream.into_iter().collect::<Vec<_>>();
        let mut index = 0;
        while index < tokens.len() {
            if let Some((path, length)) = includes::at(&tokens, index) {
                self.include(&path);
                index += length;
                continue;
            }
            if let Some((import, length)) = import_at(&tokens[index..]) {
                self.import(&import);
                index += length;
                continue;
            }
            if let [
                TokenTree::Ident(keyword),
                TokenTree::Ident(name),
                TokenTree::Group(body),
                ..,
            ] = &tokens[index..]
                && keyword == MODULE
                && body.delimiter() == Delimiter::Brace
            {
                let previous = self.context.scope;
                self.context.scope =
                    self.graph
                        .module(previous, name.to_string(), &syn::Visibility::Inherited);
                self.tokens(body.stream());
                self.context.scope = previous;
                index += 3;
                continue;
            }
            if let TokenTree::Group(group) = &tokens[index] {
                let previous = self.context.scope;
                self.context.scope = self.graph.scope(Some(previous), ScopeKind::Block);
                self.tokens(group.stream());
                self.context.scope = previous;
            }
            index += 1;
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item(&mut self, item: &'ast Item) {
        let name = match item {
            Item::Struct(item) => Some((&item.ident, &item.vis)),
            Item::Enum(item) => Some((&item.ident, &item.vis)),
            Item::Type(item) => Some((&item.ident, &item.vis)),
            Item::Trait(item) => Some((&item.ident, &item.vis)),
            Item::Union(item) => Some((&item.ident, &item.vis)),
            _ => None,
        };
        if let Some((name, visibility)) = name {
            self.graph
                .other(self.context.scope, name.to_string(), visibility);
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        self.import(item);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        let name = item
            .rename
            .as_ref()
            .map_or_else(|| item.ident.to_string(), |(_, alias)| alias.to_string());
        if item.ident == SELF_IMPORT {
            self.graph.self_alias(self.context.scope, name, &item.vis);
            return;
        }
        self.graph.import(
            self.context.scope,
            name,
            ImportPath {
                segments: vec![item.ident.to_string()],
                absolute: true,
            },
            &item.vis,
        );
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        let previous = self.context.scope;
        self.context.scope = self.graph.scope(Some(previous), ScopeKind::Block);
        syn::visit::visit_block(self, block);
        self.context.scope = previous;
    }

    fn visit_macro(&mut self, item: &'ast Macro) {
        if let Some(path) = includes::literal_path(item) {
            self.include(&path);
            return;
        }
        let previous = self.context.scope;
        self.context.scope = self.graph.scope(Some(previous), ScopeKind::Block);
        self.tokens(item.tokens.clone());
        self.context.scope = previous;
    }

    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        let name = item.ident.to_string();
        let scope = self
            .graph
            .module(self.context.scope, name.clone(), &item.vis);
        let attribute = item.attrs.iter().find_map(|attribute| {
            if !attribute.path().is_ident(PATH_ATTRIBUTE) {
                return None;
            }
            let syn::Meta::NameValue(value) = &attribute.meta else {
                return None;
            };
            let syn::Expr::Lit(expression) = &value.value else {
                return None;
            };
            let syn::Lit::Str(path) = &expression.lit else {
                return None;
            };
            Some(self.context.attribute_dir.join(path.value()))
        });
        if let Some((_, items)) = &item.content {
            let directory = attribute.unwrap_or_else(|| self.context.module_dir.join(name));
            let file = self.context.file;
            let previous = std::mem::replace(
                &mut self.context,
                Context {
                    file,
                    scope,
                    module_dir: directory.clone(),
                    attribute_dir: directory,
                },
            );
            for item in items {
                self.visit_item(item);
            }
            self.context = previous;
        } else {
            let kind = if attribute.is_some() {
                FileKind::ExplicitPath
            } else {
                FileKind::Module
            };
            let candidates = attribute.map_or_else(
                || {
                    vec![
                        self.context.module_dir.join(format!("{name}.rs")),
                        self.context.module_dir.join(name).join(MOD_SOURCE),
                    ]
                },
                |path| vec![path],
            );
            for path in candidates {
                if let Some(file) = self.paths.get(&normalize(&path)).copied() {
                    self.file(file, scope, kind);
                    break;
                }
            }
        }
    }
}

pub(super) fn float_globs(base: Option<&Path>, files: &[ParsedFile]) -> Result<FloatGlobs, String> {
    let mut graph = Graph::default();
    let placeholder = graph.scope(None, ScopeKind::Module);
    let mut collector = Collector {
        files,
        paths: files
            .iter()
            .enumerate()
            .map(|(i, file)| (normalize(Path::new(&file.path)), i))
            .collect(),
        graph,
        context: Context {
            file: 0,
            scope: placeholder,
            module_dir: PathBuf::new(),
            attribute_dir: PathBuf::new(),
        },
        active: BTreeSet::new(),
        reached: BTreeSet::new(),
        globs: Vec::new(),
    };
    // Register every library before resolving any path, including imports from
    // a package's binary or a separate integration-test crate into its library.
    for root in targets::roots(base, files)? {
        let scope = collector.graph.scope(None, ScopeKind::Module);
        if root.library.is_some() {
            collector.graph.libraries.insert(root.package, scope);
        }
        collector.graph.externs.insert(scope, root.externs);
        collector.file(root.file, scope, FileKind::CrateRoot);
    }
    // Even source outside a declared target remains audited. Give it a fresh
    // crate identity instead of borrowing unrelated bindings from other files.
    for file in 0..files.len() {
        if !collector.reached.contains(&file) {
            let scope = collector.graph.scope(None, ScopeKind::Module);
            collector.file(file, scope, FileKind::CrateRoot);
        }
    }
    let mut result = BTreeMap::<String, HashSet<_>>::new();
    for glob in collector.globs {
        if collector.graph.is_float(glob.scope, &glob.path) {
            let position = glob.span.start();
            result
                .entry(files[glob.file].path.clone())
                .or_default()
                .insert((position.line, position.column));
        }
    }
    Ok(result)
}
