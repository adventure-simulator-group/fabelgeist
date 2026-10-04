//! Resolve handwritten aliases before classifying scalar interface leaves.
use std::collections::{BTreeMap, BTreeSet};

use syn::{
    Generics, Ident, ItemEnum, ItemMod, ItemStruct, ItemType, ItemUse, Type, UseTree, visit::Visit,
};

mod associated;
mod probe;

pub(super) use probe::RawTypes;
use probe::{InterfaceLeaf, TypeProbe};

use super::SourceFile;

use super::scope::{SourceScope, Symbol};

#[derive(Clone)]
struct Alias {
    scope: SourceScope,
    generics: Generics,
    ty: Type,
}

struct ImportTarget {
    scope: SourceScope,
    path: syn::Path,
}

#[derive(Default)]
pub(super) struct AliasIndex {
    associated: associated::AssociatedTypes,
    aliases: BTreeMap<Symbol, Vec<Alias>>,
    nominal: BTreeSet<Symbol>,
    modules: BTreeSet<Symbol>,
    imports: BTreeMap<Symbol, ImportTarget>,
    globs: BTreeMap<Symbol, Vec<ImportTarget>>,
    macro_names: BTreeSet<String>,
}

pub(super) enum MacroOrigin {
    Handwritten,
    Unknown,
}

impl AliasIndex {
    pub(super) fn macro_origin(&self, path: &syn::Path, scope: &SourceScope) -> MacroOrigin {
        let mut pending = scope.candidates(path);
        let mut visited = BTreeSet::new();
        let mut imported_prefixes = BTreeSet::new();
        while let Some(symbol) = pending.pop() {
            if !visited.insert(symbol.clone()) {
                continue;
            }
            if symbol
                .0
                .last()
                .is_some_and(|name| self.macro_names.contains(name))
            {
                return MacroOrigin::Handwritten;
            }
            for split in (1..=symbol.0.len()).rev() {
                let prefix = Symbol(symbol.0[..split].to_vec());
                let suffix = Symbol(symbol.0[split..].to_vec());
                if let Some(import) = self.imports.get(&prefix) {
                    if !imported_prefixes.insert(prefix.clone()) {
                        continue;
                    }
                    pending.extend(
                        import
                            .scope
                            .candidates(&import.path)
                            .into_iter()
                            .map(|target| target.appended(&suffix)),
                    );
                }
                if suffix.0.len() == 1
                    && let Some(globs) = self.globs.get(&prefix)
                {
                    for import in globs {
                        pending.extend(
                            import
                                .scope
                                .candidates(&import.path)
                                .into_iter()
                                .filter(|target| self.modules.contains(target))
                                .map(|target| target.appended(&suffix)),
                        );
                    }
                }
            }
        }
        MacroOrigin::Unknown
    }

    pub(super) fn collect(sources: &[SourceFile]) -> Self {
        let mut index = Self::default();
        for source in sources {
            index
                .modules
                .insert(SourceScope::from_source(source).module());
            Collector {
                index: &mut index,
                scope: SourceScope::from_source(source),
            }
            .visit_file(&source.syntax);
        }
        index.associated = associated::AssociatedTypes::collect(sources, &index);
        index
    }

    pub(super) fn raw_types(
        &self,
        ty: &Type,
        scope: &SourceScope,
        generics: &Generics,
        owner: Option<&Type>,
    ) -> RawTypes {
        let mut probe = TypeProbe::for_signature(self, scope, generics);
        if let Some(owner) = owner {
            probe.bind_self(self.raw_types(owner, scope, generics, None));
            probe.bind_associated(owner);
        }
        probe.visit_type(ty);
        probe.result
    }

    pub(super) fn generic_raw_types(&self, generics: &Generics, scope: &SourceScope) -> RawTypes {
        let mut probe = TypeProbe::for_signature(self, scope, generics);
        probe.visit_generics(generics);
        probe.result
    }

