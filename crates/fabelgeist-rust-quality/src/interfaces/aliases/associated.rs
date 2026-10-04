//! Resolve concrete associated bindings without guessing generic projections.
use super::{Alias, AliasIndex, Resolution, SourceScope, Symbol};
use crate::interfaces::SourceFile;
use std::collections::BTreeMap;
use syn::{GenericParam, ItemImpl, ItemMod, Type, visit::Visit};

#[derive(Default)]
pub(super) struct AssociatedTypes {
    owners: BTreeMap<Symbol, BTreeMap<Symbol, Vec<Alias>>>,
}
impl AssociatedTypes {
    pub(super) fn collect(sources: &[SourceFile], index: &AliasIndex) -> Self {
        let mut result = Self::default();
        for source in sources {
            Collector {
                index,
                result: &mut result,
                scope: SourceScope::from_source(source),
            }
            .visit_file(&source.syntax);
        }
        result
    }
    pub(super) fn for_owner(
        &self,
        owner: &Type,
        scope: &SourceScope,
        index: &AliasIndex,
    ) -> Option<&BTreeMap<Symbol, Vec<Alias>>> {
        let Type::Path(path) = owner else {
            return None;
        };
        let Resolution::Nominal(symbol) = index.resolve(&path.path, scope) else {
            return None;
        };
        self.owners.get(&symbol)
    }
}
struct Collector<'a> {
    index: &'a AliasIndex,
    result: &'a mut AssociatedTypes,
    scope: SourceScope,
}
impl<'ast> Visit<'ast> for Collector<'_> {
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
        // Generic impl substitutions need a separate proof. Keep those
        // projections unresolved instead of treating a trait bound as a type.
        for param in &node.generics.params {
            if matches!(param, GenericParam::Type(_)) {
                return;
            }
        }
        let Type::Path(path) = node.self_ty.as_ref() else {
            return;
        };
        let Resolution::Nominal(owner) = self.index.resolve(&path.path, &self.scope) else {
            return;
        };
        for item in &node.items {
            if let syn::ImplItem::Type(item) = item {
                self.result
                    .owners
                    .entry(owner.clone())
                    .or_default()
                    .entry(Symbol(vec!["Self".into(), item.ident.to_string()]))
                    .or_default()
                    .push(Alias {
                        scope: self.scope.clone(),
                        generics: item.generics.clone(),
                        ty: item.ty.clone(),
                    });
            }
        }
    }
}
