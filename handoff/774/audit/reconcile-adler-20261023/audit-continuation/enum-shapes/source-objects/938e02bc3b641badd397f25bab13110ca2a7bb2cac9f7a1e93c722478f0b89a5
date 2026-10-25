//! Fixed puzzle choices and their canonical submission vocabulary.

use crate::ORDERED_SIGIL_COUNT;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Sigil {
    Crown,
    Hart,
    Moon,
    Rose,
    Sword,
}

impl Sigil {
    pub const ALL: [Self; ORDERED_SIGIL_COUNT] =
        [Self::Crown, Self::Hart, Self::Moon, Self::Rose, Self::Sword];

    /// Canonical submission key, independent of the visible label.
    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Crown => "Crown",
            Self::Hart => "Hart",
            Self::Moon => "Moon",
            Self::Rose => "Rose",
            Self::Sword => "Sword",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Crown => "Crown",
            Self::Hart => "Hart",
            Self::Moon => "Moon",
            Self::Rose => "Rose",
            Self::Sword => "Sword",
        }
    }
}

/// The supplied key is outside the canonical Sigil vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseSigilError;

impl fmt::Display for ParseSigilError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Choose one of the named sigils")
    }
}

impl std::error::Error for ParseSigilError {}

impl FromStr for Sigil {
    type Err = ParseSigilError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value))
            .map_err(|_| ParseSigilError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offered_choices_round_trip_through_submission_and_serde() {
        for choice in Sigil::ALL {
            assert_eq!(choice.stable_id().parse(), Ok(choice));
            let json = serde_json::to_string(&choice).unwrap();
            assert_eq!(json, format!("\"{}\"", choice.stable_id()));
            assert_eq!(serde_json::from_str::<Sigil>(&json).unwrap(), choice);
        }
    }

    #[test]
    fn malformed_choices_are_rejected() {
        for value in ["", "unknown", "moon", "Moon path", " Moon"] {
            assert_eq!(value.parse::<Sigil>(), Err(ParseSigilError));
        }
    }
}
