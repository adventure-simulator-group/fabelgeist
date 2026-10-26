use super::*;

#[test]
fn starter_personality_uses_the_canonical_serialization_contract() {
    for (presentation, presentation_id) in [
        (Presentation::Man, "man"),
        (Presentation::Ambiguous, "ambiguous"),
        (Presentation::Woman, "woman"),
    ] {
        for (inclination, inclination_id) in [
            (Inclination::Men, "men"),
            (Inclination::Either, "either"),
            (Inclination::Women, "women"),
            (Inclination::Neither, "neither"),
        ] {
            let mut candidate = default_character("canonical-personality-fixture");
            candidate.personality.presentation = presentation;
            candidate.personality.inclination = inclination;
            let encoded = serde_json::to_value(&candidate).unwrap();
            assert!(encoded.get("name").is_none());
            assert_eq!(encoded["personality"]["presentation"], presentation_id);
            assert_eq!(encoded["personality"]["inclination"], inclination_id);
            let decoded: StartingCharacterSpec = serde_json::from_value(encoded).unwrap();
            assert_eq!(decoded, candidate);
            assert_eq!(
                decoded.native_everyday_name(),
                candidate.native_everyday_name()
            );
        }
    }
    assert!(serde_json::from_str::<Presentation>("\"Man\"").is_err());
    assert!(serde_json::from_str::<Inclination>("\"Women\"").is_err());
}
