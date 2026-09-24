use adventuresim_weapon_model::{Detail, generate_model, recipe::Recipe};
use serde_json::{Value, json};

fn model(value: Value) -> Result<adventuresim_weapon_model::GeneratedModel, String> {
    let recipe: Recipe = serde_json::from_value(value).unwrap();
    generate_model(&recipe, Detail::High)
}

#[test]
fn stepped_flange_checks_both_sides_of_each_radial_step() {
    let mut value = json!({"components":[{"kind":"mace","length":0.16,"attach":{"to":"weapon.root","at":"base"},
        "rootRadius":0.014,"shoulderRadius":0.014,"cuspRadius":0.05,
        "flanges":6,"flangeThickness":0.002,"coreProfile":[[-0.08,0.01],[0.08,0.01]],
        "flangeProfile":[{"at":0,"radius":0.014},{"at":0.4,"radius":0.014},
        {"at":0.4,"radius":0.02},{"at":0.8,"radius":0.05},{"at":1,"radius":0.014}]}]});
    assert!(model(value.clone()).is_ok());
    value["components"][0]["flangeProfile"][1]["radius"] = 0.005.into();
    assert!(
        model(value)
            .unwrap_err()
            .contains("core face reaches outside")
    );
}

fn wrapped_grip() -> Value {
    json!({"components":[{"id":"grip","kind":"shaft","role":"Grip","material":"wood","attach":{"to":"weapon.root","at":"base"},
        "length":0.12,"radius":0.014,"bottomScale":1,"topScale":1,"segments":24,
        "wrappings":[
        {"start":0.007,"length":0.11,"pitch":0.003,"width":0.0028,"thickness":0.001,
         "phase":0,"pattern":"rightHanded","material":"cord","section":{"kind":"rounded","crestFraction":0.2}},
        {"start":0.012,"length":0.10,"pitch":0.025,"width":0.003,"thickness":0.001,
         "phase":0,"pattern":"leftHanded","material":"cord","onWrapping":0,
         "section":{"kind":"rounded","crestFraction":0.1}}]}]})
}

#[test]
fn dense_supported_wrapping_has_material_and_rejects_impossible_supports() {
    let value = wrapped_grip();
    let mesh = model(value.clone()).unwrap();
    assert_eq!(mesh.parts.len(), 3);
    assert!(mesh.physical.mass_kg > 0.05);
    for support in [1, 2] {
        let mut invalid = value.clone();
        invalid["components"][0]["wrappings"][1]["onWrapping"] = support.into();
        assert!(model(invalid).is_err());
    }
    for width in [0.003, 0.004] {
        let mut invalid = value.clone();
        invalid["components"][0]["wrappings"][0]["width"] = width.into();
        assert!(model(invalid).is_err());
    }
    let mut unsupported = value;
    let upper = &mut unsupported["components"][0]["wrappings"][1];
    upper["pitch"] = 0.003.into();
    upper["width"] = 0.0001.into();
    upper["pattern"] = "rightHanded".into();
    upper["start"] = 0.007.into();
    upper["phase"] = 0.into();
    assert!(model(unsupported).is_err());
}
