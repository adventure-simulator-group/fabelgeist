//! Scoped module and import identities for float-container resolution.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use super::SELF_IMPORT;

const CRATE: &str = "crate";
const SUPER: &str = "super";
const STD: &str = "std";
const CORE: &str = "core";
const F32: &str = "f32";
const F64: &str = "f64";
const PRIMITIVE: &str = "primitive";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ScopeId(usize);

#[derive(Clone, Debug, Default)]
pub(super) struct ImportPath {
    pub segments: Vec<String>,
    pub absolute: bool,
}

#[derive(Default)]
pub(super) struct Graph {
    scopes: Vec<Scope>,
    pub libraries: BTreeMap<PathBuf, ScopeId>,
    pub externs: BTreeMap<ScopeId, BTreeMap<String, Vec<PathBuf>>>,
}

struct Scope {
    parent: Option<ScopeId>,
    module: ScopeId,
    root: ScopeId,
    names: BTreeMap<String, Vec<Binding>>,
    globs: Vec<GlobImport>,
}

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd)]
struct Lookup {
    scope: ScopeId,
    name: String,
    mode: LookupMode,
    requesters: BTreeSet<ScopeId>,
}

struct Binding {
    target: BindingTarget,
    access: Access,
}
struct GlobImport {
    path: ImportPath,
    access: Access,
}

