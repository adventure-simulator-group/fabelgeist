//! A complete settlement using the production generator and fixture recipes.
use adventuresim_building_generator::BuildingProgram;
use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::*;
use fabelgeist_determinism::splitmix64;

const RESIDENT_POPULATION: u32 = 30_000;
const CITY_SEED: u64 = 47_114;
const RECIPE_SELECTION_SALT: u64 = 0x6469_7374_616e_7401;

fn curate(input: &mut TacticalSceneInput) -> Result<(), String> {
    let economy = infer_settlement_economy(
        5,
        RESIDENT_POPULATION,
        6,
        true,
        &InferredIndustryProfile::new(vec![IndustryEvidence::Fallback(
            FallbackIndustry::CroplandGrain,
        )])
        .ok_or("city industry must be valid")?,
    )
    .map_err(|error| format!("city economy: {error:?}"))?;
    let city = generate_city(CITY_SEED, RESIDENT_POPULATION, &economy);
    let mut recipes = std::collections::BTreeMap::new();
    input.buildings.clear();
    input.distant_buildings.clear();
    for lot in city.lots {
        let archetype = lot.archetype();
        let usage = lot
            .building_use()
            .unwrap_or(settlement_buildings::BuildingUse::Dwelling);
        let seeds = [42, 47, 101];
        let seed =
            seeds[splitmix64(CITY_SEED ^ RECIPE_SELECTION_SALT ^ lot.id) as usize % seeds.len()];
        let size = lot.service_size();
        let key = (archetype.slug(), usage, seed, size);
        if let std::collections::btree_map::Entry::Vacant(entry) = recipes.entry(key) {
            entry.insert(
                BuildingProgram::validated_settlement(archetype, usage, seed, size)
                    .map_err(|error| format!("city building: {error:?}"))?,
            );
        }
        let program = &recipes[&key];
        input.distant_buildings.push(DistantBuildingPlacement {
            id: lot.id,
            archetype,
            usage: Some(usage),
            service_size: program.service_size,
            seed: program.seed,
            centre_metres: lot.centre_metres,
            base_elevation_metres: 0.0,
            orientation: lot.orientation,
        });
    }
    input.streets = city.streets;
    input.yards = city.yards;
    Ok(())
}

fn main() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = std::fs::read_to_string(root.join("assets/tactical-scenes/massive-city.json"))
        .map_err(|error| error.to_string())?;
    let mut input: TacticalSceneInput =
        serde_json::from_str(&source).map_err(|error| error.to_string())?;
    curate(&mut input)?;
    let layout = serde_json::json!({
        "resident_population": RESIDENT_POPULATION,
        "buildings": input.distant_buildings,
        "streets": input.streets,
        "yards": input.yards,
    });
    std::fs::write(
        root.join("assets/art-demo/city-layout.json"),
        serde_json::to_string_pretty(&layout).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| error.to_string())?;
    println!(
        "Generated {} residents, {} buildings",
        RESIDENT_POPULATION,
        input.distant_buildings.len()
    );
    Ok(())
}
