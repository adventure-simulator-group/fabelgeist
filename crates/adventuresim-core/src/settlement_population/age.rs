//! Shared resident age classification from current character chronology.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub enum AgeBand {
    Child,
    Adolescent,
    Adult,
    Elder,
}

impl AgeBand {
    /// Classify the current age owned by character chronology.
    pub const fn for_years(age_years: u16) -> Self {
        match age_years {
            0..=12 => Self::Child,
            13..=17 => Self::Adolescent,
            18..=59 => Self::Adult,
            _ => Self::Elder,
        }
    }

    pub const fn stable_id(self) -> &'static str {
        match self {
            Self::Child => "child",
            Self::Adolescent => "adolescent",
            Self::Adult => "adult",
            Self::Elder => "elder",
        }
    }

    pub const fn stable_variant_id(self) -> &'static str {
        match self {
            Self::Child => "Child",
            Self::Adolescent => "Adolescent",
            Self::Adult => "Adult",
            Self::Elder => "Elder",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chronology_crosses_each_resident_band_at_its_boundary() {
        for (age, band) in [
            (0, AgeBand::Child),
            (12, AgeBand::Child),
            (13, AgeBand::Adolescent),
            (17, AgeBand::Adolescent),
            (18, AgeBand::Adult),
            (59, AgeBand::Adult),
            (60, AgeBand::Elder),
            (u16::MAX, AgeBand::Elder),
        ] {
            assert_eq!(AgeBand::for_years(age), band);
        }
    }
}
