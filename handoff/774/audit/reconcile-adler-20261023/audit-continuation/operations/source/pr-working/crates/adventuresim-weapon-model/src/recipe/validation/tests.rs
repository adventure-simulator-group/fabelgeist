use super::*;
fn component(kind: &str) -> serde_json::Value {
    let catalog: serde_json::Value =
        serde_json::from_str(include_str!("../../../catalog/authoring.json")).unwrap();
    catalog["presets"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["definition"]["components"].as_array().unwrap())
        .find(|p| p["kind"] == kind)
        .unwrap()
        .clone()
}
#[test]
fn malformed_mechanical_clearances_are_rejected_before_generation() {
    let cases = [
        ("arrow", "nockSlotWidth", 0.0001),
        ("arrow", "maximumStringRadius", 0.01),
        ("arrow", "nockClearance", 0.02),
        ("crossbow", "grooveWidth", 0.0001),
        ("crossbow", "nutWidth", 0.09),
        ("crossbow", "railHeight", 0.03),
        ("crossbowBolt", "buttWidth", 0.001),
        ("boltQuiver", "strapDrop", 0.001),
        ("boltQuiver", "wall", 0.015),
        ("firearm", "bore", 0.003),
        ("firearm", "barrelWall", 0.0001),
        ("firearm", "lockPosition", 0.01),
        ("ballPouch", "wall", 0.02),
        ("ballPouch", "flapAngle", 150.0),
    ];
    for (kind, field, value) in cases {
        let mut json = component(kind);
        json[field] = serde_json::json!(value);
        let recipe: Recipe =
            serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
        assert!(recipe.validate().is_err(), "{kind}.{field}");
        assert!(
            crate::generate_model(&recipe, crate::Detail::Low).is_err(),
            "{kind}.{field}"
        );
    }
}
#[test]
fn arrow_declared_string_clearance_has_a_supported_boundary() {
    let mut json = component("arrow");
    let string = json["maximumStringRadius"].as_f64().unwrap();
    let gap = json["nockClearance"].as_f64().unwrap();
    json["nockSlotWidth"] = serde_json::json!(string * 2.0 + gap);
    let recipe: Recipe = serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
    recipe.validate().unwrap();
    crate::generate_model(&recipe, crate::Detail::Low).unwrap();
}
#[test]
fn impossible_dimensions_and_sampling_requests_are_rejected() {
    let mut json = component("arrow");
    json["length"] = serde_json::json!(-1.0);
    let recipe: Recipe = serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
    assert_eq!(recipe.validate(), Err(RecipeError::Dimension));
    let mut json = component("arrow");
    json["segments"] = serde_json::json!(65535);
    let recipe: Recipe = serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
    assert_eq!(recipe.validate(), Err(RecipeError::Budget));
}

#[test]
fn minimum_sampling_counts_and_positive_blade_sections_are_enforced() {
    for (kind, field) in [
        ("leadBall", "segments"),
        ("archeryBow", "samples"),
        ("firearm", "bandCount"),
    ] {
        let mut json = component(kind);
        json[field] = serde_json::json!(0);
        let recipe: Recipe =
            serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
        assert!(recipe.validate().is_err(), "{kind}.{field}");
    }
    let mut json = serde_json::json!({"kind":"diamondBlade", "id":"blade", "attach":{"to":"weapon.root","at":"origin"}, "length":0.3, "width":0.03, "thickness":0.004});
    for (taper, valid) in [(0.99, true), (1.0, false), (2.0, false)] {
        json["taper"] = serde_json::json!(taper);
        let recipe: Recipe =
            serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
        assert_eq!(recipe.validate().is_ok(), valid, "taper {taper}");
    }
}

#[test]
fn shield_sampling_budget_is_checked_before_surface_allocation() {
    let mut round = component("roundShield");
    round["rings"] = serde_json::json!(256);
    round["radialSegments"] = serde_json::json!(256);
    let mut shaped = component("shapedShield");
    shaped["centerCurve"] = serde_json::json!(20);
    shaped["centerWidth"] = serde_json::json!(0.00375);
    for json in [round, shaped] {
        let recipe: Recipe =
            serde_json::from_value(serde_json::json!({"components":[json]})).unwrap();
        assert_eq!(recipe.validate(), Err(RecipeError::Budget));
    }
}
