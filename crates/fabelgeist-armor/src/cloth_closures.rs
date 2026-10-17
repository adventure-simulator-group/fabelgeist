//! Small fasteners joining the two free edges of a cloth opening.
use crate::{
    DesignError, GenerateError, Micrometers, Millimeters, Permille, TextileAppearance, TextileColor,
};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClothClosures {
    /// Ordered distances along the opening: zero at its inner end and one
    /// thousand at its free edge. Endpoints are excluded from attachment.
    pub positions: Vec<Permille>,
    pub fastener: ClothFastener,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ClothFastener {
    ButtonLoop {
        diameter: Millimeters,
        cord: Micrometers,
        fabric: TextileAppearance,
    },
    HookEye {
        length: Millimeters,
        width: Millimeters,
        wire: Micrometers,
        color: TextileColor,
        roughness: Permille,
    },
}

impl ClothClosures {
    pub const COUNT_RANGE: RangeInclusive<usize> = 1..=16;
    pub const POSITION_RANGE: RangeInclusive<u16> = 1..=999;
    pub fn validate(&self) -> Result<(), GenerateError> {
        if !Self::COUNT_RANGE.contains(&self.positions.len())
            || self
                .positions
                .iter()
                .any(|p| !Self::POSITION_RANGE.contains(&p.0))
            || self.positions.windows(2).any(|p| p[0].0 >= p[1].0)
        {
            return Err(DesignError::ParametricParameters.into());
        }
        self.fastener.validate()
    }
}

impl ClothFastener {
    pub const BUTTON_DIAMETER_RANGE: RangeInclusive<u16> = 3..=12;
    pub const CORD_RANGE: RangeInclusive<u16> = 200..=1500;
    pub const HOOK_LENGTH_RANGE: RangeInclusive<u16> = 4..=16;
    pub const HOOK_WIDTH_RANGE: RangeInclusive<u16> = 2..=10;
    pub const WIRE_RANGE: RangeInclusive<u16> = 200..=1000;
    pub const ROUGHNESS_RANGE: RangeInclusive<u16> = 80..=900;
    pub fn cord_range(diameter: Millimeters) -> RangeInclusive<u16> {
        let maximum = (u32::from(diameter.0) * Micrometers::PER_MILLIMETRE / 2).saturating_sub(1);
        *Self::CORD_RANGE.start()..=maximum.min(u32::from(*Self::CORD_RANGE.end())) as u16
    }
    pub fn wire_range(width: Millimeters) -> RangeInclusive<u16> {
        let maximum = (u32::from(width.0) * Micrometers::PER_MILLIMETRE / 2).saturating_sub(1);
        *Self::WIRE_RANGE.start()..=maximum.min(u32::from(*Self::WIRE_RANGE.end())) as u16
    }
    pub fn validate(&self) -> Result<(), GenerateError> {
        let valid = match self {
            Self::ButtonLoop {
                diameter,
                cord,
                fabric,
            } => {
                fabric.validate()?;
                Self::BUTTON_DIAMETER_RANGE.contains(&diameter.0)
                    && Self::cord_range(*diameter).contains(&cord.0)
            }
            Self::HookEye {
                length,
                width,
                wire,
                roughness,
                ..
            } => {
                Self::HOOK_LENGTH_RANGE.contains(&length.0)
                    && Self::HOOK_WIDTH_RANGE.contains(&width.0)
                    && Self::wire_range(*width).contains(&wire.0)
                    && Self::ROUGHNESS_RANGE.contains(&roughness.0)
                    && width.0 <= length.0
            }
        };
        if !valid {
            return Err(DesignError::ParametricParameters.into());
        }
        Ok(())
    }
}
