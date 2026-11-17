//! Resident identity and physical service stations for the playable area.

pub(crate) use actors::{ResidentActor, ResidentActors, ResidentProjection};
pub(crate) use common::CommonResidentStations;
pub(crate) use placement::OperatorStations;

mod actors;
mod common;
mod lifecycle;
mod placement;
mod reachability;

pub(crate) fn install(app: &mut bevy::prelude::App) {
    use bevy::prelude::*;
    app.init_resource::<ResidentActors>().add_systems(
        Update,
        lifecycle::reconcile
            .after(crate::player_projection::PlayerProjectionSet::Spawn)
            .after(crate::stdb::process_scene_operations)
            .run_if(resource_exists::<crate::settlement_session::SettlementSession>)
            .run_if(resource_exists::<crate::stdb::SceneReady>),
    );
}
