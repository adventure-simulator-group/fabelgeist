use super::*;

fn source(path: &Path, syntax: syn::File) -> SourceFile {
    SourceFile {
        path: path.to_string_lossy().into_owned(),
        syntax,
    }
}

fn inspect(sources: &[SourceFile]) -> Vec<Finding> {
    let index = AliasIndex::collect(sources);
    sources
        .iter()
        .flat_map(|source| scan_source(source, &index))
        .collect()
}

#[test]
fn nested_scalars_and_generic_error_returns_are_visible() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            fn sample(counts: Option<Vec<(u64, [f32; 3])>>, label: &str) -> Result<(), String> {}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 4);
    for kind in ["u64", "f32", "str", "String"] {
        assert!(
            found
                .iter()
                .any(|finding| finding.fingerprint.ends_with(kind))
        );
    }
}

#[test]
fn function_reexports_cannot_shadow_modules_during_type_lookup() {
    let sources = [
        source(
            Path::new("crates/example/src/lib.rs"),
            syn::parse_quote! {
                mod device;
                pub use device::device;
                pub struct Device;
            },
        ),
        source(
            Path::new("crates/example/src/device.rs"),
            syn::parse_quote! {
                use crate::Device;
                pub fn device(count: u64) -> Result<Device, u32> { todo!() }
            },
        ),
    ];
    let found = inspect(&sources);
    assert_eq!(
        found.len(),
        2,
        "module/function collisions must not conceal primitive ports"
    );
    let mut leaves = std::collections::BTreeSet::new();
    for finding in found {
        assert_eq!(finding.rule, "raw-interface");
        leaves.insert(finding.fingerprint.rsplit(':').next().unwrap().to_owned());
    }
    assert_eq!(
        leaves,
        std::collections::BTreeSet::from(["u32".to_owned(), "u64".to_owned()])
    );
}

#[test]
fn renamed_imports_and_generic_aliases_do_not_disguise_scalars() {
    let sources = [
        source(
            Path::new("crates/example/src/types.rs"),
            syn::parse_quote! {
                pub type Count = u64;
                pub type Batch<T> = Vec<(T, Count)>;
            },
        ),
        source(
            Path::new("crates/example/src/lib.rs"),
            syn::parse_quote! {
                use crate::types::{Batch as Hidden, Count as Number};
                fn use_counts(values: Hidden<u32>, count: Number) {}
            },
        ),
    ];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    assert_eq!(
        found
            .iter()
            .filter(|finding| finding.fingerprint.ends_with("u64"))
            .count(),
        2
    );
    assert!(
        found
            .iter()
            .any(|finding| finding.fingerprint.ends_with("u32"))
    );
}

#[test]
fn nominal_newtypes_are_not_aliases() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            struct Count(u64);
            struct String(std::string::String);
            fn owned(count: Count, text: String) {}
        },
    )];
    assert!(inspect(&sources).is_empty());
}

#[test]
fn test_helpers_trait_contracts_and_callback_bounds_are_audited() {
    let sources = [source(
        Path::new("crates/example/src/tests.rs"),
        syn::parse_quote! {
            #[cfg(test)] mod fixtures { fn helper(value: u64) {} }
            trait Engine { fn run(&self, enabled: bool); }
            fn callback<F: Fn(u32) -> bool>(callback: F) {}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 5);
    assert!(
        found
            .iter()
            .any(|finding| finding.item.ends_with("fixtures::helper"))
    );
    assert!(
        found
            .iter()
            .any(|finding| finding.item.ends_with("Engine::run"))
    );
}

#[test]
fn macros_and_recursive_aliases_remain_explicit_debt() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            type Loop = Option<Loop>;
            fn cycle(value: Loop) {}
            macro_rules! generated { ($name:ident) => { fn $name(value: u64) {} }; }
        },
    )];
    let found = inspect(&sources);
    assert!(
        found
            .iter()
            .any(|finding| finding.rule == "unexpanded-interface-macro")
    );
    assert!(
        found
            .iter()
            .any(|finding| finding.fingerprint.contains("recursive-alias:"))
    );
}

