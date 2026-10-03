//! Fixed puzzle choices and their canonical submission vocabulary.

use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WitnessPath {
    Ash,
    Moon,
    Thorn,
}

impl WitnessPath {
    pub const ALL: [Self; 3] = [Self::Ash, Self::Moon, Self::Thorn];

    /// Canonical submission key, independent of the visible label.
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Ash => "Ash",
            Self::Moon => "Moon",
            Self::Thorn => "Thorn",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ash => "Ash path",
            Self::Moon => "Moon path",
            Self::Thorn => "Thorn path",
        }
    }
}

/// The supplied key is outside the canonical WitnessPath vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseWitnessPathError;

impl fmt::Display for ParseWitnessPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Choose one of the named paths")
    }
}

impl std::error::Error for ParseWitnessPathError {}

impl FromStr for WitnessPath {
    type Err = ParseWitnessPathError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseWitnessPathError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offered_choices_round_trip_through_submission_and_serde() {
        for choice in WitnessPath::ALL {
            assert_eq!(choice.stable_id().parse(), Ok(choice));
            let json = serde_json::to_string(&choice).unwrap();
            assert_eq!(json, format!("\"{}\"", choice.stable_id()));
            assert_eq!(serde_json::from_str::<WitnessPath>(&json).unwrap(), choice);
        }
    }

    #[test]
    fn malformed_choices_are_rejected() {
        for value in ["", "unknown", "moon", "Moon path", " Moon"] {
            assert_eq!(value.parse::<WitnessPath>(), Err(ParseWitnessPathError));
        }
    }
}
