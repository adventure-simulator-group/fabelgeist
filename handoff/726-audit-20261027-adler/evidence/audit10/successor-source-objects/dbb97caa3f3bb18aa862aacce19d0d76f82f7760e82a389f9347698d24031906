//! Strict conversion of SQL wire values into generated SATS rows.
use super::{Result, SpacetimeError};
use crate::spacetimedb::types::{AlgebraicType, QueryResponse};
use serde_json::Value;
use spacetimedb_sats::{de::DeserializeOwned as SatsDeserializeOwned, serde::SerdeWrapper};
fn sum_variants(algebraic_type: &AlgebraicType) -> Option<&Vec<Value>> {
    let AlgebraicType::Value(ty) = algebraic_type;
    ty.get("Sum")?.get("variants")?.as_array()
}

fn variant_name(variant: &Value) -> Option<&str> {
    variant.get("name")?.get("some")?.as_str()
}

fn is_option_sum(variants: &[Value]) -> bool {
    if variants.len() != 2 {
        return false;
    }

    let names: Vec<_> = variants
        .iter()
        .filter_map(variant_name)
        .map(|s| s.to_ascii_lowercase())
        .collect();

    names.iter().any(|n| n == "some") && names.iter().any(|n| n == "none")
}

fn product_elements(algebraic_type: &AlgebraicType) -> Option<&Vec<Value>> {
    let AlgebraicType::Value(ty) = algebraic_type;
    ty.get("Product")?.get("elements")?.as_array()
}

fn array_element_type(algebraic_type: &AlgebraicType) -> Option<&Value> {
    let AlgebraicType::Value(ty) = algebraic_type;
    ty.get("Array")
}

fn serde_variant_name(variant: &Value) -> String {
    let name = variant_name(variant).unwrap_or("Unknown");
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => "Unknown".into(),
    }
}

fn is_identity_product(elements: &[Value]) -> bool {
    elements.len() == 1
        && elements[0]
            .get("name")
            .and_then(|name| name.get("some"))
            .and_then(Value::as_str)
            == Some("__identity__")
}

fn malformed_sats_value(message: impl Into<String>) -> SpacetimeError {
    SpacetimeError::Spacetime(format!(
        "malformed SpacetimeDB SQL value: {}",
        message.into()
    ))
}

/// Convert the SQL endpoint's positional wire values to the human-readable
/// representation accepted by SATS' serde bridge. Unlike the presentation
/// conversion above, sums remain explicit one-key objects so their tags never
/// depend on serde enum conventions.
fn convert_spacetime_value_sats(value: &Value, algebraic_type: &AlgebraicType) -> Result<Value> {
    if let Some(element_type) = array_element_type(algebraic_type) {
        // SATS serializes byte arrays as an unprefixed hexadecimal string.
        // Other arrays retain their positional JSON array representation.
        if element_type.get("U8").is_some() {
            let bytes = value
                .as_str()
                .ok_or_else(|| malformed_sats_value("expected hexadecimal bytes"))?;
            if bytes.len() % 2 != 0 || !bytes.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(malformed_sats_value("invalid hexadecimal bytes"));
            }
            return Ok(value.clone());
        }
        let values = value
            .as_array()
            .ok_or_else(|| malformed_sats_value("expected an array"))?;
        let element_type = AlgebraicType::Value(element_type.clone());
        return values
            .iter()
            .map(|value| convert_spacetime_value_sats(value, &element_type))
            .collect::<Result<Vec<_>>>()
            .map(Value::Array);
    }

    if let Some(elements) = product_elements(algebraic_type) {
        return convert_product_sats(value, elements);
    }

    let Some(variants) = sum_variants(algebraic_type) else {
        return Ok(value.clone());
    };
    let encoded = value
        .as_array()
        .ok_or_else(|| malformed_sats_value("expected a tagged sum array"))?;
    if encoded.len() != 2 {
        return Err(malformed_sats_value(format!(
            "sum expected a tag and payload but received {} values",
            encoded.len()
        )));
    }
    let tag = encoded[0]
        .as_u64()
        .ok_or_else(|| malformed_sats_value("sum tag was not an unsigned integer"))?;
    let variant = variants
        .get(tag as usize)
        .ok_or_else(|| malformed_sats_value(format!("unknown sum tag {tag}")))?;
    let name = if is_option_sum(variants) {
        variant_name(variant)
            .map(str::to_ascii_lowercase)
            .ok_or_else(|| malformed_sats_value("option variant had no name"))?
    } else {
        serde_variant_name(variant)
    };
    let payload_type = AlgebraicType::Value(
        variant
            .get("algebraic_type")
            .cloned()
            .unwrap_or(Value::Null),
    );
    let payload = convert_spacetime_value_sats(&encoded[1], &payload_type)?;
    Ok(Value::Object([(name, payload)].into_iter().collect()))
}

