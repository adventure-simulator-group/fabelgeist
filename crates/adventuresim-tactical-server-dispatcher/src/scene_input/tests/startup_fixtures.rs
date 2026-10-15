//! Native, lossless inputs for client initialization and destination benchmarks.
use super::*;
use adventuresim_core::strategic_place::{StrategicIdentityComponent, StrategicPlaceId};
use adventuresim_tactical_core::{
    regional_city::RegionalCityInput,
    regional_environment::RegionalEnvironment,
    regional_terrain::{RegionalTerrainRequest, RegionalTerrainScale},
};
use adventuresim_world_schema::{
    SettlementEconomyProfile, calendar::StrategicMinute, coordinates::Wgs84CoordinateMicrodegrees,
    source_package::SourcePackageDigest,
};

#[test]
#[ignore = "exports client fixtures to STRATEGIC_STARTUP_FIXTURE_DIR"]
fn export_distinct_startup_destinations() {
    let output = PathBuf::from(std::env::var_os("STRATEGIC_STARTUP_FIXTURE_DIR").unwrap());
    fs::create_dir_all(&output).unwrap();
    let population = std::env::var("STRATEGIC_STARTUP_FIXTURE_POPULATION")
        .map_or(900, |value| value.parse().unwrap());
    let (pack, temporary) = constant_final_pack();
    let source = SourcePackageDigest::from_hex(pack.digest()).unwrap();
    let mut first_programs = None;
    for (ordinal, (name, settlement_id, latitude)) in [
        ("a", "relative-city", 505_000_000),
        ("b", "relative-city-destination", 505_300_000),
    ]
    .into_iter()
    .enumerate()
    {
        let origin = Wgs84CoordinateE7::new(latitude, 105_000_000).unwrap();
        let mut settlement = SettlementSceneProfile {
            id: settlement_id.into(),
            population_level: 1,
            population_estimate: population,
            economy: SettlementEconomyProfile::stage_placeholder(),
            operators: Vec::new(),
        };
        add_fixture_operators(&mut settlement);
        for operator in &mut settlement.operators {
            operator.operator_character_id.0 += ordinal as u64 * 10_000;
        }
        let input = build_imported_scene(
            &pack,
            "startup-benchmark",
            "city",
            latitude,
            105_000_000,
            StrategicMinute::new(123_456),
            StrategicMinute::new(123_456),
            Some(&settlement),
        )
        .unwrap();
        let programs = input
            .physical_placements()
            .into_iter()
            .map(|placement| placement.program)
            .collect::<Vec<_>>();
        if let Some(first) = &first_programs {
            assert_ne!(
                first, &programs,
                "An unseen settlement has different building programs"
            );
        } else {
            first_programs = Some(programs);
        }
        let document = RegionalCityInput::from_scene(
            source.clone(),
            StrategicPlaceId::Settlement {
                settlement_id: StrategicIdentityComponent::try_new(settlement.id.clone()).unwrap(),
            },
            origin,
            input.clone(),
        )
        .unwrap();
        let environments = [
            RegionalTerrainScale::Neighborhood,
            RegionalTerrainScale::District,
        ]
        .map(|scale| {
            RegionalEnvironment::new(
                crate::regional_terrain::capture(
                    &pack,
                    RegionalTerrainRequest {
                        origin: Wgs84CoordinateMicrodegrees::from_e7(origin),
                        scale,
                    },
                )
                .unwrap(),
                Vec::new(),
            )
            .unwrap()
        });
        for (kind, bytes) in [
            ("actor", serde_json::to_vec(&input).unwrap()),
            ("city", serde_json::to_vec(&document).unwrap()),
            ("environment", serde_json::to_vec(&environments).unwrap()),
        ] {
            fs::write(output.join(format!("{kind}-{name}.json")), bytes).unwrap();
        }
        println!(
            "exported {name}: population {population}, {} physical buildings",
            input.physical_placements().len()
        );
    }
    drop(pack);
    fs::remove_dir_all(temporary).unwrap();
}
