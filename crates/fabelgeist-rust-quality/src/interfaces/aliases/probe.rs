//! Inspect scalar leaves while substituting type aliases and their arguments.
use super::{Alias, AliasIndex, Resolution, SourceScope, Symbol};
use std::collections::{BTreeMap, BTreeSet};
use syn::{GenericParam, Generics, PathArguments, Type, TypePath, visit::Visit};

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct InterfaceLeaf {
    kind: InterfaceKind,
    pub(crate) spelling: String,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
enum InterfaceKind {
    Raw,
    StandardError,
    Unresolved,
}

impl InterfaceLeaf {
    fn from_raw(spelling: String) -> Self {
        Self {
            kind: InterfaceKind::Raw,
            spelling,
        }
    }

    fn from_unresolved(spelling: String) -> Self {
        Self {
            kind: InterfaceKind::Unresolved,
            spelling,
        }
    }

    pub(crate) fn as_rule_name(&self) -> &'static str {
        match self.kind {
            InterfaceKind::Raw | InterfaceKind::StandardError => "raw-interface",
            InterfaceKind::Unresolved => "unresolved-interface",
        }
    }

    pub(super) fn from_symbol(symbol: &Symbol) -> Option<Self> {
        let name = symbol.0.last()?;
        let root = symbol.0.first()?;
        let builtin = symbol.0.len() == 1 || matches!(root.as_str(), "std" | "core" | "alloc");
        if builtin
            && matches!(
                name.as_str(),
                "bool"
                    | "char"
                    | "str"
                    | "String"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "f16"
                    | "f32"
                    | "f64"
                    | "f128"
            )
        {
            return Some(Self::from_raw(name.clone()));
        }
        if (matches!(root.as_str(), "anyhow" | "eyre" | "color_eyre")
            && matches!(name.as_str(), "Error" | "Report" | "Result"))
            || (name == "Error"
                && (symbol.0.len() == 1
                    || (symbol.0.len() == 3
                        && matches!(root.as_str(), "std" | "core")
                        && symbol.0[1] == "error")))
        {
            return Some(Self {
                kind: if symbol.0.len() == 3 && matches!(root.as_str(), "std" | "core") {
                    InterfaceKind::StandardError
                } else {
                    InterfaceKind::Raw
                },
                spelling: format!("generic-error:{}", symbol.0.join("::")),
            });
        }
        None
    }
}

#[derive(Clone, Default)]
pub(crate) struct RawTypes {
    pub(crate) values: Vec<InterfaceLeaf>,
}

pub(super) struct TypeProbe<'a> {
    index: &'a AliasIndex,
    scope: SourceScope,
    pub(super) result: RawTypes,
    bindings: BTreeMap<String, RawTypes>,
    visiting: BTreeSet<Symbol>,
    trait_context: TraitContext,
}

#[derive(Clone, Copy)]
enum TraitContext {
    Capability,
    Opaque,
}

impl<'a> TypeProbe<'a> {
    pub(super) fn bind_self(&mut self, owner: RawTypes) {
        self.bindings.insert("Self".into(), owner);
    }

    pub(super) fn bind_associated(&mut self, owner: &Type) {
        let Some(associations) = self
            .index
            .associated
            .for_owner(owner, &self.scope, self.index)
        else {
            return;
        };
        for (name, aliases) in associations {
            if let [alias] = aliases.as_slice() {
                let previous = std::mem::replace(&mut self.scope, alias.scope.clone());
                let raw = self.capture(&alias.ty);
                self.scope = previous;
                self.bindings.insert(name.0.join("::"), raw);
            }
        }
    }

    pub(super) fn for_signature(
        index: &'a AliasIndex,
        scope: &SourceScope,
        generics: &Generics,
    ) -> Self {
        let mut probe = Self {
            index,
            scope: scope.clone(),
            result: RawTypes::default(),
            bindings: generics
                .params
                .iter()
                .filter_map(|param| {
                    if let GenericParam::Type(param) = param {
                        Some((
                            param.ident.to_string(),
                            RawTypes {
                                values: vec![InterfaceLeaf::from_unresolved(format!(
                                    "generic-parameter:{}",
                                    param.ident
                                ))],
                            },
                        ))
                    } else {
                        None
                    }
                })
                .collect(),
            visiting: BTreeSet::new(),
            trait_context: TraitContext::Capability,
        };
        probe.bind_capabilities(generics);
        probe
    }

    fn bind_capabilities(&mut self, generics: &Generics) {
        for param in &generics.params {
            if let GenericParam::Type(param) = param {
                self.bind_error_capability(&param.ident, &param.bounds);
            }
        }
        if let Some(clause) = &generics.where_clause {
            for predicate in &clause.predicates {
                if let syn::WherePredicate::Type(predicate) = predicate
                    && let Type::Path(path) = &predicate.bounded_ty
                    && let Some(ident) = path.path.get_ident()
                {
                    self.bind_error_capability(ident, &predicate.bounds);
                }
            }
        }
    }

