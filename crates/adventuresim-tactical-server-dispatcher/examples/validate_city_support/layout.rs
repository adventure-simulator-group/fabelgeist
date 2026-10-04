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
    verify(input, &layout)?;
    Ok(layout)
}

fn verify(
    input: &TacticalSceneInput,
    layout: &CitySceneLayout,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check published floors against their accepted projection before excluding
    // them from the comparison with the compiler's unseated placements.
    input.validate()?;
    let playable_matches = layout.playable.len() == input.buildings.len()
        && layout
            .playable
            .iter()
            .zip(&input.buildings)
            .all(|(raw, seated)| {
                let mut expected = raw.clone();
                expected.base_elevation_metres = seated.base_elevation_metres;
                expected == *seated
            });
    let distant_matches = layout.distant.len() == input.distant_buildings.len()
        && layout
            .distant
            .iter()
            .zip(&input.distant_buildings)
            .all(|(raw, seated)| {
                let mut expected = *raw;
                expected.base_elevation_metres = seated.base_elevation_metres;
                expected == *seated
            });
    if !playable_matches
        || !distant_matches
        || layout.compounds != input.compounds
        || layout.gardens != input.gardens
        || layout.streets != input.streets
        || layout.yards != input.yards
        || layout.parishes != input.parishes
    {
        return Err("recreated programmes, membership or horizontal placement differs from the frozen input".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "layout/tests.rs"]
mod tests;
