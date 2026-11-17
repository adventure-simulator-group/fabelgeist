//! Attack eligibility follows explicit relationships, not unequal actor roles.

use super::TacticalCombatSide;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TacticalRelationship {
    Friendly,
    Neutral,
    Hostile,
}

impl TacticalCombatSide {
    pub const fn relationship_to(self, other: Self) -> TacticalRelationship {
        match (self, other) {
            (Self::Neutral, _) | (_, Self::Neutral) => TacticalRelationship::Neutral,
            (Self::Party, Self::Enemy) | (Self::Enemy, Self::Party) => {
                TacticalRelationship::Hostile
            }
            (Self::Party, Self::Party) | (Self::Enemy, Self::Enemy) => {
                TacticalRelationship::Friendly
            }
        }
    }

    /// Neutral residents cannot initiate combat or become unprovoked targets.
    /// A future aggression action must explicitly establish a hostile relation.
    pub const fn is_hostile_to(self, other: Self) -> bool {
        matches!(self.relationship_to(other), TacticalRelationship::Hostile)
    }
}
