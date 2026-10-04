//! Reproducible distinct destinations for the real-renderer travel benchmark.
use super::*;

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
    for (index, offset) in [1_u64, 1001].into_iter().enumerate() {
        let mut input = base.clone();
        input.seed = input.seed.to_u64().wrapping_add(offset).into();
        input.scene_key = format!("travel-distinct-{}", index + 1);
        for &(id, seed) in &DESTINATION_SEEDS[index] {
            if let Some(placement) = input.buildings.iter_mut().find(|p| p.id == id) {
                placement.program.seed = seed;
            } else {
                input
                    .distant_buildings
                    .iter_mut()
                    .find(|p| p.id == id)
                    .unwrap()
                    .seed = seed;
            }
        }
        let destinations = request.placements(&input);
        assert_eq!(destinations.len(), DESTINATION_SEEDS[index].len());
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
const DESTINATION_SEEDS: [[(u64, u64); 15]; 2] = [
    [
        (2, 48),
        (4, 10192903604835745921),
        (5, 43),
        (8, 102),
        (10, 48),
        (11, 48),
        (12, 102),
        (58, 43),
        (66, 48),
        (70, 43),
        (154, 48),
        (193, 49),
        (390, 102),
        (596, 102),
        (940, 103),
    ],
    [
        (2, 1048),
        (4, 10192903604835746921),
        (5, 1043),
        (8, 1102),
        (10, 1048),
        (11, 1048),
        (12, 1102),
        (58, 1043),
        (66, 1048),
        (70, 1043),
        (154, 1048),
        (193, 1048),
        (390, 1102),
        (596, 1102),
        (940, 1102),
    ],
];
