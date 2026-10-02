//! A complete settlement using the production generator and fixture recipes.
use adventuresim_tactical_core::prelude::*;
use adventuresim_world_schema::*;

const RESIDENT_POPULATION: u32 = 30_000;
const CITY_SEED: u64 = 47_114;

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
    let city =
        CitySite::central_german_market_town().generate(CITY_SEED, RESIDENT_POPULATION, &economy);
    let layout = city
        .compile(CITY_SEED)
        .and_then(|city| city.partition(None))
        .map_err(|error| error.to_string())?;
    input.buildings = layout.playable;
    input.distant_buildings = layout.distant;
    input.streets = layout.streets;
    input.yards = layout.yards;
    input.parishes = layout.parishes;
    input.compounds = layout.compounds;
    input.gardens = layout.gardens;
    Ok(())
}

fn main() -> Result<(), String> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = std::fs::read_to_string(root.join("assets/tactical-scenes/massive-city.json"))
        .map_err(|error| error.to_string())?;
    let mut input: TacticalSceneInput =
        serde_json::from_str(&source).map_err(|error| error.to_string())?;
    curate(&mut input)?;
    let generated = input.generate().map_err(|error| error.to_string())?;
    let furniture = generated.furniture;
    let instances = furniture
        .instances
        .into_iter()
        .chain(furniture.distant_instances)
        .collect::<Vec<_>>();
    let layout = serde_json::json!({
        "resident_population": RESIDENT_POPULATION,
        "buildings": input.distant_buildings,
        "streets": input.streets,
        "yards": input.yards,
        "parishes": input.parishes,
        "compounds": input.compounds,
        "gardens": input.gardens,
    });
    write_changed(
        &root.join("assets/art-demo/city-layout.json"),
        serde_json::to_string_pretty(&layout).map_err(|error| error.to_string())? + "\n",
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
    if std::fs::read(path).ok().as_deref() == Some(contents.as_bytes()) {
        return Ok(());
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents)?;
    std::fs::rename(temporary, path)
}
