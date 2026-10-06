//! A complete settlement using the production generator and fixture recipes.
use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::*;

const RESIDENT_POPULATION: u32 = 30_000;
const CITY_SEED: u64 = 47_114;

fn curate(mut input: TacticalSceneInput) -> Result<TacticalSceneInput, String> {
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
    let city = CitySite::central_german_market_town().generate(
        (CITY_SEED).into(),
        RESIDENT_POPULATION,
        &economy,
    );
    let layout = city
        .compile((CITY_SEED).into())
        .and_then(|city| city.partition(None))
        .map_err(|error| error.to_string())?;
    input.establishments = layout
        .businesses
        .iter()
        .map(|site| {
            let mut establishment = input
                .establishments
                .iter()
                .find(|operator| operator.business_id.key == site.key)
                .cloned()
                .ok_or_else(|| {
                    format!(
                        "fixture lacks operator identity for business {:?}",
                        site.key
                    )
                })?;
            establishment.building_id = site.building_id;
            Ok(establishment)
        })
        .collect::<Result<Vec<_>, String>>()?;
    input.buildings = layout.playable.clone();
    input.distant_buildings = layout.distant.clone();
    input.streets = layout.streets.clone();
    input.yards = layout.yards.clone();
    input.parishes = layout.parishes.clone();
    input.compounds = layout.compounds.clone();
    input.gardens = layout.gardens.clone();
    // This is offline authoring of a distinct complete layout, not loading or
    // repairing a partially bound occupied scene in the browser.
    input.grounding = None;
    input
        .ground_generated_city(
            &layout,
            adventuresim_tactical_core::city_layout::CompoundGradingPolicy::bounded_settlement(),
        )
        .map_err(|error| error.to_string())
}

fn main() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = std::fs::read_to_string(root.join("assets/tactical-scenes/massive-city.json"))
        .map_err(|error| error.to_string())?;
    let input: TacticalSceneInput =
        serde_json::from_str(&source).map_err(|error| error.to_string())?;
    let input = curate(input)?;
    let generated = input.generate().map_err(|error| error.to_string())?;
    let furniture = generated.furniture;
    let instances = furniture
        .instances
        .into_iter()
        .chain(furniture.distant_instances)
        .collect::<Vec<_>>();
    let layout = serde_json::json!({
        "resident_population": RESIDENT_POPULATION,
        "schema_version": input.schema_version,
        "generation_version": input.generation_version,
        "grounding": input.grounding,
        "establishments": input.establishments,
        "buildings": input.distant_buildings,
        "streets": input.streets,
        "yards": input.yards,
        "parishes": input.parishes,
        "compounds": input.compounds,
        "gardens": input.gardens,
    });
    write_changed(
        &root.join("assets/art-demo/city-layout.json"),
        serde_json::to_string(&layout).map_err(|error| error.to_string())? + "\n",
    )
    .map_err(|error| error.to_string())?;
    write_changed(
        &root.join("assets/art-demo/city-furniture.json"),
        serde_json::to_string(&serde_json::json!({
            "instances": instances, "groups": furniture.groups,
        }))
        .map_err(|error| error.to_string())?
            + "\n",
    )
    .map_err(|error| error.to_string())?;
    println!(
        "Generated {} residents, {} buildings",
        RESIDENT_POPULATION,
        input.distant_buildings.len()
    );
    Ok(())
}

fn write_changed(path: &std::path::Path, contents: String) -> std::io::Result<()> {
    assert!(
        contents.len() as u64 <= adventuresim_tactical_core::scene_input::MAX_SCENE_INPUT_BYTES,
        "prepared art-demo asset exceeds the scene-input payload bound"
    );
    if std::fs::read(path).ok().as_deref() == Some(contents.as_bytes()) {
        return Ok(());
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents)?;
    std::fs::rename(temporary, path)
}
