//! Keep Viabundus source intervals distinct from the positive game calendar.

use syn::{Attribute, Fields, GenericArgument, Item, PathArguments, Type, Visibility};

pub const PATH: &str = "crates/adventuresim-world-import/src/sources/viabundus/chronology.rs";
pub const IMPORTER_CRATE: &str = "crates/adventuresim-world-import";
const CFG_ATTRIBUTE: &str = "cfg";
const CFG_ATTR_ATTRIBUTE: &str = "cfg_attr";

pub fn check_file(syntax: &syn::File) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if syntax
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident(CFG_ATTRIBUTE) || attr.path().is_ident(CFG_ATTR_ATTRIBUTE))
    {
        diagnostics.push(format!(
            "{PATH}: source-year contract cannot be conditional"
        ));
    }

    let owner = syntax.items.iter().any(|item| match item {
        Item::Struct(item) if item.ident == "SourceYear" => {
            permitted_attributes(&item.attrs, true)
                && item.generics.params.is_empty()
                && item.generics.where_clause.is_none()
                && !item.fields.is_empty()
                && item.fields.iter().all(|field| {
                    matches!(field.vis, Visibility::Inherited)
                        && permitted_attributes(&field.attrs, false)
                })
        }
        _ => false,
    });
    if !owner {
        diagnostics.push(format!("{PATH}: restore the private SourceYear owner"));
    }

    let interval = syntax.items.iter().any(|item| match item {
        Item::Struct(item) if item.ident == "ActiveInterval" => {
            permitted_attributes(&item.attrs, true)
                && field_has_type(&item.fields, "from", true)
                && field_has_type(&item.fields, "to", true)
        }
        _ => false,
    });
    if !interval {
        diagnostics.push(format!("{PATH}: keep interval endpoints as source years"));
    }

    let feature = syntax.items.iter().any(|item| match item {
        Item::Enum(item) if item.ident == "DatedFeature" => {
            permitted_attributes(&item.attrs, true)
                && item.variants.iter().any(|variant| {
                    variant.ident == "Absent"
                        && permitted_attributes(&variant.attrs, false)
                        && matches!(variant.fields, Fields::Unit)
                })
                && item.variants.iter().any(|variant| {
                    variant.ident == "ContradictoryEmptyRange"
                        && permitted_attributes(&variant.attrs, false)
                        && field_has_type(&variant.fields, "year", false)
                })
                && item.variants.iter().any(|variant| {
                    variant.ident == "Present"
                        && permitted_attributes(&variant.attrs, false)
                        && field_has_type(&variant.fields, "from", true)
                        && field_has_type(&variant.fields, "to", true)
                })
        }
        _ => false,
    });
    if !feature {
        diagnostics.push(format!(
            "{PATH}: keep dated-feature endpoints as source years"
        ));
    }
    diagnostics
}

fn permitted_attributes(attributes: &[Attribute], allow_derive: bool) -> bool {
    attributes.iter().all(|attribute| {
        attribute.path().is_ident("doc") || (allow_derive && attribute.path().is_ident("derive"))
    })
}

fn field_has_type(fields: &Fields, name: &str, optional: bool) -> bool {
    fields.iter().any(|field| {
        field.ident.as_ref().is_some_and(|ident| ident == name)
            && permitted_attributes(&field.attrs, false)
            && if optional {
                optional_source_year(&field.ty)
            } else {
                simple_type(&field.ty, "SourceYear")
            }
    })
}

fn optional_source_year(ty: &Type) -> bool {
    let Type::Path(path) = ty else { return false };
    if path.qself.is_some() || path.path.leading_colon.is_some() || path.path.segments.len() != 1 {
        return false;
    }
    let segment = &path.path.segments[0];
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    segment.ident == "Option"
        && arguments.args.len() == 1
        && matches!(arguments.args.first(), Some(GenericArgument::Type(inner)) if simple_type(inner, "SourceYear"))
}

fn simple_type(ty: &Type, name: &str) -> bool {
    let Type::Path(path) = ty else { return false };
    path.qself.is_none()
        && path.path.leading_colon.is_none()
        && path.path.segments.len() == 1
        && path.path.segments[0].ident == name
        && matches!(path.path.segments[0].arguments, PathArguments::None)
}

#[cfg(test)]
mod tests {
    use super::check_file;

    const CONTRACT: &str = r#"
        pub(super) struct SourceYear { year: i32 }
        pub(super) struct ActiveInterval { from: Option<SourceYear>, to: Option<SourceYear> }
        pub(super) enum DatedFeature {
            Absent,
            ContradictoryEmptyRange { year: SourceYear },
            Present { from: Option<SourceYear>, to: Option<SourceYear> },
        }
    "#;

    fn diagnostics(source: &str) -> Vec<String> {
        check_file(&syn::parse_file(source).unwrap())
    }

    #[test]
    fn accepts_the_distinct_owner_and_typed_intervals() {
        assert!(diagnostics(CONTRACT).is_empty());
        assert!(diagnostics(&format!("{CONTRACT} mod unrelated {{ #![cfg(any())] }}")).is_empty());
        let checked_backing = CONTRACT.replace("year: i32", "year: NonZeroI32");
        assert!(diagnostics(&checked_backing).is_empty());
    }

    #[test]
    fn rejects_missing_or_wrong_source_year_ownership() {
        for source in [
            CONTRACT.replace(
                "pub(super) struct SourceYear { year: i32 }",
                "type SourceYear = CalendarYear;",
            ),
            CONTRACT.replace("year: i32", "pub year: i32"),
            CONTRACT.replace("from: Option<SourceYear>", "from: Option<CalendarYear>"),
            CONTRACT.replace("year: SourceYear", "year: CalendarYear"),
        ] {
            assert!(!diagnostics(&source).is_empty(), "{source}");
        }
    }

    #[test]
    fn ignores_text_and_declarations_nested_in_other_items() {
        for source in [
            format!("/* {CONTRACT} */"),
            format!("const DECOY: &str = r##\"{CONTRACT}\"##;"),
            format!("mod nested {{ {CONTRACT} }}"),
            format!("decoy!({CONTRACT});"),
            format!("decoy![{CONTRACT}];"),
        ] {
            assert!(!diagnostics(&source).is_empty(), "{source}");
        }
    }

    #[test]
    fn rejects_conditional_or_transforming_attributes_on_contract_items() {
        for source in [
            format!("#![cfg(any())]\n{CONTRACT}"),
            CONTRACT.replace(
                "pub(super) struct SourceYear",
                "#[cfg(any())] pub(super) struct SourceYear",
            ),
            CONTRACT.replace(
                "pub(super) struct ActiveInterval",
                "#[cfg_attr(any(), cfg(any()))] pub(super) struct ActiveInterval",
            ),
            CONTRACT.replace(
                "pub(super) enum DatedFeature",
                "#[custom_transform] pub(super) enum DatedFeature",
            ),
            CONTRACT.replace("Absent,", "#[cfg(any())] Absent,"),
            CONTRACT.replace(
                "from: Option<SourceYear>",
                "#[cfg(any())] from: Option<SourceYear>",
            ),
        ] {
            assert!(!diagnostics(&source).is_empty(), "{source}");
        }
    }
}