pub(super) fn decode_sats_query_response<T: SatsDeserializeOwned>(
    query_response: &QueryResponse,
) -> Result<Vec<T>> {
    let Some(first) = query_response.first() else {
        return Ok(Vec::new());
    };
    if first
        .schema
        .elements
        .iter()
        .any(|element| element.name.is_none())
    {
        return Err(malformed_sats_value(
            "generated-row query returned an unnamed column",
        ));
    }

    first
        .rows
        .iter()
        .map(|row| {
            let values = row
                .as_array()
                .ok_or_else(|| malformed_sats_value("SpacetimeDB returned a non-array SQL row"))?;
            if values.len() != first.schema.elements.len() {
                return Err(malformed_sats_value(format!(
                    "row expected {} columns but received {}",
                    first.schema.elements.len(),
                    values.len()
                )));
            }
            let mut object = serde_json::Map::new();
            for (element, value) in first.schema.elements.iter().zip(values) {
                let name = element
                    .name
                    .as_ref()
                    .expect("all SQL columns were checked as named");
                object.insert(
                    name.some.clone(),
                    convert_spacetime_value_sats(value, &element.algebraic_type)?,
                );
            }
            serde_json::from_value::<SerdeWrapper<T>>(Value::Object(object))
                .map(|wrapped| wrapped.0)
                .map_err(Into::into)
        })
        .collect()
}

