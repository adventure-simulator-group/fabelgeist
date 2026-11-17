//! Identities shared by combat missions and ordinary live settlement scenes.

use std::{fmt, str::FromStr};

use thiserror::Error;

use crate::{mission::MissionId, strategic_place::StrategicIdentityComponent};

pub use visit_id::{LocalSceneVisitId, LocalSceneVisitIdentityError};

mod visit_id;

const MISSION_INSTANCE_PREFIX: &str = "scene:mission:";
const SETTLEMENT_INSTANCE_PREFIX: &str = "scene:settlement:";

/// The unique serving slot for one mission or a settlement's central area.
/// A server identity distinguishes successive runtime instances of this slot.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TacticalInstanceId {
    Mission(MissionId),
    Settlement(StrategicIdentityComponent),
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TacticalInstanceIdentityError {
    #[error("unknown tactical instance kind")]
    UnknownKind,
    #[error("invalid mission in tactical instance identity")]
    InvalidMission,
    #[error("invalid settlement in tactical instance identity")]
    InvalidSettlement,
}

impl TacticalInstanceId {
    pub fn for_mission(mission: &MissionId) -> Self {
        Self::Mission(mission.clone())
    }

    pub fn for_settlement(settlement: &StrategicIdentityComponent) -> Self {
        Self::Settlement(settlement.clone())
    }

    pub fn into_string(self) -> String {
        self.to_string()
    }
}

impl FromStr for TacticalInstanceId {
    type Err = TacticalInstanceIdentityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Some(mission) = value.strip_prefix(MISSION_INSTANCE_PREFIX) {
            return MissionId::new(mission)
                .map(|mission| Self::for_mission(&mission))
                .map_err(|_| TacticalInstanceIdentityError::InvalidMission);
        }
        if let Some(settlement) = value.strip_prefix(SETTLEMENT_INSTANCE_PREFIX) {
            return StrategicIdentityComponent::try_new(settlement)
                .map(|settlement| Self::for_settlement(&settlement))
                .map_err(|_| TacticalInstanceIdentityError::InvalidSettlement);
        }
        Err(TacticalInstanceIdentityError::UnknownKind)
    }
}

impl fmt::Display for TacticalInstanceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mission(mission) => write!(formatter, "{MISSION_INSTANCE_PREFIX}{mission}"),
            Self::Settlement(settlement) => write!(
                formatter,
                "{SETTLEMENT_INSTANCE_PREFIX}{}",
                settlement.as_str()
            ),
        }
    }
}