#[derive(Clone, Copy)]
pub(super) enum ScopeKind {
    Module,
    Block,
}

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
enum LookupMode {
    Lexical,
    Member,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Target {
    Module(ScopeId),
    StandardLibrary,
    Primitives,
    Float,
    Other,
}

enum BindingTarget {
    Direct(Target),
    Import(ImportPath),
}

enum Access {
    Public,
    Within(ScopeId),
    Restricted(ImportPath),
}

impl Graph {
    pub fn scope(&mut self, parent: Option<ScopeId>, kind: ScopeKind) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope {
            parent,
            module: if matches!(kind, ScopeKind::Module) {
                id
            } else {
                parent.map_or(id, |p| self.scopes[p.0].module)
            },
            root: parent.map_or(id, |p| self.scopes[p.0].root),
            names: BTreeMap::new(),
            globs: Vec::new(),
        });
        id
    }

    pub fn module(
        &mut self,
        parent: ScopeId,
        name: String,
        visibility: &syn::Visibility,
    ) -> ScopeId {
        let child = self.scope(Some(parent), ScopeKind::Module);
        self.bind(
            parent,
            name,
            BindingTarget::Direct(Target::Module(child)),
            visibility,
        );
        child
    }

    pub fn other(&mut self, scope: ScopeId, name: String, visibility: &syn::Visibility) {
        self.bind(
            scope,
            name,
            BindingTarget::Direct(Target::Other),
            visibility,
        );
    }

    pub fn self_alias(&mut self, scope: ScopeId, name: String, visibility: &syn::Visibility) {
        let root = self.scopes[scope.0].root;
        self.bind(
            scope,
            name,
            BindingTarget::Direct(Target::Module(root)),
            visibility,
        );
    }

    pub fn import(
        &mut self,
        scope: ScopeId,
        name: String,
        path: ImportPath,
        visibility: &syn::Visibility,
    ) {
        self.bind(scope, name, BindingTarget::Import(path), visibility);
    }

    pub fn glob(&mut self, scope: ScopeId, path: ImportPath, visibility: &syn::Visibility) {
        let access = Self::access(scope, visibility);
        self.scopes[scope.0].globs.push(GlobImport { path, access });
    }

    pub fn is_float(&self, scope: ScopeId, path: &ImportPath) -> bool {
        self.resolve(scope, path, &mut BTreeSet::new())
            .contains(&Target::Float)
    }

    fn bind(
        &mut self,
        scope: ScopeId,
        name: String,
        target: BindingTarget,
        visibility: &syn::Visibility,
    ) {
        let access = Self::access(scope, visibility);
        self.scopes[scope.0]
            .names
            .entry(name)
            .or_default()
            .push(Binding { target, access });
    }

    fn access(scope: ScopeId, visibility: &syn::Visibility) -> Access {
        match visibility {
            syn::Visibility::Public(_) => Access::Public,
            syn::Visibility::Inherited => Access::Within(scope),
            syn::Visibility::Restricted(restriction) => Access::Restricted(ImportPath {
                segments: restriction
                    .path
                    .segments
                    .iter()
                    .map(|segment| segment.ident.to_string())
                    .collect(),
                absolute: restriction.path.leading_colon.is_some(),
            }),
        }
    }

    fn visible(
        &self,
        access: &Access,
        declaration: ScopeId,
        requester: ScopeId,
        seen: &mut BTreeSet<Lookup>,
    ) -> bool {
        match access {
            Access::Public => true,
            Access::Within(boundary) => self.descendant(requester, *boundary),
            Access::Restricted(path) => self.resolve(declaration, path, seen).into_iter().any(|target| {
                matches!(target, Target::Module(boundary) if self.descendant(requester, boundary))
            }),
        }
    }

    fn descendant(&self, mut scope: ScopeId, boundary: ScopeId) -> bool {
        loop {
            if scope == boundary {
                return true;
            }
            let Some(parent) = self.scopes[scope.0].parent else {
                return false;
            };
            scope = parent;
        }
    }

    fn resolve(
        &self,
        scope: ScopeId,
        path: &ImportPath,
        seen: &mut BTreeSet<Lookup>,
    ) -> BTreeSet<Target> {
        let Some((head, tail)) = path.segments.split_first() else {
            return BTreeSet::new();
        };
        let current = &self.scopes[scope.0];
        let mut targets = match head.as_str() {
            CRATE if !path.absolute => BTreeSet::from([Target::Module(current.root)]),
            SELF_IMPORT if !path.absolute => BTreeSet::from([Target::Module(current.module)]),
            SUPER if !path.absolute => self
                .parent_module(current.module)
                .map(Target::Module)
                .into_iter()
                .collect(),
            _ if path.absolute => self.external(scope, head),
            _ => self.lookup(
                scope,
                head,
                LookupMode::Lexical,
                &BTreeSet::from([scope]),
                seen,
            ),
        };
        for segment in tail {
            targets = targets
                .into_iter()
                .flat_map(|target| match target {
                    Target::Module(module) if segment == SUPER => self
                        .parent_module(module)
                        .map(Target::Module)
                        .into_iter()
                        .collect(),
                    Target::Module(module) => self.lookup(
                        module,
                        segment,
                        LookupMode::Member,
                        &BTreeSet::from([scope]),
                        seen,
                    ),
                    Target::StandardLibrary if segment == PRIMITIVE => {
                        BTreeSet::from([Target::Primitives])
                    }
                    Target::StandardLibrary | Target::Primitives
                        if matches!(segment.as_str(), F32 | F64) =>
                    {
                        BTreeSet::from([Target::Float])
                    }
                    _ => BTreeSet::new(),
                })
                .collect();
        }
        targets
    }

    fn lookup(
        &self,
        scope: ScopeId,
        name: &str,
        mode: LookupMode,
        requesters: &BTreeSet<ScopeId>,
        seen: &mut BTreeSet<Lookup>,
    ) -> BTreeSet<Target> {
        let key = Lookup {
            scope,
            name: name.into(),
            mode,
            requesters: requesters.clone(),
        };
        if !seen.insert(key.clone()) {
            // `use core as core` (and a same-name external-crate import)
            // refers to the extern prelude rather than creating a dead cycle.
            return if mode == LookupMode::Lexical {
                self.external(scope, name)
            } else {
                BTreeSet::new()
            };
        }
        let current = &self.scopes[scope.0];
        let mut targets = BTreeSet::new();
        if let Some(bindings) = current.names.get(name) {
            // A local declaration shadows parents and the extern prelude even
            // when it is unrelated to a float module or cannot be resolved.
            for binding in bindings {
                if !requesters
                    .iter()
                    .all(|requester| self.visible(&binding.access, scope, *requester, seen))
                {
                    continue;
                }
                match &binding.target {
                    BindingTarget::Direct(target) => {
                        targets.insert(*target);
                    }
                    BindingTarget::Import(path) => targets.extend(self.resolve(scope, path, seen)),
                }
            }
        } else {
            for glob in &current.globs {
                if !requesters
                    .iter()
                    .all(|requester| self.visible(&glob.access, scope, *requester, seen))
                {
                    continue;
                }
                let mut readers = requesters.clone();
                readers.insert(scope);
                for target in self.resolve(scope, &glob.path, seen) {
                    if let Target::Module(module) = target {
                        // Every forwarding scope and the final reader must
                        // be allowed to see a member. A public glob cannot
                        // widen a private or restricted member's visibility.
                        targets.extend(self.lookup(
                            module,
                            name,
                            LookupMode::Member,
                            &readers,
                            seen,
                        ));
                    }
                }
            }
            if targets.is_empty() && mode == LookupMode::Lexical {
                // Blocks inherit lexical bindings. Module scopes do not
                // inherit unqualified parent bindings.
                targets = if current.module == scope {
                    self.external(scope, name)
                } else {
                    current.parent.map_or_else(
                        || self.external(scope, name),
                        |parent| self.lookup(parent, name, LookupMode::Lexical, requesters, seen),
                    )
                };
            }
        }
        seen.remove(&key);
        targets
    }

    fn external(&self, scope: ScopeId, name: &str) -> BTreeSet<Target> {
        let root = self.scopes[scope.0].root;
        if let Some(packages) = self
            .externs
            .get(&root)
            .and_then(|externs| externs.get(name))
        {
            return packages
                .iter()
                .filter_map(|package| self.libraries.get(package))
                .copied()
                .map(Target::Module)
                .collect();
        }
        match name {
            STD | CORE => BTreeSet::from([Target::StandardLibrary]),
            F32 | F64 => BTreeSet::from([Target::Float]),
            _ => BTreeSet::new(),
        }
    }

    fn parent_module(&self, module: ScopeId) -> Option<ScopeId> {
        self.scopes[module.0]
            .parent
            .map(|parent| self.scopes[parent.0].module)
    }
}
