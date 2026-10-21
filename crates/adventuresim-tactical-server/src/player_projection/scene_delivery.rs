//! Deliver scene assets only to an admitted session, after accepted enrollment.

use adventuresim_tactical_core::prelude::TacticalCombatConfig;
use adventuresim_tactical_netcode::{
    aeronet::{io::Session, transport::TransportConfig},
    bevy_replicon::prelude::{ClientId, SendTargets, ServerTriggerExt, ToClients},
    prelude::TacticalCombatConfigSnapshot,
    transport_policy::tactical_asset_transport_ready,
};
use bevy::{ecs::system::SystemParam, prelude::*};

use crate::SceneVistaBundleResource;

#[derive(SystemParam)]
pub(crate) struct SceneJoinDelivery<'w, 's> {
    sessions: Query<'w, 's, &'static TransportConfig, With<Session>>,
    vista: Res<'w, SceneVistaBundleResource>,
    combat: Res<'w, TacticalCombatConfig>,
}

impl SceneJoinDelivery<'_, '_> {
    /// Validate the actual endpoint before changing mission enrollment state.
    pub(super) fn admit_transport(&self, client: ClientId) -> Result<Option<Entity>> {
        let Some(entity) = client.entity() else {
            return Ok(None);
        };
        if !tactical_asset_transport_ready(self.sessions.get(entity)?) {
            return Err("tactical session has no admitted scene asset budget".into());
        }
        Ok(Some(entity))
    }

    pub(super) fn send_to(&self, commands: &mut Commands, client: ClientId) {
        if let Some(message) = &self.vista.0 {
            commands.server_trigger(ToClients {
                targets: SendTargets::Single(client),
                message: message.clone(),
            });
        }
        commands.server_trigger(ToClients {
            targets: SendTargets::Single(client),
            message: TacticalCombatConfigSnapshot(self.combat.clone()),
        });
        #[cfg(feature = "grown-trees")]
        commands.trigger(crate::scene_tree_assets::RequestSceneTreeBank { client });
    }
}
