//! Materialize a canonical scene from purpose-specific captured authority.

use super::*;

pub(crate) fn materialize_requested_scene(
    terrain: &TerrainPack,
    directory: &Path,
    request: &TacticalServerRequest,
) -> Result<PathBuf, String> {
    let snapshot = match &request.session {
        adventuresim_stdb_client::TacticalSession::Mission(mission) => mission.settlement.as_ref(),
        adventuresim_stdb_client::TacticalSession::Settlement(settlement) => Some(settlement),
    };
    let profile = snapshot
        .map(|settlement| -> Result<SettlementSceneProfile, String> {
            Ok(SettlementSceneProfile {
                id: settlement.id.clone(),
                population_level: settlement.population_level,
                population_estimate: settlement.population_estimate,
                economy: settlement_economy_adapter::economy_profile(&settlement.economy),
                operators: settlement
                    .operators
                    .iter()
                    .map(|operator| {
                        let operator_name =
                            adventuresim_world_schema::person_names::RenderedPersonalName::try_from(
                                operator.operator_name.clone(),
                            )
                            .map_err(|error| format!("invalid operator name: {error}"))?;
                        Ok(SettlementBusinessOperatorProfile {
                            business_id:
                                adventuresim_world_schema::settlement_buildings::BusinessId::new(
                                    &operator.business_id.settlement_id,
                                    adventuresim_world_schema::settlement_buildings::BusinessKey {
                                        usage: settlement_economy_adapter::building_use(
                                            &operator.business_id.key.usage,
                                        ),
                                        ordinal: operator.business_id.key.ordinal,
                                    },
                                ),
                            operator_character_id: adventuresim_tactical_core::player::CharacterId(
                                operator.operator_character_id,
                            ),
                            operator_name,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?,
            })
        })
        .transpose()?;
    let input = scene_input::build_imported_scene(
        terrain,
        &request.instance_id,
        &request.scene_key,
        request.latitude_e_7,
        request.longitude_e_7,
        adventuresim_world_schema::calendar::StrategicMinute::new(request.absolute_minute.minutes),
        adventuresim_world_schema::calendar::StrategicMinute::new(
            request.lunar_phase_minute.minutes,
        ),
        profile.as_ref(),
    )?;
    scene_input::materialize_scene_input(directory, &request.instance_id, &input)
}
