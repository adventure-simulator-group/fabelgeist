//! Required synthetic sizes exercise the complete production handoff.
use super::*;
use adventuresim_core::prelude::weather_at;
use adventuresim_world_schema::calendar::StrategicMinute;

#[test]
fn production_required_sizes_preserve_all_homes_services_bindings_and_soil_roots() {
    let cases = [900, 6500, 12000]
        .into_iter()
        .flat_map(|population| [42, 47, 101].map(|seed| (population, seed)))
        .chain(std::iter::once((30000, 101)));
    let mut rows = Vec::new();
    for (population, seed) in cases {
        let started = std::time::Instant::now();
        let identity = format!("terrain-support-{population}-{seed}");
        let (mut draft, layout) = city_input_for(seed, population);
        draft.scene_key = identity.clone();
        draft.source = SceneSource::SyntheticFixture(identity.clone());
        draft.absolute_minute = StrategicMinute::new(340320);
        draft.lunar_phase_minute = draft.absolute_minute;
        draft.weather = weather_at(
            seed,
            draft.absolute_minute,
            draft.latitude_microdegrees,
            draft.longitude_microdegrees,
            draft.absolute_elevation_metres,
        );
        let homes = layout.generated_homes(&identity, population).unwrap();
        let original = draft.clone();
        let accepted = draft
            .ground_generated_city(&layout, CompoundGradingPolicy::bounded_settlement())
            .unwrap_or_else(|error| panic!("required positive {identity}: {error}"));
        assert_eq!(accepted.compounds, original.compounds);
        assert_eq!(accepted.gardens, original.gardens);
        assert_eq!(accepted.streets, original.streets);
        assert_eq!(accepted.yards, original.yards);
        assert_eq!(accepted.playable, original.playable);
        assert_eq!(accepted.vista, original.vista);
        let final_layout = CitySceneLayout {
            playable: accepted.buildings.clone(),
            distant: accepted.distant_buildings.clone(),
            ..layout
        };
        assert_eq!(
            final_layout.generated_homes(&identity, population).unwrap(),
            homes
        );
        for (before, after) in original
            .physical_placements()
            .iter()
            .zip(accepted.physical_placements())
        {
            assert_eq!(before.id, after.id);
            assert_eq!(before.program, after.program);
            assert_eq!(before.centre_metres, after.centre_metres);
            assert_eq!(before.orientation, after.orientation);
        }
        let encoded = serde_json::to_vec(&accepted).unwrap();
        assert!(encoded.len() as u64 <= MAX_SCENE_INPUT_BYTES);
        if let Ok(directory) = std::env::var("FABELGEIST_SUPPORT_SCENE_OUTPUT") {
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(
                std::path::Path::new(&directory).join(format!("{identity}.json")),
                &encoded,
            )
            .unwrap();
        }
        let restored: TacticalSceneInput = serde_json::from_slice(&encoded).unwrap();
        let generated = restored
            .generate_unfurnished(GeneratedBuildingRecipes::default())
            .unwrap_or_else(|error| panic!("production consumer {identity}: {error}"));
        assert_eq!(generated.repairs.levelled_building_samples, 0);
        let mut enclosure_cells = 0;
        for compound in &restored.compounds {
            let enclosure =
                crate::scene_input::GeneratedBoundary::project(compound, &generated.terrain)
                    .unwrap_or_else(|error| {
                        panic!(
                            "required positive {identity}, enclosure {:?}: {error}",
                            compound.id
                        )
                    });
            assert_eq!(enclosure.scene.property_id, compound.id);
            assert_eq!(
                enclosure.scene.front_building_id,
                compound.front_building_id
            );
            assert_eq!(enclosure.scene.boundary, compound.boundary);
            let _collider = enclosure.scene.fixed_support.collider();
            enclosure_cells += enclosure.scene.fixed_support.cells.len();
            let bytes = postcard::to_allocvec(&enclosure).unwrap();
            let replicated: crate::scene_input::GeneratedBoundary =
                postcard::from_bytes(&bytes).unwrap();
            assert_eq!(replicated.scene, enclosure.scene);
            assert_eq!(replicated.elevation_metres, enclosure.elevation_metres);
        }
        for garden in &restored.gardens {
            let projection =
                crate::scene_input::SceneGarden::project(garden.clone(), &generated.terrain)
                    .unwrap();
            assert_eq!(projection.garden, *garden);
            assert_eq!(projection.plant_support.len(), garden.plants.len());
        }
        rows.push(serde_json::json!({"fixture":identity,"seed":seed,"population":population,
            "absolute_minute":340320,"schema":restored.schema_version,"generation":restored.generation_version,"input_digest":restored.digest().unwrap(),"bytes":encoded.len(),
            "buildings":restored.buildings.len()+restored.distant_buildings.len(),
            "homes":homes.homes.len(),"gardens":restored.gardens.len(),"enclosures":restored.compounds.len(),"enclosure_cells":enclosure_cells,
            "seconds":started.elapsed().as_secs_f64(),"scope":"One native correctness run. Identity, capacity, compact support reconstruction and complete root and enclosure projections, materialized enclosure collision and replication; not full route/collision/renderer or calibrated performance acceptance."}));
        let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../target/terrain-grounding/production-required-support-matrix{}.json",
            TACTICAL_SCENE_GENERATION_VERSION
        ));
        std::fs::create_dir_all(output.parent().unwrap()).unwrap();
        std::fs::write(output, serde_json::to_vec_pretty(&rows).unwrap()).unwrap();
    }
}
