pub fn audit(seeds: u64) -> BTreeMap<TemplateFamily, u64> {
    let mut out = BTreeMap::new();
    for word in 0..seeds {
        let seed = fabelgeist_determinism::Seed::from_u64(word);
        let context = GenerationContext {
            seed,
            observer_entropy_hi: word ^ 0x6f62_7365_7276_6572,
            observer_entropy_lo: fabelgeist_determinism::StreamId::new("quest.fixture-observer-high").seed(seed, &[]).to_u64(),
            settlement_id: "audit".into(),
            settlement_name: "Audit".into(),
            scope: Scope::Settlement {
                settlement_id: "audit".into(),
            },
            ordinal: 0,
            now_minute: adventuresim_world_schema::calendar::StrategicMinute::new(1_000),
            incident_weather: crate::weather::Precipitation::Clear,
            requested_family: None,
            witness_candidates: test_witnesses(),
        };
        if let Ok(case) = generate(&context) {
            *out.entry(case.family).or_default() += 1;
        }
    }
    out
}

pub fn test_witnesses() -> Vec<WitnessCandidate> {
    vec![
        WitnessCandidate {
            resident_character_id: 1,
            display_name: "Anna Weber".into(),
            demographic: WitnessDemographic::Child,
            age_band: "child".into(),
            sex: Some(adventuresim_world_schema::Sex::Female),
            profession: "apprentice".into(),
            visible_description: "a short, fair-haired apprentice".into(),
            expected_location: "residences".into(),
            expected_location_label: "Residences".into(),
            presence_version: 11,
            allowed_circumstances: BTreeSet::from([
                Circumstance::NightWindow,
                Circumstance::AdultVenue,
            ]),
        },
        WitnessCandidate {
            resident_character_id: 2,
            display_name: "Berthold Fischer".into(),
            demographic: WitnessDemographic::Guard,
            age_band: "adult".into(),
            sex: Some(adventuresim_world_schema::Sex::Male),
            profession: "guard".into(),
            visible_description: "a tall guard with dark hair".into(),
            expected_location: "keep".into(),
            expected_location_label: "Keep".into(),
            presence_version: 12,
            allowed_circumstances: BTreeSet::from([
                Circumstance::RoadJourney,
                Circumstance::GraveDuty,
            ]),
        },
        WitnessCandidate {
            resident_character_id: 3,
            display_name: "Clara Hoffmann".into(),
            demographic: WitnessDemographic::Merchant,
            age_band: "elder".into(),
            sex: Some(adventuresim_world_schema::Sex::Female),
            profession: "merchant".into(),
            visible_description: "a broad merchant with grey hair".into(),
            expected_location: "market".into(),
            expected_location_label: "General Market".into(),
            presence_version: 13,
            allowed_circumstances: BTreeSet::from([
                Circumstance::RoadJourney,
                Circumstance::SecretRiversideMeeting,
            ]),
        },
    ]
}