    fn resolve(&self, path: &syn::Path, scope: &SourceScope) -> Resolution<'_> {
        for candidate in scope.candidates(path) {
            let result = self.resolve_symbol(&candidate, &mut ResolutionTrace::default());
            if !matches!(result, Resolution::Missing) {
                return result;
            }
        }
        // Filesystem modules cannot identify include! expansion sites. Keep
        // any matching alias visible as unresolved debt rather than guessing.
        if path.segments.len() == 1 {
            let name = path.segments.last().unwrap().ident.to_string();
            if let Some(key) = self
                .aliases
                .keys()
                .find(|key| key.0.first() == Some(&scope.crate_name) && key.0.last() == Some(&name))
            {
                return Resolution::Ambiguous(key.clone());
            }
        }
        Resolution::Missing
    }

    fn resolve_symbol(&self, symbol: &Symbol, visiting: &mut ResolutionTrace) -> Resolution<'_> {
        if matches!(symbol.0.as_slice(), [root, module, name]
            if matches!(root.as_str(), "std" | "core")
                && matches!(module.as_str(), "convert")
                && matches!(name.as_str(), "Infallible"))
        {
            return Resolution::Nominal(symbol.clone());
        }
        if self.nominal.contains(symbol) {
            return Resolution::Nominal(symbol.clone());
        }
        if let Some(aliases) = self.aliases.get(symbol) {
            return match aliases.as_slice() {
                [alias] => Resolution::Alias(symbol.clone(), alias),
                _ => Resolution::Ambiguous(symbol.clone()),
            };
        }
        if matches!(
            symbol.0.first().map(String::as_str),
            Some("std" | "core" | "alloc")
        ) && matches!(
            symbol.0.last().unwrap().as_str(),
            "Vec"
                | "Option"
                | "Result"
                | "Box"
                | "Rc"
                | "Arc"
                | "Weak"
                | "Cell"
                | "RefCell"
                | "UnsafeCell"
                | "BTreeMap"
                | "BTreeSet"
                | "HashMap"
                | "HashSet"
                | "VecDeque"
                | "Cow"
                | "PhantomData"
                | "Mutex"
                | "RwLock"
                | "Pin"
        ) {
            return Resolution::Nominal(symbol.clone());
        }
        if visiting.imports.contains(symbol) && self.imports.contains_key(symbol) {
            return Resolution::Ambiguous(symbol.clone());
        }
        if !visiting.symbols.insert(symbol.clone()) {
            // A repeated glob branch has already been searched. Keep this
            // set for the entire lookup so diamond imports remain bounded.
            return Resolution::Missing;
        }
        let result = self.resolve_imports(symbol, visiting);
        if matches!(result, Resolution::Missing)
            && let Some(leaf) = InterfaceLeaf::from_symbol(symbol)
        {
            return Resolution::Raw(leaf);
        }
        result
    }

    fn resolve_imports(&self, symbol: &Symbol, visiting: &mut ResolutionTrace) -> Resolution<'_> {
        for split in (1..=symbol.0.len()).rev() {
            let prefix = Symbol(symbol.0[..split].to_vec());
            let suffix = Symbol(symbol.0[split..].to_vec());
            // A function re-export may share a module's name in the value
            // namespace. It cannot replace that module in a qualified type
            // path, nor turn every local type lookup into an import cycle.
            if let Some(import) = self.imports.get(&prefix)
                && (!self.modules.contains(&prefix) || suffix.0.is_empty())
            {
                if !visiting.imports.insert(prefix.clone()) {
                    return Resolution::Ambiguous(prefix);
                }
                let result = self.resolve_target(import, &suffix, visiting);
                visiting.imports.remove(&prefix);
                return result;
            }
            if let Some(globs) = self.globs.get(&prefix) {
                if !visiting.imports.insert(prefix.clone()) {
                    continue;
                }
                let mut resolved = BTreeMap::new();
                for import in globs {
                    let next = self.resolve_glob(import, &suffix, visiting);
                    if !matches!(next, Resolution::Missing) {
                        resolved.insert(next.identity().unwrap(), next);
                    }
                }
                visiting.imports.remove(&prefix);
                match resolved.len() {
                    0 => {}
                    1 => return resolved.into_values().next().unwrap(),
                    _ => return Resolution::Ambiguous(symbol.clone()),
                }
            }
        }
        Resolution::Missing
    }

    fn resolve_glob(
        &self,
        import: &ImportTarget,
        suffix: &Symbol,
        visiting: &mut ResolutionTrace,
    ) -> Resolution<'_> {
        // A glob imports members of an actual module. Trying nonexistent
        // relative module candidates recursively grows bogus paths and can
        // traverse the same import graph exponentially.
        for target in import.scope.candidates(&import.path) {
            if self.modules.contains(&target) {
                let result = self.resolve_symbol(&target.appended(suffix), visiting);
                if !matches!(result, Resolution::Missing) {
                    return result;
                }
            }
        }
        Resolution::Missing
    }

    fn resolve_target(
        &self,
        import: &ImportTarget,
        suffix: &Symbol,
        visiting: &mut ResolutionTrace,
    ) -> Resolution<'_> {
        for target in import.scope.candidates(&import.path) {
            let result = self.resolve_symbol(&target.appended(suffix), visiting);
            if !matches!(result, Resolution::Missing) {
                return result;
            }
        }
        Resolution::Missing
    }
}

