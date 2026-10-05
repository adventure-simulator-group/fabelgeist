use super::*;

#[test]
fn kassel_18_rounded_bearing_is_valid_but_a_real_concavity_is_rejected() {
    let value: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/kassel-18-bearing-outline.json"
    )))
    .unwrap();
    let original: Vec<bevy::math::DVec2> =
        serde_json::from_value(value["outline"].clone()).unwrap();
    let edge = original[10] - original[9];
    let observed = edge.perp_dot(original[8] - original[9]) / edge.length();
    assert!(observed < 0.0 && observed > -0.00002);
    assert!(valid_outline(&original));
    let mut notched = original.clone();
    let centre = original.iter().copied().sum::<bevy::math::DVec2>() / original.len() as f64;
    notched[8] += (centre - original[8]).normalize() * 0.02;
    assert!(!valid_outline(&notched));
}

#[test]
fn accepted_surfaces_require_selected_treatment_and_exact_member_arity() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/tactical-grounding/carpenter-passage-access.json"
    )))
    .unwrap();
    let original: PropertySupportSurface =
        serde_json::from_value(fixture["surface"].clone()).unwrap();
    assert_eq!(original.validate_encoded(), Ok(()));
    let mut unselected = original.clone();
    unselected.treatment = SupportGradingAttempt::NotSelected;
    assert_eq!(
        unselected.validate_encoded(),
        Err(SupportSurfaceIssue::Treatment)
    );
    let mut extra_member = original.clone();
    extra_member.mesh.member_building_ids.push(10);
    assert_eq!(
        extra_member.validate_encoded(),
        Err(SupportSurfaceIssue::Treatment)
    );
    let mut incorrect_compound = original;
    incorrect_compound.treatment = SupportGradingAttempt::Compound(CourtTreatment::Level);
    assert_eq!(
        incorrect_compound.validate_encoded(),
        Err(SupportSurfaceIssue::Treatment)
    );
}
