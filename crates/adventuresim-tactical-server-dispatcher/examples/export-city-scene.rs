//! Offline city capture through the production dispatcher and geographic terrain.
//! Operators are explicit appearance fixtures, not projected persistent residents.
use adventuresim_core::{
    reputation::effective_population, settlement_population::settlement_building_seed,
};
use adventuresim_tactical_core::scene_input::TacticalSceneInput;
use adventuresim_tactical_server_dispatcher::{
    scene_input::{ImportedTerrainCapture, build_imported_scene},
    settlement_buildings::{SettlementBusinessOperatorProfile, SettlementSceneProfile},
};
use adventuresim_terrain::{TerrainPack, TerrainPurpose};
use adventuresim_world_schema::{
    CURRENT_INFERENCE_RULES_VERSION, CompiledWorld, SettlementImport, WORLD_SCHEMA_VERSION,
    calendar::StrategicMinute,
    coordinates::{LatitudeE7, LongitudeE7, Wgs84CoordinateE7},
    person_names::RenderedPersonalName,
    settlement_buildings::{BusinessId, SettlementBuildingDemand},
};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Capture {
    #[arg(long)]
    world: PathBuf,
    #[arg(long)]
    settlement: String,
    #[arg(long)]
    terrain_manifest: PathBuf,
    #[arg(long)]
    terrain_pack: PathBuf,
    #[arg(long, value_parser = parse_minute)]
    absolute_minute: StrategicMinute,
    #[arg(long)]
    output: PathBuf,
    /// Export the authoritative home catalogue independently of any UI.
    #[arg(long)]
    property_catalog: Option<PathBuf>,
    /// Write source transects and production samples before city grading.
    #[arg(long)]
    terrain_stages: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let capture = Capture::parse();
    let world: CompiledWorld = serde_json::from_slice(&std::fs::read(&capture.world)?)?;
    if world.metadata.schema_version != WORLD_SCHEMA_VERSION
        || world.metadata.inference_rules_version != CURRENT_INFERENCE_RULES_VERSION
    {
        return Err("capture requires the current world schema and inference rules".into());
    }
    let settlement = world
        .settlements
        .iter()
        .find(|settlement| settlement.id == capture.settlement)
        .ok_or("settlement is absent from the imported world")?;
    let terrain = TerrainPack::load(&capture.terrain_manifest, &capture.terrain_pack)?;
    if terrain.purpose() != TerrainPurpose::Final {
        return Err("capture requires a final terrain pack".into());
    }
    let minute = capture.absolute_minute;
    let input = build_imported_scene(
        &terrain,
        &format!("city-capture:{}:{}", settlement.id, terrain.digest()),
        &settlement.scene_key,
        LatitudeE7::from_degrees(settlement.latitude)
            .ok_or("invalid latitude")?
            .get(),
        LongitudeE7::from_degrees(settlement.longitude)
            .ok_or("invalid longitude")?
            .get(),
        minute,
        minute,
        Some(&appearance_fixture(settlement)?),
    )?;
    if let Some(parent) = capture.output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&capture.output, serde_json::to_vec(&input)?)?;
    capture.export_terrain_stages(&terrain, settlement, &input)?;
    if let Some(path) = &capture.property_catalog {
        std::fs::write(
            path,
            serde_json::to_vec(input.properties.as_ref().ok_or("missing home catalogue")?)?,
        )?;
    }
    println!(
        "{}: {} at {}, {} — {} buildings; world {}; terrain {}",
        settlement.id,
        settlement.name.trim(),
        settlement.latitude,
        settlement.longitude,
        input.buildings.len() + input.distant_buildings.len(),
        world.metadata.manifest_digest,
        terrain.digest(),
    );
    Ok(())
}

impl Capture {
    fn export_terrain_stages(
        &self,
        terrain: &TerrainPack,
        settlement: &SettlementImport,
        input: &TacticalSceneInput,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let Some(path) = &self.terrain_stages else {
            return Ok(());
        };
        let coordinates = Wgs84CoordinateE7::new(
            LatitudeE7::from_degrees(settlement.latitude)
                .ok_or("invalid latitude")?
                .get(),
            LongitudeE7::from_degrees(settlement.longitude)
                .ok_or("invalid longitude")?
                .get(),
        )
        .ok_or("invalid geographic coordinate")?;
        let stages = ImportedTerrainCapture::sample(terrain, coordinates, input.seed)?;
        let source_compounds = input.compounds.iter().map(|property| {
            let samples = ImportedTerrainCapture::sample_points(terrain, coordinates, property.plot.corners())?;
            Ok(serde_json::json!({"property_id":property.id, "members":[property.front_building_id,property.rear_building_id], "source_samples":samples}))
        }).collect::<Result<Vec<_>, String>>()?;
        std::fs::write(
            path,
            serde_json::to_vec(&serde_json::json!({
                "input_digest":input.digest()?, "seed":input.seed,
                "absolute_minute":input.absolute_minute,
                "latitude_e7":coordinates.latitude().get(),
                "longitude_e7":coordinates.longitude().get(),
                "source_digest": stages.source_digest,
                "absolute_elevation_metres": stages.absolute_elevation_metres,
                "source_transects": stages.source_transects,
                "ungraded_vista": stages.ungraded_vista,
                "source_compounds": source_compounds,
            }))?,
        )?;
        Ok(())
    }
}

fn parse_minute(value: &str) -> Result<StrategicMinute, std::num::ParseIntError> {
    value.parse().map(StrategicMinute::new)
}

fn appearance_fixture(
    settlement: &SettlementImport,
) -> Result<SettlementSceneProfile, Box<dyn std::error::Error>> {
    let demand = SettlementBuildingDemand::new(
        settlement_building_seed(&settlement.id),
        effective_population(settlement.population_level, settlement.population_estimate),
        &settlement.economy,
    );
    let operators = demand
        .buildings
        .into_iter()
        .filter_map(|demand| demand.business_key())
        .enumerate()
        .map(|(index, key)| {
            Ok(SettlementBusinessOperatorProfile {
                business_id: BusinessId::new(&settlement.id, key),
                operator_character_id: u64::try_from(index)? + 1,
                operator_name: RenderedPersonalName::try_from(format!("Capture operator {index}"))?,
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    Ok(SettlementSceneProfile {
        id: settlement.id.clone(),
        population_level: settlement.population_level,
        population_estimate: settlement.population_estimate,
        economy: settlement.economy.clone(),
        operators,
    })
}