#[derive(Default)]
struct ResolutionTrace {
    symbols: BTreeSet<Symbol>,
    imports: BTreeSet<Symbol>,
}

enum Resolution<'a> {
    Raw(InterfaceLeaf),
    Alias(Symbol, &'a Alias),
    Ambiguous(Symbol),
    Nominal(Symbol),
    Missing,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum ResolutionIdentity {
    Alias(Symbol),
    Nominal(Symbol),
    Unresolved(Symbol),
    Raw(InterfaceLeaf),
}

impl Resolution<'_> {
    fn identity(&self) -> Option<ResolutionIdentity> {
        Some(match self {
            Self::Alias(symbol, _) => ResolutionIdentity::Alias(symbol.clone()),
            Self::Nominal(symbol) => ResolutionIdentity::Nominal(symbol.clone()),
            Self::Ambiguous(symbol) => ResolutionIdentity::Unresolved(symbol.clone()),
            Self::Raw(leaf) => ResolutionIdentity::Raw(leaf.clone()),
            Self::Missing => return None,
        })
    }
}

struct Collector<'a> {
    index: &'a mut AliasIndex,
    scope: SourceScope,
}

impl Collector<'_> {
    fn imports(&mut self, tree: &UseTree, prefix: &mut Vec<Ident>) {
        match tree {
            UseTree::Path(path) => {
                prefix.push(path.ident.clone());
                self.imports(&path.tree, prefix);
                prefix.pop();
            }
            UseTree::Group(group) => {
                for tree in &group.items {
                    self.imports(tree, prefix);
                }
            }
            UseTree::Name(name) => {
                let mut parts = prefix.clone();
                let local = if name.ident == "self" {
                    prefix.last().unwrap()
                } else {
                    parts.push(name.ident.clone());
                    &name.ident
                };
                self.index
                    .imports
                    .insert(self.scope.local_symbol(local), self.target(&parts));
            }
            UseTree::Rename(rename) => {
                let mut parts = prefix.clone();
                if rename.ident != "self" {
                    parts.push(rename.ident.clone());
                }
                self.index
                    .imports
                    .insert(self.scope.local_symbol(&rename.rename), self.target(&parts));
            }
            UseTree::Glob(_) => {
                let target = self.target(prefix);
                self.index
                    .globs
                    .entry(self.scope.module())
                    .or_default()
                    .push(target);
            }
        }
    }

    fn target(&self, parts: &[Ident]) -> ImportTarget {
        ImportTarget {
            scope: self.scope.clone(),
            path: syn::parse_str(
                &parts
                    .iter()
                    .map(Ident::to_string)
                    .collect::<Vec<_>>()
                    .join("::"),
            )
            .unwrap(),
        }
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if let Some(name) = &node.ident {
            self.index.macro_names.insert(name.to_string());
        }
    }

    fn visit_item_mod(&mut self, node: &'ast ItemMod) {
        self.index
            .modules
            .insert(self.scope.local_symbol(&node.ident));
        if let Some((_, items)) = &node.content {
            self.scope.push(&node.ident);
            for item in items {
                self.visit_item(item);
            }
            self.scope.pop();
        }
    }
    fn visit_item_type(&mut self, node: &'ast ItemType) {
        self.index
            .aliases
            .entry(self.scope.local_symbol(&node.ident))
            .or_default()
            .push(Alias {
                scope: self.scope.clone(),
                generics: node.generics.clone(),
                ty: (*node.ty).clone(),
            });
    }
    fn visit_item_struct(&mut self, node: &'ast ItemStruct) {
        self.index
            .nominal
            .insert(self.scope.local_symbol(&node.ident));
    }
    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        self.index
            .nominal
            .insert(self.scope.local_symbol(&node.ident));
    }
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        self.imports(&node.tree, &mut Vec::new());
    }
}
