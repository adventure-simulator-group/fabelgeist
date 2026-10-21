use super::super::{Finding, ParsedFile, check_with_globs};
use super::float_globs;

fn findings(sources: &[(&str, &str)]) -> Vec<Finding> {
    let files = sources
        .iter()
        .map(|(path, source)| ParsedFile {
            path: (*path).into(),
            syntax: syn::parse_file(source).unwrap(),
        })
        .collect::<Vec<_>>();
    check_with_globs(&files, &float_globs(None, &files).unwrap())
}

#[test]
fn unrelated_crates_can_bind_the_same_terminal_name() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            "use std::f32 as values; pub fn minimum() -> f32 { values::MIN }",
        ),
        (
            "crates/b/src/lib.rs",
            r#"
            mod values { pub fn count() -> usize { 0 } }
            use values::*;
            pub fn count_items() -> usize { count() }
        "#,
        ),
    ]);
    assert!(actual.is_empty(), "{actual:#?}");
}

#[test]
fn unrelated_modules_do_not_share_private_aliases() {
    let actual = findings(&[
        ("crates/a/src/lib.rs", "mod numeric; mod ordinary;"),
        (
            "crates/a/src/numeric.rs",
            "use std::f32 as values; pub fn minimum() -> f32 { values::MIN }",
        ),
        (
            "crates/a/src/ordinary.rs",
            r#"
            mod values { pub fn count() -> usize { 0 } }
            use values::*;
            pub fn count_items() -> usize { count() }
        "#,
        ),
    ]);
    assert!(actual.is_empty(), "{actual:#?}");
}

#[test]
fn blocks_and_macro_bodies_shadow_names_without_leaking() {
    let actual = findings(&[(
        "crates/a/src/lib.rs",
        r#"
        use std::f32 as values;
        fn ordinary() { mod values {} use values::*; }
        fn floating_call() { use values::*; }
        macro_rules! ordinary_macro { () => {{ mod values {} use values::*; }}; }
        macro_rules! floating_macro { () => {{ use values::*; }}; }
        fn private_alias() { use std::f32 as local; }
        fn unrelated() { mod local {} use local::*; }
    "#,
    )]);
    assert_eq!(actual.len(), 2, "{actual:#?}");
    assert_eq!(actual[0].item, "floating_call");
    assert_eq!(actual[1].item, "floating_macro");
}

#[test]
fn binary_and_integration_test_roots_do_not_inherit_library_bindings() {
    let actual = findings(&[
        ("crates/a/src/lib.rs", "use std::f32 as values;"),
        (
            "crates/a/src/main.rs",
            "mod values {} use values::*; fn main() {}",
        ),
        (
            "crates/a/tests/independent.rs",
            "mod values {} use values::*;",
        ),
    ]);
    assert!(actual.is_empty(), "{actual:#?}");
}

#[test]
fn qualified_reexports_and_module_globs_keep_their_provenance() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            #[path = "shared/source.rs"] pub mod origin;
            mod relay; mod consumer;
            pub use relay::*;
            use crate::origin::values::*;
        "#,
        ),
        (
            "crates/a/src/shared/source.rs",
            "pub use ::core::f64 as values;",
        ),
        (
            "crates/a/src/relay.rs",
            "pub use super::origin::{values as number};",
        ),
        (
            "crates/a/src/consumer.rs",
            r#"
            mod values {} use self::values::*;
            use crate::relay as modules;
            use modules::number::*;
            use super::number::*;
        "#,
        ),
        ("crates/a/tests/numeric.rs", "use a::number::*;"),
    ]);
    assert_eq!(actual.len(), 4, "{actual:#?}");
    assert_eq!(
        actual
            .iter()
            .filter(|finding| finding.path.ends_with("consumer.rs"))
            .count(),
        2
    );
    assert!(
        actual
            .iter()
            .any(|finding| finding.path.ends_with("tests/numeric.rs"))
    );
}

#[test]
fn cyclic_globs_terminate_without_hiding_resolvable_names() {
    let actual = findings(&[(
        "crates/a/src/lib.rs",
        r#"
        mod left { pub use super::right::*; pub use std::f32 as values; }
        mod right { pub use super::left::*; }
        use right::values::*;
        use right::missing::*;
    "#,
    )]);
    assert_eq!(actual.len(), 1, "{actual:#?}");
}

#[test]
fn explicit_crate_aliases_and_same_name_prelude_imports_resolve() {
    let actual = findings(&[(
        "crates/a/src/lib.rs",
        r#"
        extern crate self as own;
        mod source { pub use std::f32 as number; }
        use own::source::number::*;
        mod another {
            use core as core;
            use core::f64::*;
        }
    "#,
    )]);
    assert_eq!(actual.len(), 2, "{actual:#?}");
}

#[test]
fn module_files_and_explicit_paths_use_their_own_child_directories() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            mod ordinary;
            #[path="redirect.rs"] mod logical;
            use ordinary::inner::values::*;
            use logical::child::values::*;
            use logical::inline::leaf::values::*;
        "#,
        ),
        ("crates/a/src/ordinary.rs", "pub mod inner;"),
        (
            "crates/a/src/ordinary/inner.rs",
            "pub use std::f32 as values;",
        ),
        (
            "crates/a/src/redirect.rs",
            r#"
            pub mod child;
            pub mod inline { #[path="another.rs"] pub mod leaf; }
        "#,
        ),
        ("crates/a/src/child.rs", "pub use std::f64 as values;"),
        (
            "crates/a/src/inline/another.rs",
            "pub use core::f32 as values;",
        ),
    ]);
    assert_eq!(actual.len(), 3, "{actual:#?}");
}