    fn bind_error_capability(
        &mut self,
        ident: &syn::Ident,
        bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
    ) {
        for bound in bounds {
            if let syn::TypeParamBound::Trait(bound) = bound
                && let Resolution::Raw(leaf) = self.index.resolve(&bound.path, &self.scope)
                && matches!(leaf.kind, InterfaceKind::StandardError)
            {
                // No scalar implements this foreign standard trait. Keep
                // arbitrary bounds unresolved; nested dyn/impl errors are
                // still inspected separately in the generic bound census.
                self.bindings.insert(ident.to_string(), RawTypes::default());
            }
        }
    }

    fn expand(&mut self, path: &TypePath, symbol: Symbol, alias: &Alias) {
        if !self.visiting.insert(symbol.clone()) {
            self.result
                .values
                .push(InterfaceLeaf::from_unresolved(format!(
                    "recursive-alias:{}",
                    symbol.0.join("::")
                )));
            return;
        }
        let explicit: Vec<_> = path
            .path
            .segments
            .last()
            .into_iter()
            .flat_map(|last| match &last.arguments {
                PathArguments::AngleBracketed(arguments) => arguments
                    .args
                    .iter()
                    .filter_map(|argument| match argument {
                        syn::GenericArgument::Type(ty) => Some(ty),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            })
            .map(|ty| self.capture(ty))
            .collect();
        let scope = std::mem::replace(&mut self.scope, alias.scope.clone());
        let previous = std::mem::take(&mut self.bindings);
        let mut explicit = explicit.into_iter();
        for param in &alias.generics.params {
            if let GenericParam::Type(param) = param {
                let raw = explicit.next().unwrap_or_else(|| {
                    param
                        .default
                        .as_ref()
                        .map_or_else(RawTypes::default, |ty| self.capture(ty))
                });
                self.bindings.insert(param.ident.to_string(), raw);
            }
        }
        self.visit_type(&alias.ty);
        self.bindings = previous;
        self.scope = scope;
        self.visiting.remove(&symbol);
    }

    fn capture(&mut self, ty: &Type) -> RawTypes {
        let previous = std::mem::take(&mut self.result);
        self.visit_type(ty);
        std::mem::replace(&mut self.result, previous)
    }
}

impl<'ast> Visit<'ast> for TypeProbe<'_> {
    fn visit_type_infer(&mut self, _node: &'ast syn::TypeInfer) {
        self.result
            .values
            .push(InterfaceLeaf::from_unresolved("inferred-type".into()));
    }

    fn visit_type_macro(&mut self, node: &'ast syn::TypeMacro) {
        use quote::ToTokens;
        self.result
            .values
            .push(InterfaceLeaf::from_unresolved(format!(
                "unexpanded-type-macro:{}",
                node.mac.path.to_token_stream()
            )));
    }

    fn visit_trait_bound(&mut self, node: &'ast syn::TraitBound) {
        if let Resolution::Raw(leaf) = self.index.resolve(&node.path, &self.scope) {
            match (&leaf.kind, self.trait_context) {
                (InterfaceKind::StandardError, TraitContext::Capability) => {}
                _ => self.result.values.push(leaf),
            }
        }
        syn::visit::visit_trait_bound(self, node);
    }

    fn visit_type_trait_object(&mut self, node: &'ast syn::TypeTraitObject) {
        let previous = std::mem::replace(&mut self.trait_context, TraitContext::Opaque);
        syn::visit::visit_type_trait_object(self, node);
        self.trait_context = previous;
    }

    fn visit_type_impl_trait(&mut self, node: &'ast syn::TypeImplTrait) {
        let previous = std::mem::replace(&mut self.trait_context, TraitContext::Opaque);
        syn::visit::visit_type_impl_trait(self, node);
        self.trait_context = previous;
    }

    fn visit_type_path(&mut self, node: &'ast TypePath) {
        use quote::ToTokens;
        let spelling = node.path.to_token_stream().to_string().replace(' ', "");
        if node.qself.is_none()
            && let Some(raw) = self.bindings.get(&spelling)
        {
            self.result.values.extend(raw.values.clone());
            return;
        }
        match self.index.resolve(&node.path, &self.scope) {
            Resolution::Alias(symbol, alias) => {
                self.expand(node, symbol, alias);
                return;
            }
            Resolution::Ambiguous(symbol) => {
                self.result
                    .values
                    .push(InterfaceLeaf::from_unresolved(format!(
                        "unresolved-alias:{}",
                        symbol.0.join("::")
                    )));
            }
            Resolution::Nominal(_) => {}
            Resolution::Raw(leaf) => self.result.values.push(leaf),
            Resolution::Missing => {
                use quote::ToTokens;
                let name = node.path.segments.last().unwrap().ident.to_string();
                // Containers still expose their generic arguments below.
                // Self denotes the existing impl/trait owner, not an alias.
                let root = node.path.segments.first().unwrap();
                let standard_container = node.path.segments.len() == 1
                    || matches!(root.ident.to_string().as_str(), "std" | "core" | "alloc");
                if !standard_container
                    || !matches!(
                        name.as_str(),
                        "Self"
                            | "Vec"
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
                    )
                {
                    self.result
                        .values
                        .push(InterfaceLeaf::from_unresolved(format!(
                            "unresolved-type:{}",
                            node.to_token_stream().to_string().replace(' ', "")
                        )));
                }
            }
        }
        syn::visit::visit_type_path(self, node);
    }
}
