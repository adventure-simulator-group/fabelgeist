use super::{BTreeSet, PresenceBridge};

/// Causal admission requirement for a population outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BridgeRequirement {
    Unrestricted,
    Required(PresenceBridge),
}
impl BridgeRequirement {
    pub(super) fn is_satisfied_by(self, available: &BTreeSet<PresenceBridge>) -> bool {
        match self {
            Self::Unrestricted => true,
            Self::Required(bridge) => available.contains(&bridge),
        }
    }
    pub(super) const fn required_bridge(self) -> Option<PresenceBridge> {
        match self {
            Self::Unrestricted => None,
            Self::Required(bridge) => Some(bridge),
        }
    }
}