#[test]
fn value_namespace_items_do_not_shadow_imported_modules() {
    let actual = findings(&[(
        "crates/a/src/lib.rs",
        r#"
        use std::f32 as values;
        fn floating() { fn values() {} use values::*; }
    "#,
    )]);
    assert_eq!(actual.len(), 1, "{actual:#?}");
}

#[test]
fn restricted_aliases_follow_module_and_crate_visibility() {
    let actual = findings(&[(
        "crates/a/src/lib.rs",
        r#"
        mod source {
            use std::f32 as private_values;
            pub(crate) use std::f64 as crate_values;
            pub(super) use std::f32 as super_values;
            pub(in crate::source) use std::f64 as restricted_values;
            pub mod inner {
                use super::private_values::*;
                use super::restricted_values::*;
            }
        }
        mod sibling {
            use crate::source::*;
            use crate_values::*;
            use super_values::*;
        }
    "#,
    )]);
    assert_eq!(actual.len(), 4, "{actual:#?}");
}

#[test]
fn literal_fragments_share_aliases_with_the_including_module() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            use std::f32 as values;
            include!("checks.rs");
            include!("aliases.rs");
            use doubles::*;
        "#,
        ),
        (
            "crates/a/src/checks.rs",
            "\nuse values::*;\npub fn near_zero(x: f32) -> bool { x <= EPSILON }",
        ),
        ("crates/a/src/aliases.rs", "use core::f64 as doubles;"),
    ]);
    assert_eq!(actual.len(), 2, "{actual:#?}");
    let fragment = actual
        .iter()
        .find(|finding| finding.path == "crates/a/src/checks.rs")
        .unwrap();
    assert_eq!(fragment.line, 2);
    assert_eq!(fragment.item, "<module>");
}

#[test]
fn nested_fragments_and_their_modules_resolve_from_the_fragment_source() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            mod inline {
                use std::f32 as values;
                include!("parts/body.rs");
                use child::values::*;
            }
        "#,
        ),
        (
            "crates/a/src/parts/body.rs",
            "include!(\"nested/checks.rs\"); mod child;",
        ),
        (
            "crates/a/src/parts/nested/checks.rs",
            "use values::*; pub fn near_zero(x: f32) -> bool { x <= EPSILON }",
        ),
        (
            "crates/a/src/parts/child.rs",
            "pub use core::f64 as values;",
        ),
    ]);
    assert_eq!(actual.len(), 2, "{actual:#?}");
    assert!(
        actual
            .iter()
            .any(|finding| finding.path.ends_with("nested/checks.rs"))
    );
}

#[test]
fn included_bindings_stay_in_their_module_block_and_crate() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            mod numeric { use std::f32 as values; include!("numeric.rs"); }
            mod ordinary { mod values {} include!("ordinary.rs"); }
            fn inside() { include!("aliases.rs"); use local::*; }
            fn outside() { mod local {} use local::*; }
        "#,
        ),
        ("crates/a/src/numeric.rs", "use values::*;"),
        ("crates/a/src/ordinary.rs", "use values::*;"),
        ("crates/a/src/aliases.rs", "use core::f64 as local;"),
        (
            "crates/b/src/lib.rs",
            "mod local {} use local::*; mod values {} include!(\"ordinary.rs\");",
        ),
        ("crates/b/src/ordinary.rs", "use values::*;"),
    ]);
    assert_eq!(actual.len(), 2, "{actual:#?}");
    assert!(actual.iter().any(|finding| finding.item == "inside"));
    assert!(
        actual
            .iter()
            .any(|finding| finding.path.ends_with("numeric.rs"))
    );
}

#[test]
fn a_shared_fragment_is_resolved_in_every_caller_and_reported_once() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            mod ordinary {
                mod values { pub const EPSILON: f32 = 0.0; }
                include!("shared.rs");
            }
            mod numeric { use std::f32 as values; include!("shared.rs"); }
            mod another { use core::f32 as values; include!("shared.rs"); }
        "#,
        ),
        (
            "crates/a/src/shared.rs",
            "use values::*; pub fn current() -> f32 { EPSILON }",
        ),
    ]);
    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].path, "crates/a/src/shared.rs");
}

#[test]
fn recursive_includes_terminate_without_hiding_inherited_aliases() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            "use std::f32 as values; include!(\"first.rs\");",
        ),
        ("crates/a/src/first.rs", "include!(\"second.rs\");"),
        (
            "crates/a/src/second.rs",
            "include!(\"first.rs\"); use values::*;",
        ),
    ]);
    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].path, "crates/a/src/second.rs");
}

#[test]
fn macro_bodies_follow_literal_includes_in_their_own_scope() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            macro_rules! numeric {
                () => {{ use core::f64 as values; ::std::include!(r"checks.rs",); }};
            }
            macro_rules! ordinary {
                () => {{ mod values {} core::include!("ordinary.rs"); }};
            }
            mod local {} use local::*;
        "#,
        ),
        (
            "crates/a/src/checks.rs",
            "use values::*; use std::f32 as local;",
        ),
        ("crates/a/src/ordinary.rs", "use values::*;"),
    ]);
    assert_eq!(actual.len(), 1, "{actual:#?}");
    assert_eq!(actual[0].path, "crates/a/src/checks.rs");
}

#[test]
fn text_custom_and_nonliteral_macros_do_not_inherit_include_semantics() {
    let actual = findings(&[
        (
            "crates/a/src/lib.rs",
            r#"
            use std::f32 as values;
            custom::include!("checks.rs");
            macro_rules! text {
                () => {{
                    custom::include!("checks.rs");
                    include_str!("checks.rs");
                    include!(concat!("checks", ".rs"));
                }};
            }
        "#,
        ),
        ("crates/a/src/checks.rs", "use values::*;"),
    ]);
    assert!(actual.is_empty(), "{actual:#?}");
}
