//! Flared cuff hanging from the distal band of a sleeve over the hand.
use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::{DesignError, GenerateError, Millimeters, TextileAppearance};

/// The cuff continues past the wrist along the forearm axis. It encloses the
/// unposed hand section at its free edge; it is not fitted to the fingers.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SleeveCuff {
    pub length: Millimeters,
    /// Outward turn of the free edge beyond the enclosed hand section.
    pub flare: Millimeters,
    pub fabric: TextileAppearance,
}

impl SleeveCuff {
    pub const LENGTH_RANGE: RangeInclusive<u16> = 20..=120;
    pub const FLARE_RANGE: RangeInclusive<u16> = 0..=60;

    pub(crate) fn validate(&self) -> Result<(), GenerateError> {
        self.fabric.validate()?;
        if !Self::LENGTH_RANGE.contains(&self.length.0)
            || !Self::FLARE_RANGE.contains(&self.flare.0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
