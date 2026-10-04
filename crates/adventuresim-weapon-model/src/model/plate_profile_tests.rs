use super::contoured_plate_tests::closed;
use super::*;

fn plate() -> serde_json::Value {
    let profile = |middle: f64| {
        serde_json::json!([
            {"across":-1,"thickness":0.002}, {"across":-0.5,"thickness":0.002},
            {"across":-0.25,"thickness":0.010}, {"across":middle,"thickness":0.003},
            {"across":0.25,"thickness":0.008}, {"across":0.5,"thickness":0.002},
            {"across":1,"thickness":0.002}
        ])
    };
    serde_json::json!({"kind":"contouredPlate","width":0.08,"length":0.12,"start":[-0.5,0],
        "boundary":[{"kind":"line","to":[0.5,0]},{"kind":"line","to":[0.5,1]},
            {"kind":"line","to":[-0.5,1]},{"kind":"line","to":[-0.5,0]}],
        "surface":{"kind":"profile","stations":[{"at":0,"profile":profile(-0.05)},
            {"at":1,"profile":profile(0.05)}]}})
}

fn parameters(value: serde_json::Value) -> ContouredPlateParameters {
    let Shape::ContouredPlate(p) = serde_json::from_value(value).unwrap() else {
        panic!("expected a plate")
    };
    p
}

#[test]
fn moving_multiple_ridges_preserve_integrated_material_and_export() {
    let p = parameters(plate());
    // The two end profiles integrate to the same mean trapezoid area;
    // moving the valley linearly makes that area affine along the blank.
    let expected = 0.12
        * 0.08
        * 0.25
        * ((0.002 + 0.010) / 2.0
            + (0.010 + 0.003) / 2.0
            + (0.003 + 0.008) / 2.0
            + (0.008 + 0.002) / 2.0);
    for detail in [Detail::Low, Detail::Medium, Detail::High] {
        let solid = contoured_plate::construct(&p, detail).unwrap();
        closed(&solid);
        let error_bound = 0.08 * 0.12 * 2.0 * detail.error(0.00005);
        assert!((solid.volume() - expected).abs() <= error_bound);
    }
}

#[test]
fn profile_rejects_inverted_tracks_uncovered_width_and_missing_landmarks() {
    for mutation in 0..4 {
        let mut value = plate();
        let profile = &mut value["surface"]["stations"][1]["profile"];
        match mutation {
            0 => profile[3]["across"] = 0.3.into(),
            1 => profile[0]["across"] = (-0.4).into(),
            2 => {
                profile.as_array_mut().unwrap().pop();
            }
            _ => profile[2]["thickness"] = (-0.001).into(),
        }
        let recipe: Recipe =
            serde_json::from_value(serde_json::json!({"components":[value]})).unwrap();
        assert!(recipe.validate().is_err());
    }
}

#[test]
fn profile_rejects_zero_thickness_inside_occupied_material() {
    let mut value = plate();
    for station in value["surface"]["stations"].as_array_mut().unwrap() {
        station["profile"][3]["thickness"] = 0.into();
    }
    assert!(
        contoured_plate::construct(&parameters(value), Detail::High)
            .unwrap_err()
            .contains("isolated authored boundary apices")
    );
}
