use super::*;

#[test]
fn mutable_access_distinguishes_future_from_missing_chronology() {
    let observer = PersonalFrontier::new(CharacterId::from(7), Some(StrategicMinute::new(60)));
    let subject_id = CharacterId::from(8);
    for (minute, expected) in [
        (
            Some(StrategicMinute::new(59)),
            MutableCharacterAccess::Available,
        ),
        (
            Some(StrategicMinute::new(60)),
            MutableCharacterAccess::Available,
        ),
        (
            Some(StrategicMinute::new(61)),
            MutableCharacterAccess::Future,
        ),
        (None, MutableCharacterAccess::Unknown),
    ] {
        let subject = PersonalFrontier::new(subject_id, minute);
        assert_eq!(subject.mutable_access_for(&observer), expected);
    }
    let unknown_observer = PersonalFrontier::new(CharacterId::from(7), None);
    let known_subject = PersonalFrontier::new(subject_id, Some(StrategicMinute::ZERO));
    assert_eq!(
        known_subject.mutable_access_for(&unknown_observer),
        MutableCharacterAccess::Unknown
    );
    assert_eq!(
        unknown_observer.mutable_access_for(&unknown_observer),
        MutableCharacterAccess::Available
    );
}

#[test]
fn alignment_requires_two_known_equal_frontiers_even_for_the_same_identity() {
    let identity = CharacterId::from(7);
    let known = PersonalFrontier::new(identity, Some(StrategicMinute::ZERO));
    let unknown = PersonalFrontier::new(identity, None);
    let later = PersonalFrontier::new(identity, Some(StrategicMinute::new(1)));
    assert_eq!(known.alignment_with(&known), FrontierAlignment::Aligned);
    assert_eq!(known.alignment_with(&later), FrontierAlignment::Different);
    assert_eq!(known.alignment_with(&unknown), FrontierAlignment::Unknown);
    assert_eq!(unknown.alignment_with(&unknown), FrontierAlignment::Unknown);
}

#[test]
fn death_becomes_known_at_its_effective_minute_without_disclosing_future_death() {
    let observer = Some(StrategicMinute::new(60));
    for (death, expected) in [
        (Some(StrategicMinute::new(59)), ObservedLife::Dead),
        (Some(StrategicMinute::new(60)), ObservedLife::Dead),
        (Some(StrategicMinute::new(61)), ObservedLife::Alive),
        (None, ObservedLife::Alive),
    ] {
        assert_eq!(ObservedLife::at_date(observer, death), expected);
    }
    assert_eq!(
        ObservedLife::at_date(None, Some(StrategicMinute::ZERO)),
        ObservedLife::Alive
    );
}

#[test]
fn corpses_do_not_participate_in_party_readiness() {
    for (alive, expected) in [(true, ObservedLife::Alive), (false, ObservedLife::Dead)] {
        let member = crate::spacetimedb::CharacterView {
            id: 7,
            name: "member".into(),
            xp: 0,
            level: 1,
            current_settlement_id: None,
            current_case_site_id: None,
            party_id: None,
            age_years: 20,
            alive,
            temporary: false,
            social_notification_count: 0,
            automatic_social_chat_enabled: false,
        };
        assert_eq!(ObservedLife::from_character(&member), expected);
    }
}
