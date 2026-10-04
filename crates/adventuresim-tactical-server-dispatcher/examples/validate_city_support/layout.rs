//! Recreate exact reservations from the recorded settlement and verify identity.
use super::*;
use adventuresim_tactical_server_dispatcher::settlement_buildings::{
    SettlementSceneProfile, place_settlement_buildings,
};
use adventuresim_world_schema::{
    CURRENT_INFERENCE_RULES_VERSION, CompiledWorld, WORLD_SCHEMA_VERSION,
};

pub(super) fn reproduce(
    input: &TacticalSceneInput,
    world_path: &std::path::Path,
) -> Result<CitySceneLayout, Box<dyn std::error::Error>> {
    let world: CompiledWorld = serde_json::from_slice(&std::fs::read(world_path)?)?;
    if world.metadata.schema_version != WORLD_SCHEMA_VERSION
        || world.metadata.inference_rules_version != CURRENT_INFERENCE_RULES_VERSION
    {
        return Err("support reproduction requires the current imported world schema".into());
    }
    let id = &input
        .properties
        .as_ref()
        .ok_or("production fixture lacks authoritative home catalogue")?
        .settlement_id;
    let settlement = world
        .settlements
        .iter()
        .find(|s| s.id == *id)
        .ok_or("fixture settlement is absent from imported world")?;
    let profile = SettlementSceneProfile {
        id: settlement.id.clone(),
        population_level: settlement.population_level,
        population_estimate: settlement.population_estimate,
        economy: settlement.economy.clone(),
        operators: Vec::new(),
    };
    let extent = f32::from(input.playable.width - 1) * input.playable.spacing_metres * 0.5;
    let layout = place_settlement_buildings(&profile, extent)?;
    if layout.playable != input.buildings
        || layout.distant != input.distant_buildings
        || layout.compounds != input.compounds
        || layout.gardens != input.gardens
        || layout.streets != input.streets
    {
        return Err("recreated programmes, membership or horizontal placement differs from the frozen input".into());
    }
    Ok(layout)
}
