//! Seeded evaluator fixture context and stable witness identities.
use super::*;

pub(super) fn generation_context(seed: u64, family: TemplateFamily) -> qg::GenerationContext {
    let circumstances = BTreeSet::from([
        Circumstance::NightWindow,
        Circumstance::SecretRiversideMeeting,
        Circumstance::AdultVenue,
        Circumstance::RoadJourney,
        Circumstance::GraveDuty,
        Circumstance::LivestockWatch,
    ]);
    let witness = |id: &str, display_name: &str, demographic, description: &str, location: &str| {
        WitnessCandidate {
            resident_character_id: witness_character_id(id),
            display_name: display_name.into(),
            demographic,
            age_band: "adult".into(),
            sex: "unspecified".into(),
            profession: id.into(),
            visible_description: description.into(),
            expected_location: location.into(),
            expected_location_label: location.into(),
            presence_version: 1,
            allowed_circumstances: circumstances.clone(),
        }
    };
    qg::GenerationContext {
        seed,
        observer_entropy_hi: fabelgeist_determinism::StreamId::new("quest.fixture-observer-high")
            .seed(seed, &[])
            .to_u64(),
        observer_entropy_lo: fabelgeist_determinism::StreamId::new("quest.fixture-observer-low")
            .seed(seed, &[])
            .to_u64(),
        settlement_id: "settlement:evaluator".into(),
        settlement_name: "Greifenhagen".into(),
        scope: adventuresim_core::local_problem::Scope::Settlement {
            settlement_id: "settlement:evaluator".into(),
        },
        ordinal: (seed & u64::from(u16::MAX)) as u16,
        now_minute: 100_000,
        incident_weather: adventuresim_core::weather::Precipitation::Clear,
        requested_family: Some(family),
        witness_candidates: vec![
            witness(
                "watchman",
                "Konrad",
                WitnessDemographic::Guard,
                "a tall watchman with cropped fair hair and a scarred chin",
                "the gatehouse",
            ),
            witness(
                "cooper",
                "Marta",
                WitnessDemographic::Laborer,
                "a short cooper with dark curls and a blue apron",
                "the riverside workshop",
            ),
            witness(
                "merchant",
                "Elsbeth",
                WitnessDemographic::Merchant,
                "an elderly merchant in a red wool cap",
                "the market arcade",
            ),
        ],
    }
}

pub(super) fn witness_character_id(witness_key: &str) -> u64 {
    #[derive(serde::Serialize)]
    struct WitnessIdentity<'a> {
        witness_key: &'a str,
    }

    let digest = semantic_digest(
        SemanticDigestPurpose::WitnessIdentity,
        &WitnessIdentity { witness_key },
    )
    .expect("witness identity is serializable");
    let low_bits = u64::from_str_radix(&digest[..16], 16)
        .expect("semantic digests start with sixteen hexadecimal digits");
    low_bits | (1u64 << 63)
}