#[test]
fn parent_imports_reexports_and_cross_crate_globs_resolve_aliases() {
    let sources = [
        source(
            Path::new("crates/owner/src/types.rs"),
            syn::parse_quote! {
                pub type Count = u64;
            },
        ),
        source(
            Path::new("crates/owner/src/lib.rs"),
            syn::parse_quote! {
                pub use crate::types::Count as Total;
            },
        ),
        source(
            Path::new("crates/consumer/src/lib.rs"),
            syn::parse_quote! {
                use owner::Total as Number;
                mod child { fn sample(value: super::Number) {} }
                mod globbed { use owner::*; fn sample(value: Total) {} }
            },
        ),
    ];
    let found = inspect(&sources);
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|finding| finding.fingerprint.ends_with("u64"))
    );
}

#[test]
fn relative_imports_and_default_generic_arguments_are_expanded() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            mod types { pub type List<T = u64> = Vec<T>; }
            use types::List as Batch;
            fn sample(values: Batch) {}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 1);
    assert!(found[0].fingerprint.ends_with("u64"));
}

#[test]
fn imported_builtin_names_and_generic_error_traits_are_visible() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            use std::string::String as Text;
            use std::error::Error as Failure;
            fn sample(value: Text) -> anyhow::Result<()> {}
            fn dynamic() -> Box<dyn Failure> {}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    assert!(
        found
            .iter()
            .any(|finding| finding.fingerprint.ends_with(":String"))
    );
    assert_eq!(
        found
            .iter()
            .filter(|finding| finding.fingerprint.contains("generic-error:"))
            .count(),
        2
    );
}

#[test]
fn explicit_and_inferred_closures_are_not_hidden() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            fn sample() {
                let typed = |value: u64| -> bool { true };
                let inferred = |value| value;
            }
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 4);
    assert!(
        found
            .iter()
            .any(|finding| finding.rule == "unresolved-interface")
    );
}

#[test]
fn nominal_shadowing_does_not_expand_an_unrelated_alias() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            mod unrelated { pub type Count = u64; }
            mod owner {
                struct Count(u64);
                fn sample(value: Count) {}
            }
        },
    )];
    assert!(inspect(&sources).is_empty());
}

#[test]
fn cyclic_imports_and_ambiguous_globs_remain_unresolved_debt() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            mod a { pub type Count = u64; }
            mod b { pub type Count = u32; }
            use a::*;
            use b::*;
            use Cycle as Loop;
            use Loop as Cycle;
            fn sample(value: Count, cyclic: Loop) {}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|finding| finding.fingerprint.contains("unresolved-alias:"))
    );
}

#[test]
fn generic_parameters_shadow_aliases_but_remain_unresolved_debt() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            type Count = u64;
            fn sample<Count>(value: Count) -> Count {}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|finding| finding.rule == "unresolved-interface"
                && finding.fingerprint.ends_with("generic-parameter:Count"))
    );
}

#[test]
fn glob_cycles_do_not_turn_builtin_scalars_into_ambiguous_aliases() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            pub use child::*;
            mod child { use super::*; fn sample(value: f32) {} }
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].fingerprint, "value:f32:f32");
}

#[test]
fn multiple_reexports_of_the_same_owner_are_not_ambiguous() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            mod types { pub struct Count(u64); }
            mod other { pub use crate::types::Count; }
            use types::*;
            use other::*;
            fn sample(value: Count) {}
        },
    )];
    assert!(inspect(&sources).is_empty());
}

#[test]
fn unknown_external_types_and_foreign_signatures_require_review() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            fn sample(value: dependency::OpaqueAlias) {}
            extern "C" { fn bridge(value: u64) -> u32; }
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    assert_eq!(
        found
            .iter()
            .filter(|finding| finding.rule == "unresolved-interface")
            .count(),
        1
    );
    assert_eq!(
        found
            .iter()
            .filter(|finding| finding.rule == "raw-interface")
            .count(),
        2
    );
}

