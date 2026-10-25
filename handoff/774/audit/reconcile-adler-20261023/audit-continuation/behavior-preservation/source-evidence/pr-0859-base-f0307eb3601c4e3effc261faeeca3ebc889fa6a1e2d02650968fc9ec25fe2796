//! Classifications shared by combat scheduling and immutable replay records.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleAttackKind {
    Melee,
    Ranged,
}

impl std::fmt::Display for BattleAttackKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Melee => "melee",
            Self::Ranged => "ranged",
        })
    }
}

/// Requested movement; realized displacement and velocity are separate telemetry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeleeMovementAction {
    Close,
    Retreat,
    Hold,
}

impl MeleeMovementAction {
    pub(super) fn target_velocity(self, maximum_speed_metres_per_second: f32) -> f32 {
        match self {
            Self::Close => -maximum_speed_metres_per_second,
            Self::Hold => 0.0,
            Self::Retreat => maximum_speed_metres_per_second,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_classifications_keep_their_wire_vocabulary() {
        for (kind, expected) in [
            (BattleAttackKind::Melee, "melee"),
            (BattleAttackKind::Ranged, "ranged"),
        ] {
            assert_eq!(serde_json::to_value(kind).unwrap(), expected);
            assert_eq!(kind.to_string(), expected);
        }
        for (action, expected) in [
            (MeleeMovementAction::Close, "close"),
            (MeleeMovementAction::Retreat, "retreat"),
            (MeleeMovementAction::Hold, "hold"),
        ] {
            assert_eq!(serde_json::to_value(action).unwrap(), expected);
        }
    }
}
