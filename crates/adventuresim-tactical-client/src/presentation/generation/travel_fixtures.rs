//! Reproducible distinct destinations for the real-renderer travel benchmark.
use super::*;
use adventuresim_tactical_core::scene_input::SceneBuildingId;
use fabelgeist_determinism::Seed;

#[test]
#[ignore = "writes explicit benchmark scene inputs to STRATEGIC_TRAVEL_FIXTURE_DIR"]
fn distinct_city_inputs_validate_occupied_layouts() {
    let output = std::env::var("STRATEGIC_TRAVEL_FIXTURE_DIR").expect("fixture output directory");
    std::fs::create_dir_all(&output).unwrap();
    let base: TacticalSceneInput = serde_json::from_str(include_str!(
        "../../../../../assets/tactical-scenes/massive-city.json"
    ))
    .unwrap();
    let request: venue::VenueRequest = serde_json::from_value(serde_json::json!({
        "places": [
            {"id":"public-square","kind":"square"}, {"id":"residences","kind":"residence"},
            {"id":"keep","kind":"keep"}, {"id":"merchants","kind":"market"},
            {"id":"weapons","kind":"smith"}, {"id":"armor","kind":"armor"},
            {"id":"clothing","kind":"tailor"}, {"id":"herbalist","kind":"apothecary"},
            {"id":"books","kind":"books"}, {"id":"inn","kind":"inn"},
            {"id":"religion","kind":"church"}
        ], "people": []
    }))
    .unwrap();
    let occupied = request.placements(&base);
    for (index, (offset, destination_seeds)) in [1_u64, 1001]
        .into_iter()
        .zip(DESTINATION_SEEDS.iter())
        .enumerate()
    {
        let mut input = base.clone();
        input.seed = input.seed.wrapping_offset(offset);
        input.scene_key = format!("travel-distinct-{}", index + 1);
        for &DestinationSeed { building, seed } in destination_seeds {
            if let Some(placement) = input.buildings.iter_mut().find(|p| p.id == building) {
                placement.program.seed = seed;
            } else {
                input
                    .distant_buildings
                    .iter_mut()
                    .find(|p| p.id == building)
                    .unwrap()
                    .seed = seed;
            }
        }
        let destinations = request.placements(&input);
        assert_eq!(destinations.len(), destination_seeds.len());
        for placement in destinations {
            let original = occupied.iter().find(|p| p.id == placement.id).unwrap();
            assert_ne!(original.program, placement.program);
            let recipe =
                adventuresim_tactical_core::scene_input::GeneratedBuildingRecipe::generate(
                    placement.program,
                )
                .unwrap();
            adventuresim_building_generator::interior::furnish(&recipe.plan, &recipe.program)
                .unwrap();
        }
        input.generate().unwrap();
        let path =
            std::path::Path::new(&output).join(format!("travel-distinct-{}.json", index + 1));
        std::fs::write(path, serde_json::to_vec(&input).unwrap()).unwrap();
        println!(
            "validated destination {}: {} distinct occupied programs",
            index + 1,
            occupied.len()
        );
    }
}

// Authored fixture seeds stay fixed so generator changes cannot silently choose
// easier layouts and invalidate benchmark comparisons.
struct DestinationSeed {
    building: SceneBuildingId,
    seed: Seed,
}

const DESTINATION_SEEDS: [[DestinationSeed; 15]; 2] = [
    [
        DestinationSeed {
            building: SceneBuildingId(2),
            seed: Seed::from_u64(48),
        },
        DestinationSeed {
            building: SceneBuildingId(4),
            seed: Seed::from_u64(10192903604835745921),
        },
        DestinationSeed {
            building: SceneBuildingId(5),
            seed: Seed::from_u64(43),
        },
        DestinationSeed {
            building: SceneBuildingId(8),
            seed: Seed::from_u64(102),
        },
        DestinationSeed {
            building: SceneBuildingId(10),
            seed: Seed::from_u64(48),
        },
        DestinationSeed {
            building: SceneBuildingId(11),
            seed: Seed::from_u64(48),
        },
        DestinationSeed {
            building: SceneBuildingId(12),
            seed: Seed::from_u64(102),
        },
        DestinationSeed {
            building: SceneBuildingId(58),
            seed: Seed::from_u64(43),
        },
        DestinationSeed {
            building: SceneBuildingId(66),
            seed: Seed::from_u64(48),
        },
        DestinationSeed {
            building: SceneBuildingId(70),
            seed: Seed::from_u64(43),
        },
        DestinationSeed {
            building: SceneBuildingId(154),
            seed: Seed::from_u64(48),
        },
        DestinationSeed {
            building: SceneBuildingId(193),
            seed: Seed::from_u64(49),
        },
        DestinationSeed {
            building: SceneBuildingId(390),
            seed: Seed::from_u64(102),
        },
        DestinationSeed {
            building: SceneBuildingId(596),
            seed: Seed::from_u64(102),
        },
        DestinationSeed {
            building: SceneBuildingId(940),
            seed: Seed::from_u64(103),
        },
    ],
    [
        DestinationSeed {
            building: SceneBuildingId(2),
            seed: Seed::from_u64(1048),
        },
        DestinationSeed {
            building: SceneBuildingId(4),
            seed: Seed::from_u64(10192903604835746921),
        },
        DestinationSeed {
            building: SceneBuildingId(5),
            seed: Seed::from_u64(1043),
        },
        DestinationSeed {
            building: SceneBuildingId(8),
            seed: Seed::from_u64(1102),
        },
        DestinationSeed {
            building: SceneBuildingId(10),
            seed: Seed::from_u64(1048),
        },
        DestinationSeed {
            building: SceneBuildingId(11),
            seed: Seed::from_u64(1048),
        },
        DestinationSeed {
            building: SceneBuildingId(12),
            seed: Seed::from_u64(1102),
        },
        DestinationSeed {
            building: SceneBuildingId(58),
            seed: Seed::from_u64(1043),
        },
        DestinationSeed {
            building: SceneBuildingId(66),
            seed: Seed::from_u64(1048),
        },
        DestinationSeed {
            building: SceneBuildingId(70),
            seed: Seed::from_u64(1043),
        },
        DestinationSeed {
            building: SceneBuildingId(154),
            seed: Seed::from_u64(1048),
        },
        DestinationSeed {
            building: SceneBuildingId(193),
            seed: Seed::from_u64(1048),
        },
        DestinationSeed {
            building: SceneBuildingId(390),
            seed: Seed::from_u64(1102),
        },
        DestinationSeed {
            building: SceneBuildingId(596),
            seed: Seed::from_u64(1102),
        },
        DestinationSeed {
            building: SceneBuildingId(940),
            seed: Seed::from_u64(1102),
        },
    ],
];