#[test]
fn opaque_macro_and_closure_edits_change_the_debt_fingerprint() {
    let previous = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            macro_rules! helper { () => { fn sample(value: u64) {} }; }
            helper!();
            fn sample() { let call = |value| value; }
        },
    )];
    let changed = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            macro_rules! helper { () => { fn sample(value: u64, flag: bool) {} }; }
            helper!(additional);
            fn sample() { let call = |value| value + 1; }
        },
    )];
    let prior = inspect(&previous);
    let next = inspect(&changed);
    assert_eq!(prior.len(), 4);
    assert_eq!(next.len(), 4);
    assert!(prior.iter().all(|before| {
        !next
            .iter()
            .any(|after| before.fingerprint == after.fingerprint)
    }));
}

#[test]
fn handwritten_macro_calls_inside_functions_cannot_hide_an_added_interface() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            macro_rules! helper { ($type:ty) => { fn local(value: $type) {} }; }
            fn sample() { helper!(u64); }
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .all(|finding| finding.rule == "unexpanded-interface-macro")
    );
}

#[test]
fn primitive_receivers_and_self_returns_are_interfaces_too() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            trait Operation { fn combine(self, other: Self) -> Self; }
            type Raw = u64;
            impl Operation for Raw { fn combine(self, other: Self) -> Self { self + other } }
            struct Owned(u64);
            impl Owned { fn combine(self, other: Self) -> Self { self } }
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    assert!(
        found
            .iter()
            .all(|finding| finding.item == "Raw::combine" && finding.fingerprint.ends_with("u64"))
    );
}

#[test]
fn impl_generic_parameters_shadow_same_named_primitive_aliases() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            type T = u64;
            struct Owned<T>(T);
            impl<T> Owned<T> { fn sample(&self, value: T) -> T {} }
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    assert!(
        found
            .iter()
            .all(|finding| finding.rule == "unresolved-interface"
                && finding.fingerprint.ends_with("generic-parameter:T"))
    );
}

#[test]
fn renamed_and_reexported_handwritten_macros_remain_visible() {
    let sources = [
        source(
            Path::new("crates/owner/src/lib.rs"),
            syn::parse_quote! {
                macro_rules! helper { ($type:ty) => { fn local(value: $type) {} }; }
                pub use helper as Public;
            },
        ),
        source(
            Path::new("crates/consumer/src/lib.rs"),
            syn::parse_quote! {
                use owner::Public as Imported;
                mod globbed { use owner::*; fn sample() { Public!(u64); } }
                fn sample() { Imported!(u32); }
            },
        ),
    ];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    assert!(
        found
            .iter()
            .all(|finding| finding.rule == "unexpanded-interface-macro")
    );
}

#[test]
fn recursive_macro_import_prefixes_do_not_grow_unbounded_paths() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            use cycle::nested as cycle;
            fn sample() { cycle!(); }
        },
    )];
    let _ = inspect(&sources);
}

#[test]
fn associated_bindings_resolve_concrete_errors_and_keep_scalar_or_generic_debt() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            trait Hook {type Error; fn run(&self) -> Result<(), Self::Error>;}
            struct DomainError;
            struct Concrete;
            impl Hook for Concrete {type Error = DomainError; fn run(&self) -> Result<(), Self::Error> {todo!()}}
            struct Scalar;
            impl Hook for Scalar {type Error = String; fn run(&self) -> Result<(), Self::Error> {todo!()}}
            struct Generic<T>(T);
            impl<T> Hook for Generic<T> {type Error = T; fn run(&self) -> Result<(), Self::Error> {todo!()}}
            fn incapable() -> Result<(), std::convert::Infallible> {todo!()}
        },
    )];
    let found = inspect(&sources);
    let mut scalar = Vec::new();
    let mut generic = Vec::new();
    let mut declaration = Vec::new();
    for finding in &found {
        match finding.item.as_str() {
            "Scalar::run" => {
                assert_eq!(finding.rule, "raw-interface");
                assert!(finding.fingerprint.ends_with(":String"));
                scalar.push(finding);
            }
            "Generic<T>::run" => generic.push(finding),
            "Hook::run" => declaration.push(finding),
            _ => panic!("unexpected unresolved interface: {}", finding.item),
        }
    }
    assert_eq!(scalar.len(), 1);
    assert!(!generic.is_empty());
    for finding in generic {
        assert_eq!(finding.rule, "unresolved-interface");
    }
    assert_eq!(declaration.len(), 1);
    assert!(
        declaration[0]
            .fingerprint
            .ends_with("unresolved-type:Self::Error")
    );
}

