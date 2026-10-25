//! Canonical schema, attachment and bounded-input contracts at public entrypoints.
use adventuresim_weapon_model::*;
use serde_json::{Value, json};

fn design(value: Value) -> WeaponDesign {
    serde_json::from_value(json!({"catalog_id":"fixture","recipe":value})).unwrap()
}
fn component(id: &str, parent: &str) -> Value {
    json!({"kind":"shaft","id":id,"length":0.2,"radius":0.01,"attach":{"to":parent}})
}

#[test]
fn invalid_attachment_graphs_fail_generation_and_transport() {
    for components in [
        vec![component("head", "missing.top")],
        vec![
            component("same", "weapon.root"),
            component("same", "weapon.root"),
        ],
        vec![component("a", "b.top"), component("b", "a.top")],
    ] {
        let design = design(json!({"components":components}));
        assert!(validate(&design).is_err());
        assert!(generate(&design).is_err());
        assert!(encode(&design).is_err());
    }
    let reserved = design(
        json!({"shaft":{"length":1.0,"radius":0.01},"components":[component("shaft","weapon.root")]}),
    );
    assert!(generate(&reserved).is_err());
}

#[test]
fn grip_shapes_enforce_anatomical_caps_at_the_authoritative_boundary() {
    for (shape, field, valid, invalid) in [
        (
            json!({"kind":"grip","length":0.12,"radius":0.022}),
            "radius",
            0.022,
            0.023,
        ),
        (
            json!({"kind":"ovalGrip","length":0.12,"width":0.038,"thickness":0.024}),
            "width",
            0.038,
            0.039,
        ),
        (
            json!({"kind":"ovalGrip","length":0.12,"width":0.038,"thickness":0.028}),
            "thickness",
            0.028,
            0.029,
        ),
    ] {
        let mut shape = shape;
        shape["attach"] = json!({"to":"weapon.root"});
        shape[field] = json!(valid);
        assert!(generate(&design(json!({"components":[shape.clone()]}))).is_ok());
        shape[field] = json!(invalid);
        assert!(generate(&design(json!({"components":[shape]}))).is_err());
    }
}

#[test]
fn hostile_numbers_and_unknown_fields_never_reach_geometry() {
    for value in [json!(-1), json!(0), json!(1e30)] {
        let mut shape = component("grip", "weapon.root");
        shape["length"] = value;
        let design = design(json!({"components":[shape]}));
        assert!(
            std::panic::catch_unwind(|| generate(&design))
                .unwrap()
                .is_err()
        );
    }
    for value in [json!(0), json!(65535)] {
        let mut shape = component("grip", "weapon.root");
        shape["segments"] = value;
        let design = design(json!({"components":[shape]}));
        assert!(
            std::panic::catch_unwind(|| generate(&design))
                .unwrap()
                .is_err()
        );
    }
    let mut shape = component("grip", "weapon.root");
    shape["unknown"] = json!(1);
    assert!(serde_json::from_value::<recipe::Recipe>(json!({"components":[shape]})).is_err());
}

#[test]
fn native_numeric_editor_paths_are_live_and_cover_every_chassis() {
    let mut accepted_changes = 0;
    for id in MELEE_CATALOG_IDS {
        let source = default_design(id).unwrap();
        let fields = numeric_editor_fields(&source);
        assert!(!fields.is_empty(), "{id}");
        let original = serde_json::to_value(&source).unwrap();
        for field in fields {
            let pointer = format!("/{}", field.path.replace('.', "/"));
            let current = original.pointer(&pointer).unwrap().as_f64().unwrap();
            assert!(
                current >= field.min && current <= field.max,
                "{id} {}: {current} outside {}..{}",
                field.path,
                field.min,
                field.max
            );
            let mut edited = original.clone();
            let next = (current + field.step).min(field.max);
            *edited.pointer_mut(&pointer).unwrap() = json!(next);
            let Ok(edited) = serde_json::from_value::<WeaponDesign>(edited) else {
                continue;
            };
            if let Ok(model) = generate(&edited) {
                accepted_changes += 1;
                assert!(model.derived.mass_kg > 0.0);
                assert!(
                    model
                        .parts
                        .iter()
                        .flat_map(|p| p.positions.iter().flatten())
                        .all(|n| n.is_finite())
                );
            }
        }
    }
    assert!(accepted_changes > 100);
}

#[test]
fn unusual_polearm_flanged_mace_uses_the_shared_composer_and_attachment_graph() {
    let catalog = authoring::authoring_catalog();
    for haft in catalog["hafts"].as_array().unwrap() {
        for head in catalog["heads"].as_array().unwrap() {
            let recipe = authoring::compose_weapon(
                haft["id"].as_str().unwrap(),
                head["id"].as_str().unwrap(),
            )
            .unwrap();
            let model = generate_model(&recipe, Detail::High).unwrap();
            assert!(model.physical.mass_kg > 0.0);
            assert!(model.parts.len() > 2);
        }
    }
}
