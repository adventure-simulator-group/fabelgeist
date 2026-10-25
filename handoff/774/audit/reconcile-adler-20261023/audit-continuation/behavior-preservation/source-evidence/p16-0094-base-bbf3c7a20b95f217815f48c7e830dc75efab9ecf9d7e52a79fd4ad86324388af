//! Disease identifier serialization, parsing, and deterministic coordinates.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

/// Canonical disease vocabulary. Parsing uses serde's snake_case keys;
/// deterministic coordinates retain their separate uppercase spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiseaseId {
    Influenza,
    Dysentery,
    Typhus,
    Tetanus,
    Erysipelas,
    Smallpox,
    Plague,
    Consumption,
    Mahrdruck,
    ShroudFever,
    Bilwisschuss,
    Kobeldunst,
}

impl DiseaseId {
    /// Canonical snake_case storage and public differential key.
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Influenza => "influenza",
            Self::Dysentery => "dysentery",
            Self::Typhus => "typhus",
            Self::Tetanus => "tetanus",
            Self::Erysipelas => "erysipelas",
            Self::Smallpox => "smallpox",
            Self::Plague => "plague",
            Self::Consumption => "consumption",
            Self::Mahrdruck => "mahrdruck",
            Self::ShroudFever => "shroud_fever",
            Self::Bilwisschuss => "bilwisschuss",
            Self::Kobeldunst => "kobeldunst",
        }
    }

    /// Stable variant code used by deterministic coordinates that historically
    /// embedded the Rust variant spelling.
    pub const fn stable_variant_id(self) -> &'static str {
        match self {
            Self::Influenza => "Influenza",
            Self::Dysentery => "Dysentery",
            Self::Typhus => "Typhus",
            Self::Tetanus => "Tetanus",
            Self::Erysipelas => "Erysipelas",
            Self::Smallpox => "Smallpox",
            Self::Plague => "Plague",
            Self::Consumption => "Consumption",
            Self::Mahrdruck => "Mahrdruck",
            Self::ShroudFever => "ShroudFever",
            Self::Bilwisschuss => "Bilwisschuss",
            Self::Kobeldunst => "Kobeldunst",
        }
    }
}

/// The supplied key is outside the canonical disease vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseDiseaseIdError;

impl fmt::Display for ParseDiseaseIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Unknown disease")
    }
}

impl std::error::Error for ParseDiseaseIdError {}

impl FromStr for DiseaseId {
    type Err = ParseDiseaseIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // Reuse serde's enum vocabulary; no independent inverse table exists.
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseDiseaseIdError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_authored_disease_uses_the_same_key_in_storage_and_serialization() {
        for definition in super::super::STARTER_DISEASES {
            let id = definition.id;
            let encoded = serde_json::to_string(&id).unwrap();
            assert_eq!(
                serde_json::from_str::<String>(&encoded).unwrap(),
                id.stable_id()
            );
            assert_eq!(id.stable_id().parse(), Ok(id));
            assert_eq!(serde_json::from_str::<DiseaseId>(&encoded).unwrap(), id);
        }
    }

    #[test]
    fn malformed_and_noncanonical_keys_fail_closed() {
        for invalid in [
            "",
            "unknown",
            "Influenza",
            "ShroudFever",
            "shroudfever",
            "shroud-fever",
            " influenza",
            "influenza ",
            "\"influenza\"",
            "null",
        ] {
            assert_eq!(invalid.parse::<DiseaseId>(), Err(ParseDiseaseIdError));
        }
        assert_eq!("shroud_fever".parse(), Ok(DiseaseId::ShroudFever));
    }

    #[test]
    fn deterministic_coordinates_keep_their_existing_variant_spelling() {
        assert_eq!(DiseaseId::Influenza.stable_variant_id(), "Influenza");
        assert_eq!(DiseaseId::ShroudFever.stable_variant_id(), "ShroudFever");
        assert_ne!(
            DiseaseId::ShroudFever.stable_variant_id(),
            DiseaseId::ShroudFever.stable_id()
        );
    }
}
