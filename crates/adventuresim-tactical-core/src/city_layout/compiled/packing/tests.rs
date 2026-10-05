use super::*;

#[test]
fn measured_packing_retains_selected_roster_and_capacity() {
    for (seed, population) in [(42, 900), (101, 6500)] {
        let mut generated = CitySite::central_german_market_town().generate(
            seed,
            population,
            &super::super::super::tests::economy(),
        );
        let context = std::mem::replace(
            &mut generated.packing,
            Ok(super::super::super::packing::CityPackingContext::default()),
        )
        .unwrap();
        let mut compiled = generated.compile_properties(seed).unwrap();
        let before = compiled.clone();
        compiled.finalize_packing(&context).unwrap();
        if population == 6500 {
            save_pair(&before, &compiled);
            assert_diagnosed_pair_is_separate(&before, &mut compiled);
        }
        assert_eq!(compiled.parishes, before.parishes);
        assert_eq!(compiled.businesses, before.businesses);
        assert_eq!(compiled.streets, before.streets);
        assert_eq!(compiled.support_recipes, before.support_recipes);
        assert_eq!(compiled.buildings.len(), before.buildings.len());
        for (actual, original) in compiled.buildings.iter().zip(&before.buildings) {
            let mut expected = original.clone();
            expected.centre_metres = actual.centre_metres;
            assert_eq!(*actual, expected);
        }
        assert_eq!(
            compiled
                .compounds
                .iter()
                .map(|p| (p.id, p.front_building_id, p.rear_building_id))
                .collect::<Vec<_>>(),
            before
                .compounds
                .iter()
                .map(|p| (p.id, p.front_building_id, p.rear_building_id))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            compiled
                .gardens
                .iter()
                .map(|g| (
                    g.owner,
                    g.front_building_id,
                    g.plants
                        .iter()
                        .map(|p| (p.id, p.specimen, p.scale))
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>(),
            before
                .gardens
                .iter()
                .map(|g| (
                    g.owner,
                    g.front_building_id,
                    g.plants
                        .iter()
                        .map(|p| (p.id, p.specimen, p.scale))
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn twelve_thousand_seed_101_retains_buildable_complete_properties() {
    CitySite::central_german_market_town()
        .generate(101, 12000, &super::super::super::tests::economy())
        .compile(101)
        .unwrap();
}

fn save_pair(before: &CompiledCityLayout, after: &CompiledCityLayout) {
    let Some(path) = std::env::var_os("FABELGEIST_SUPPORT_DIAGNOSTIC_DIR") else {
        return;
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path);
    std::fs::create_dir_all(&path).unwrap();
    for (label, layout) in [("before", before), ("after", after)] {
        let buildings: Vec<_> = layout
            .buildings
            .iter()
            .filter(|b| [57, 193].contains(&b.id))
            .collect();
        std::fs::write(path.join(format!("packing-6500-101-pair-{label}.json")),serde_json::to_vec_pretty(&serde_json::json!({
            "fixture":"population 6500, seed 101, economy inn/temple", "buildings":buildings,
            "scope":"Exact production programme selection and packing; architectural floor zero control before geographic support selection."
        })).unwrap()).unwrap();
    }
}

fn assert_diagnosed_pair_is_separate(before: &CompiledCityLayout, after: &mut CompiledCityLayout) {
    let envelope = |layout: &mut CompiledCityLayout, id| {
        let building = layout
            .buildings
            .iter()
            .find(|b| b.id == id)
            .unwrap()
            .clone();
        let recipe = layout
            .support_recipes
            .for_program(&building.program)
            .unwrap();
        CityPlotBounds {
            centre_metres: building.centre_metres
                + building
                    .orientation
                    .local_to_world((recipe.render_min + recipe.render_max) * 0.5),
            dimensions_metres: recipe.render_max - recipe.render_min,
            orientation: building.orientation,
        }
    };
    let mut before = before.clone();
    assert!(
        envelope(&mut before, 57).intersects(envelope(&mut before, 193)),
        "the fixed 6500/101 case must reproduce its original exterior conflict"
    );
    assert!(
        !envelope(after, 57).intersects(envelope(after, 193)),
        "accepted full exterior projections must separate the diagnosed pair at every floor datum"
    );
    assert!(after.buildings.iter().any(|b| {
        before
            .buildings
            .iter()
            .find(|old| old.id == b.id)
            .unwrap()
            .centre_metres
            != b.centre_metres
    }));
}

#[test]
fn packing_is_independent_of_selected_lot_iteration_order() {
    let generated = CitySite::central_german_market_town().generate(
        47,
        900,
        &super::super::super::tests::economy(),
    );
    let mut reversed = generated.clone();
    reversed.lots.reverse();
    assert_eq!(
        generated.compile(47).unwrap(),
        reversed.compile(47).unwrap()
    );
}

#[test]
fn twelve_thousand_seed_47_retains_garden_bounds() {
    CitySite::central_german_market_town()
        .generate(47, 12000, &super::super::super::tests::economy())
        .compile(47)
        .unwrap();
}

#[test]
fn residence_authority_town_keeps_complete_buildable_properties() {
    let seed = adventuresim_core::settlement_population::settlement_building_seed("town");
    let mut generated = CitySite::central_german_market_town().generate(
        seed,
        6500,
        &SettlementEconomyProfile::stage_placeholder(),
    );
    let context = std::mem::replace(
        &mut generated.packing,
        Ok(super::super::super::packing::CityPackingContext::default()),
    )
    .unwrap();
    let mut layout = generated.compile_properties(seed).unwrap();
    let before = layout.clone();
    let property = before
        .single_properties
        .iter()
        .find(|p| p.id == CityPropertyId(615))
        .unwrap();
    assert!(
        single_bearing_points(&before, property)
            .iter()
            .any(|point| !property.plot.contains(*point)),
        "the nominal placement must reproduce the ground bearing outside its property"
    );
    layout.finalize_packing(&context).unwrap();
    for property in &layout.single_properties {
        assert!(
            single_bearing_points(&layout, property)
                .iter()
                .all(|point| property.plot.contains(*point)),
            "member {} bearing lies outside its unchanged owned dimensions",
            property.building_id
        );
        let original = before
            .buildings
            .iter()
            .find(|b| b.id == property.building_id)
            .unwrap();
        let accepted = layout
            .buildings
            .iter()
            .find(|b| b.id == property.building_id)
            .unwrap();
        assert_eq!(original.program, accepted.program);
        assert_eq!(original.orientation, accepted.orientation);
    }
    assert_eq!(
        before.gardens.iter().map(|g| g.owner).collect::<Vec<_>>(),
        layout.gardens.iter().map(|g| g.owner).collect::<Vec<_>>()
    );
}

fn single_bearing_points(layout: &CompiledCityLayout, property: &CitySingleProperty) -> Vec<Vec2> {
    let building = layout
        .buildings
        .iter()
        .find(|b| b.id == property.building_id)
        .unwrap();
    let mut recipes = layout.support_recipes.clone();
    let recipe = recipes.for_program(&building.program).unwrap();
    let origin = recipe.collision.bounds.centre();
    recipe
        .collision
        .ground_floor_footprint()
        .unwrap()
        .unwrap()
        .vertices()
        .iter()
        .map(|point| {
            building.centre_metres
                + building
                    .orientation
                    .local_to_world(point.metres() - Vec2::new(origin.x, origin.z))
        })
        .collect()
}

#[test]
fn insufficient_owned_plot_rejects_the_exact_member_without_programme_substitution() {
    let seed = adventuresim_core::settlement_population::settlement_building_seed("town");
    let mut layout = CitySite::central_german_market_town()
        .generate(seed, 6500, &SettlementEconomyProfile::stage_placeholder())
        .compile_properties(seed)
        .unwrap();
    let before = layout
        .buildings
        .iter()
        .map(|b| (b.id, b.program.clone()))
        .collect::<Vec<_>>();
    let property = layout
        .single_properties
        .iter_mut()
        .find(|p| p.id == CityPropertyId(615))
        .unwrap();
    property.plot.dimensions_metres = Vec2::splat(4.0);
    let error = layout.seat_single_bearings().unwrap_err();
    assert!(
        matches!(error,CityCompileError::Packing { property:CityPropertyId(615), issue:CityPackingIssue::BearingOutsidePlot { building:615, plot_half_dimensions_metres, minimum_local_metres, maximum_local_metres } } if plot_half_dimensions_metres==Vec2::splat(2.0) && (maximum_local_metres-minimum_local_metres).min_element()>4.0)
    );
    assert_eq!(
        before,
        layout
            .buildings
            .iter()
            .map(|b| (b.id, b.program.clone()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn large_population_frontage_retains_complete_measured_properties() {
    use adventuresim_world_schema::*;
    let population = 100_000;
    let economy = infer_settlement_economy(
        4,
        population,
        3,
        true,
        &InferredIndustryProfile::new(vec![IndustryEvidence::Fallback(
            FallbackIndustry::CroplandGrain,
        )])
        .unwrap(),
    )
    .unwrap();
    let seed =
        adventuresim_core::settlement_population::settlement_building_seed("massive-city-3229");
    let city = CitySite::central_german_market_town().generate(seed, population, &economy);
    assert_eq!(city.unhoused_population, 0);
    assert!(city.unplaced_services.is_empty());
    let selected = city.lots.len();
    let context = city.packing.clone().unwrap();
    let mut compiled = city
        .compile_properties(seed)
        .expect("fixed roster compiles");
    let original_buildings = compiled.buildings.clone();
    let original_gardens = compiled.gardens.clone();
    let original_compounds = compiled.compounds.clone();
    let original_businesses = compiled.businesses.clone();
    compiled
        .finalize_packing(&context)
        .expect("complete measured roster fits its city");
    assert_eq!(compiled.businesses, original_businesses);
    assert_eq!(compiled.gardens.len(), original_gardens.len());
    for (before, after) in original_buildings.iter().zip(&compiled.buildings) {
        let mut expected = before.clone();
        expected.centre_metres = after.centre_metres;
        assert_eq!(&expected, after, "only horizontal placement may change");
    }
    for (before, after) in original_gardens.iter().zip(&compiled.gardens) {
        let delta = after.plot.centre_metres - before.plot.centre_metres;
        assert_eq!(before.owner, after.owner);
        assert_eq!(before.front_building_id, after.front_building_id);
        assert_eq!(before.beds.len(), after.beds.len());
        assert_eq!(before.plants.len(), after.plants.len());
        for (original, moved) in before.plants.iter().zip(&after.plants) {
            assert_eq!(original.id, moved.id);
            assert_eq!(original.specimen, moved.specimen);
            assert_eq!(original.scale, moved.scale);
            assert_eq!(original.orientation, moved.orientation);
            assert!(
                (moved.centre_metres - original.centre_metres - delta).length()
                    < CityPlotBounds::COORDINATE_TOLERANCE_METRES as f32
            );
        }
        assert_eq!(before.plot.dimensions_metres, after.plot.dimensions_metres);
        assert_eq!(before.plot.orientation, after.plot.orientation);
        after.validate_geometry(&compiled.streets).unwrap();
    }
    for (before, after) in original_compounds.iter().zip(&compiled.compounds) {
        assert_eq!(before.id, after.id);
        assert_eq!(before.front_building_id, after.front_building_id);
        assert_eq!(before.rear_building_id, after.rear_building_id);
        assert_eq!(before.plot.dimensions_metres, after.plot.dimensions_metres);
        assert_eq!(
            before.court.dimensions_metres,
            after.court.dimensions_metres
        );
        assert_eq!(before.boundary.walls.len(), after.boundary.walls.len());
        assert_eq!(
            before.boundary.gate.width_metres,
            after.boundary.gate.width_metres
        );
    }
    assert_eq!(
        compiled.buildings.len(),
        selected + compiled.compounds.len()
    );
}