#[test]
fn only_actual_erased_error_paths_are_generic_errors() {
    let sources = [source(
        Path::new("crates/example/src/error.rs"),
        syn::parse_quote! {
            use std::error::Error as StandardError;
            mod domain {pub struct Error;}
            fn standard() -> Box<dyn StandardError> {todo!()}
            fn qualified() -> Box<dyn std::error::Error> {todo!()}
            fn domain() -> domain::Error {todo!()}
            fn unknown() -> company::error::Error {todo!()}
        },
    )];
    let found = inspect(&sources);
    assert_eq!(found.len(), 3);
    for finding in &found {
        match finding.item.as_str() {
            "error::standard" | "error::qualified" => assert!(
                finding
                    .fingerprint
                    .ends_with("generic-error:std::error::Error")
            ),
            "error::unknown" => assert!(
                finding
                    .fingerprint
                    .ends_with("unresolved-type:company::error::Error")
            ),
            _ => panic!("unexpected finding: {}", finding.item),
        }
    }
}

#[test]
fn standard_error_capabilities_do_not_erase_concrete_generic_types() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            fn concrete<E: std::error::Error>(cause: E) -> E {cause}
            fn arbitrary<T: Clone>(value: T) -> T {value}
            fn erased() -> Box<dyn std::error::Error> {todo!()}
            fn opaque() -> impl std::error::Error {todo!()}
            fn nested<F: FnOnce() -> Box<dyn std::error::Error>>(callback: F) {}
        },
    )];
    let found = inspect(&sources);
    let mut erased = Vec::new();
    let mut concrete = Vec::new();
    for finding in &found {
        if finding.item == "arbitrary" {
            assert_eq!(finding.rule, "unresolved-interface");
            assert!(finding.fingerprint.ends_with("generic-parameter:T"));
            concrete.push(finding);
        }
        if finding
            .fingerprint
            .ends_with("generic-error:std::error::Error")
        {
            erased.push(finding.item.as_str());
        }
    }
    erased.sort();
    assert_eq!(erased, ["erased", "nested", "opaque"]);
    for finding in &found {
        assert_ne!(finding.item, "concrete");
    }
    assert_eq!(concrete.len(), 2);
    assert!(concrete[0].fingerprint.starts_with("value:"));
}

#[test]
fn qualified_container_names_do_not_hide_associated_or_external_types() {
    let sources = [source(
        Path::new("crates/example/src/lib.rs"),
        syn::parse_quote! {
            use std::fmt as native_format;
            fn formatted() -> native_format::Result {todo!()}
            trait Field {type Cell; fn cell(&self) -> Self::Cell;}
            fn external(value: custom::Cell<usize>) {}
            fn standard(value: std::cell::Cell<usize>) {}
        },
    )];
    let found = inspect(&sources);
    let mut external = Vec::new();
    let mut standard = Vec::new();
    let mut projected = Vec::new();
    for finding in found {
        match finding.item.as_str() {
            "Field::cell" => projected.push(finding),
            "external" => external.push(finding),
            "standard" => standard.push(finding),
            _ => panic!("unexpected interface: {}", finding.item),
        }
    }
    assert_eq!(projected.len(), 1);
    assert!(
        projected[0]
            .fingerprint
            .ends_with("unresolved-type:Self::Cell")
    );
    assert_eq!(external.len(), 2);
    assert_eq!(standard.len(), 1);
    assert!(standard[0].fingerprint.ends_with(":usize"));
}