fn convert_product_sats(value: &Value, elements: &[Value]) -> Result<Value> {
    let values = value
        .as_array()
        .ok_or_else(|| malformed_sats_value("expected a product array"))?;
    if values.len() != elements.len() {
        return Err(malformed_sats_value(format!(
            "product expected {} fields but received {}",
            elements.len(),
            values.len()
        )));
    }

    if is_identity_product(elements) {
        let raw = values
            .first()
            .and_then(Value::as_str)
            .ok_or_else(|| malformed_sats_value("identity was not a hexadecimal string"))?;
        let digits = raw.strip_prefix("0x").unwrap_or(raw);
        if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(malformed_sats_value(
                "identity contained invalid hexadecimal digits",
            ));
        }
        let significant = digits.trim_start_matches('0');
        let quantity = if significant.is_empty() {
            "0x0".to_owned()
        } else {
            format!("0x{significant}")
        };
        return Ok(Value::Array(vec![Value::String(quantity)]));
    }

    if elements.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }

    if elements.iter().all(|element| {
        element
            .get("name")
            .and_then(|name| name.get("some"))
            .and_then(Value::as_str)
            .is_some()
    }) {
        let mut object = serde_json::Map::new();
        for (element, value) in elements.iter().zip(values) {
            let name = element
                .get("name")
                .and_then(|name| name.get("some"))
                .and_then(Value::as_str)
                .expect("all product elements were checked as named");
            let nested_type = AlgebraicType::Value(
                element
                    .get("algebraic_type")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            object.insert(
                name.to_owned(),
                convert_spacetime_value_sats(value, &nested_type)?,
            );
        }
        return Ok(Value::Object(object));
    }

    let values = elements
        .iter()
        .zip(values)
        .map(|(element, value)| {
            let nested_type = AlgebraicType::Value(
                element
                    .get("algebraic_type")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            convert_spacetime_value_sats(value, &nested_type)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::Array(values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn sats_sql_hex_bytes_decode_generated_weapon_and_holder_rows() {
        let fixture = query_fixture(
            &[
                ("physical_object_id", json!({"U64":[]})),
                ("generator_version", json!({"U16":[]})),
                ("design_hash", json!({"Array":{"U8":[]}})),
                ("recipe", json!({"Array":{"U8":[]}})),
                ("mass_grams", json!({"U32":[]})),
                ("length_mm", json!({"U32":[]})),
                ("grip_to_tip_mm", json!({"U32":[]})),
            ],
            json!([170, 9, "00".repeat(32), "007bff80", 1175, 1225, 1060]),
        );
        let weapons =
            decode_sats_query_response::<adventuresim_stdb_client::WeaponInstance>(&fixture)
                .unwrap();
        let holders =
            decode_sats_query_response::<adventuresim_stdb_client::WeaponHolderInstance>(&fixture)
                .unwrap();
        assert_eq!(weapons[0].recipe, vec![0, 123, 255, 128]);
        assert_eq!(weapons[0].design_hash, vec![0; 32]);
        assert_eq!(weapons[0].physical_object_id, 170);
        assert_eq!(holders[0].recipe, weapons[0].recipe);
        assert_eq!(holders[0].design_hash, weapons[0].design_hash);
    }
    #[test]
    fn sats_sql_byte_arrays_require_complete_hexadecimal_octets() {
        let ty = AlgebraicType::Value(json!({"Array":{"U8":[]}}));
        for value in [
            json!("0"),
            json!("0g"),
            json!("0x01"),
            json!([0, 1]),
            json!(null),
        ] {
            assert!(convert_spacetime_value_sats(&value, &ty).is_err());
        }
        for bytes in ["", "00", "01aBff"] {
            let converted = convert_spacetime_value_sats(&json!(bytes), &ty).unwrap();
            let decoded = serde_json::from_value::<SerdeWrapper<Vec<u8>>>(converted)
                .unwrap()
                .0;
            assert_eq!(decoded.len(), bytes.len() / 2);
        }
    }
    use adventuresim_stdb_client::{
        BackendForageReceipt, CaseStatus, InvestigationActionAvailability,
        InvestigationActionUnavailableReason, SettlementAlias, StrategicGatewayAuthority,
    };

    fn query_fixture(columns: &[(&str, Value)], row: Value) -> QueryResponse {
        vec![crate::spacetimedb::types::QueryResult {
            schema: crate::spacetimedb::types::QuerySchema {
                elements: columns
                    .iter()
                    .map(
                        |(name, algebraic_type)| crate::spacetimedb::types::SchemaElement {
                            name: Some(crate::spacetimedb::types::AlgebraicTypeRef {
                                some: (*name).to_owned(),
                            }),
                            algebraic_type: AlgebraicType::Value(algebraic_type.clone()),
                        },
                    )
                    .collect(),
            },
            rows: vec![row],
        }]
    }

    #[test]
    fn sats_conversion_preserves_named_products_options_and_one_key_sums() {
        let option_string = json!({ "Sum": { "variants": [
            {
                "name": { "some": "some" },
                "algebraic_type": { "String": [] }
            },
            {
                "name": { "some": "none" },
                "algebraic_type": { "Product": { "elements": [] } }
            }
        ] } });
        let ty = AlgebraicType::Value(json!({
            "Product": { "elements": [
                { "name": { "some": "id" }, "algebraic_type": { "String": [] } },
                { "name": { "some": "settlement_id" }, "algebraic_type": { "String": [] } },
                { "name": { "some": "name" }, "algebraic_type": { "String": [] } },
                { "name": { "some": "prefix" }, "algebraic_type": option_string },
                { "name": { "some": "language" }, "algebraic_type": option_string }
            ] }
        }));
        let converted = convert_spacetime_value_sats(
            &json!(["alias-1", "settlement-1", "Harbour", [0, "Old"], [1, []]]),
            &ty,
        )
        .unwrap();
        assert_eq!(
            converted,
            json!({
                "id": "alias-1",
                "settlement_id": "settlement-1",
                "name": "Harbour",
                "prefix": { "some": "Old" },
                "language": { "none": [] }
            })
        );
        let decoded = serde_json::from_value::<SerdeWrapper<SettlementAlias>>(converted)
            .unwrap()
            .0;
        assert_eq!(decoded.prefix.as_deref(), Some("Old"));
        assert_eq!(decoded.language, None);

        let sum = AlgebraicType::Value(json!({ "Sum": { "variants": [
            {
                "name": { "some": "Open" },
                "algebraic_type": { "Product": { "elements": [] } }
            },
            {
                "name": { "some": "Resolved" },
                "algebraic_type": { "Product": { "elements": [] } }
            },
            {
                "name": { "some": "Failed" },
                "algebraic_type": { "Product": { "elements": [] } }
            }
        ] } }));
        let converted = convert_spacetime_value_sats(&json!([1, []]), &sum).unwrap();
        assert_eq!(converted, json!({ "Resolved": [] }));
        assert_eq!(
            serde_json::from_value::<SerdeWrapper<CaseStatus>>(converted)
                .unwrap()
                .0,
            CaseStatus::Resolved
        );
    }

    #[test]
    fn sats_generated_products_reject_missing_and_unknown_fields() {
        let complete = json!({
            "id": "alias-1",
            "settlement_id": "settlement-1",
            "name": "Harbour",
            "prefix": { "none": [] },
            "language": { "none": [] }
        });
        let mut missing = complete.clone();
        missing.as_object_mut().unwrap().remove("name");
        assert!(serde_json::from_value::<SerdeWrapper<SettlementAlias>>(missing).is_err());

        let mut unknown = complete;
        unknown["display_name"] = json!("Harbour");
        assert!(serde_json::from_value::<SerdeWrapper<SettlementAlias>>(unknown).is_err());
    }

    #[test]
    fn sats_sql_fixture_decodes_generated_identity_row() {
        let option_string = json!({ "Sum": { "variants": [
            {
                "name": { "some": "some" },
                "algebraic_type": { "String": [] }
            },
            {
                "name": { "some": "none" },
                "algebraic_type": { "Product": { "elements": [] } }
            }
        ] } });
        let identity = json!({ "Product": { "elements": [{
            "name": { "some": "__identity__" },
            "algebraic_type": { "U256": [] }
        }] } });
        let response = query_fixture(
            &[
                ("id", json!({ "U8": [] })),
                ("identity", identity),
                ("terrain_package_digest", option_string),
                ("terrain_schema", json!({ "U32": [] })),
            ],
            json!([
                1,
                ["0000000000000000000000000000000000000000000000000000000000000000"],
                [1, []],
                3
            ]),
        );
        let decoded = decode_sats_query_response::<StrategicGatewayAuthority>(&response).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(
            decoded[0].identity,
            adventuresim_stdb_client::spacetimedb_sdk::Identity::ZERO
        );
        assert_eq!(decoded[0].terrain_package_digest, None);
    }

    #[test]
    fn sats_sql_fixture_decodes_generated_array_row() {
        let response = query_fixture(
            &[
                ("character_id", json!({ "U64": [] })),
                ("request_id", json!({ "String": [] })),
                ("elapsed_minutes", json!({ "U64": [] })),
                ("yielded_item_ids", json!({ "Array": { "String": [] } })),
                ("yielded_quantities", json!({ "Array": { "U16": [] } })),
                ("interrupted", json!({ "Bool": [] })),
                ("legal_outcome", json!({ "String": [] })),
            ],
            json!([
                7,
                "forage:7:1",
                45,
                ["yarrow", "nettle"],
                [2, 1],
                false,
                "lawful"
            ]),
        );
        let decoded = decode_sats_query_response::<BackendForageReceipt>(&response).unwrap();
        assert_eq!(decoded[0].yielded_item_ids, ["yarrow", "nettle"]);
        assert_eq!(decoded[0].yielded_quantities, [2, 1]);
    }

    #[test]
    fn sats_fixture_decodes_generated_data_carrying_sum() {
        let reason = json!({ "Sum": { "variants": [
            { "name": { "some": "PartyNotReady" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "TravelRequired" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "NightWindow" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "TargetChanged" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "ContactScheduleWindow" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "ContactNotPresent" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "CharacterUnavailable" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "PartyRequired" }, "algebraic_type": { "Product": { "elements": [] } } }
        ] } });
        let unavailable = json!({ "Product": { "elements": [
            { "name": { "some": "reason" }, "algebraic_type": reason },
            { "name": { "some": "can_travel_to_required_site" }, "algebraic_type": { "Bool": [] } },
            { "name": { "some": "wait_minutes" }, "algebraic_type": { "U32": [] } }
        ] } });
        let availability = AlgebraicType::Value(json!({ "Sum": { "variants": [
            { "name": { "some": "Available" }, "algebraic_type": { "Product": { "elements": [] } } },
            { "name": { "some": "Unavailable" }, "algebraic_type": unavailable }
        ] } }));
        let converted =
            convert_spacetime_value_sats(&json!([1, [[1, []], true, 45]]), &availability).unwrap();
        let decoded =
            serde_json::from_value::<SerdeWrapper<InvestigationActionAvailability>>(converted)
                .unwrap()
                .0;
        let InvestigationActionAvailability::Unavailable(details) = decoded else {
            panic!("expected data-carrying unavailable variant");
        };
        assert_eq!(
            details.reason,
            InvestigationActionUnavailableReason::TravelRequired
        );
        assert!(details.can_travel_to_required_site);
        assert_eq!(details.wait_minutes, 45);
    }

    #[test]
    fn sats_conversion_rejects_malformed_products_and_variants() {
        let product = AlgebraicType::Value(json!({
            "Product": { "elements": [
                { "name": { "some": "value" }, "algebraic_type": { "U64": [] } }
            ] }
        }));
        assert!(convert_spacetime_value_sats(&json!([]), &product).is_err());

        let sum = AlgebraicType::Value(json!({ "Sum": { "variants": [
            {
                "name": { "some": "Only" },
                "algebraic_type": { "Product": { "elements": [] } }
            }
        ] } }));
        assert!(convert_spacetime_value_sats(&json!([1, []]), &sum).is_err());
        assert!(convert_spacetime_value_sats(&json!([0]), &sum).is_err());
    }
}
