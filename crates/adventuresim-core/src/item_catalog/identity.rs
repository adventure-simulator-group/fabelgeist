//! Exact item-definition identity, independent of catalog membership or custody.

use serde::{Deserialize, Serialize};

/// A catalog or persisted item-definition key, preserving its exact spelling.
///
/// Constructing an identity does not assert that the definition exists. Lookup
/// retains that separate check, including for unknown or empty native keys.
///
/// ```compile_fail
/// use adventuresim_core::{identity::CharacterId, item_catalog::definition};
/// definition(&CharacterId::from(7));
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemDefinitionId(String);

impl ItemDefinitionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ItemDefinitionId {
    fn from(encoded: &str) -> Self {
        Self(encoded.to_owned())
    }
}

impl From<String> for ItemDefinitionId {
    fn from(encoded: String) -> Self {
        Self(encoded)
    }
}

impl From<&String> for ItemDefinitionId {
    fn from(encoded: &String) -> Self {
        Self(encoded.clone())
    }
}

impl std::fmt::Display for ItemDefinitionId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_identity_round_trips_without_normalizing_or_claiming_membership() {
        for encoded in ["", "sage", "Sage", "sage ", "薬草", "merchant's tonic"] {
            let id = ItemDefinitionId::from(encoded);
            assert_eq!(id.as_str(), encoded);
            assert_eq!(id.to_string(), encoded);
            let json = serde_json::to_value(&id).unwrap();
            assert_eq!(json, serde_json::json!(encoded));
            assert_eq!(
                serde_json::from_value::<ItemDefinitionId>(json).unwrap(),
                id
            );
        }
        assert!(super::super::definition(&("sage").into()).is_some());
        for encoded in ["", "Sage", "sage ", "unknown_item"] {
            assert!(super::super::definition(&(encoded).into()).is_none());
        }
    }
}
