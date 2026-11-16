//! Purpose-specific authority captured for a tactical scene's serving slot.

use adventuresim_core::{
    mission::{MissionId, TacticalPartyRoster},
    strategic_place::StrategicIdentityComponent,
    tactical_session::TacticalInstanceId,
};
use spacetimedb::SpacetimeType;

use super::{TacticalMissionResolution, TacticalSettlementSnapshot};

#[derive(Clone, Debug, SpacetimeType)]
pub struct TacticalMissionSession {
    pub mission_id: String,
    pub party_id: String,
    pub authorized_party_member_ids: Vec<u64>,
    pub required_enemy_kills: u32,
    pub enemy_difficulty: i32,
    pub enemy_combat_scale_bps: u32,
    pub countermeasure_multiplier_bps: u32,
    pub normalized_combat_power: u32,
    pub enemy_character_ids: Vec<u64>,
    pub party_has_surprise: bool,
    pub settlement: Option<TacticalSettlementSnapshot>,
}

/// Combat authority belongs only to mission sessions. Settlement sessions have
/// no enemy roster, party enrollment snapshot or terminal combat outcome.
#[derive(Clone, Debug, SpacetimeType)]
pub enum TacticalSession {
    Mission(TacticalMissionSession),
    Settlement(TacticalSettlementSnapshot),
}

/// Provisioning failures are operational state, independent of combat results.
#[derive(Clone, Copy, Debug, PartialEq, Eq, SpacetimeType)]
pub enum TacticalProvisioningStatus {
    Pending,
    Failed,
}

impl TacticalMissionResolution {
    pub(crate) fn succeeded(self) -> bool {
        match self {
            Self::Failed | Self::CaptureTargetKilled => false,
            Self::Defeated | Self::DrivenOff | Self::Captured => true,
        }
    }
}

impl TacticalSession {
    pub fn mission(&self) -> Option<&TacticalMissionSession> {
        match self {
            Self::Mission(mission) => Some(mission),
            Self::Settlement(_) => None,
        }
    }

    pub fn instance_id(&self) -> Result<TacticalInstanceId, String> {
        match self {
            Self::Mission(mission) => MissionId::new(mission.mission_id.clone())
                .map(|id| TacticalInstanceId::for_mission(&id))
                .map_err(str::to_owned),
            Self::Settlement(settlement) => StrategicIdentityComponent::try_new(&settlement.id)
                .map(|id| TacticalInstanceId::for_settlement(&id))
                .map_err(|error| error.to_string()),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.instance_id()?;
        if let Self::Mission(mission) = self {
            TacticalPartyRoster::try_from(mission.authorized_party_member_ids.clone())
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
}
