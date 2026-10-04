//! Bounded numeric values carried by compiled water and route records.
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldValueError {
    WaterDistance(u16),
    Strahler(u8),
    EdgeProgress(u16),
}
impl std::fmt::Display for WorldValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WaterDistance(value) => write!(
                f,
                "water distance {value} exceeds {} meters",
                WaterDistanceMeters::MAX
            ),
            Self::Strahler(value) => write!(
                f,
                "Strahler order {value} is outside 1..={}",
                StrahlerOrder::MAX
            ),
            Self::EdgeProgress(value) => write!(
                f,
                "edge progress {value} exceeds {}",
                EdgeProgressPermille::MAX
            ),
        }
    }
}
impl std::error::Error for WorldValueError {}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct WaterDistanceMeters {
    meters: u16,
}

impl WaterDistanceMeters {
    pub const MAX: u16 = 10_000;

    pub fn new(meters: u16) -> Result<Self, WorldValueError> {
        if meters <= Self::MAX {
            Ok(Self { meters })
        } else {
            Err(WorldValueError::WaterDistance(meters))
        }
    }

    pub const fn get(self) -> u16 {
        self.meters
    }
}

impl<'de> Deserialize<'de> for WaterDistanceMeters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            meters: u16,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.meters).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<u16> for WaterDistanceMeters {
    type Error = WorldValueError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
#[cfg(feature = "spacetimedb")]
crate::checked_sats::checked_numeric_product!(WaterDistanceMeters, meters: u16);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct StrahlerOrder {
    order: u8,
}

impl StrahlerOrder {
    pub const MAX: u8 = 12;

    pub fn new(order: u8) -> Result<Self, WorldValueError> {
        if (1..=Self::MAX).contains(&order) {
            Ok(Self { order })
        } else {
            Err(WorldValueError::Strahler(order))
        }
    }

    pub const fn get(self) -> u8 {
        self.order
    }
}

impl<'de> Deserialize<'de> for StrahlerOrder {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            order: u8,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.order).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<u8> for StrahlerOrder {
    type Error = WorldValueError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
#[cfg(feature = "spacetimedb")]
crate::checked_sats::checked_numeric_product!(StrahlerOrder, order: u8);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct EdgeProgressPermille {
    permille: u16,
}

impl EdgeProgressPermille {
    pub const MAX: u16 = 1_000;

    pub fn new(value: u16) -> Result<Self, WorldValueError> {
        if value <= Self::MAX {
            Ok(Self { permille: value })
        } else {
            Err(WorldValueError::EdgeProgress(value))
        }
    }

    pub const fn get(self) -> u16 {
        self.permille
    }
}

impl<'de> Deserialize<'de> for EdgeProgressPermille {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wire {
            permille: u16,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.permille).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<u16> for EdgeProgressPermille {
    type Error = WorldValueError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
#[cfg(feature = "spacetimedb")]
crate::checked_sats::checked_numeric_product!(EdgeProgressPermille, permille: u16);

#[cfg(all(test, feature = "spacetimedb"))]
mod tests {
    use super::*;
    #[test]
    fn database_decoding_obeys_water_and_progress_bounds() {
        use spacetimedb_lib::bsatn;
        assert!(
            bsatn::from_slice::<WaterDistanceMeters>(&bsatn::to_vec(&10_001_u16).unwrap()).is_err()
        );
        assert!(
            bsatn::from_slice::<EdgeProgressPermille>(&bsatn::to_vec(&1_001_u16).unwrap()).is_err()
        );
        for invalid in [0_u8, 13] {
            assert!(bsatn::from_slice::<StrahlerOrder>(&bsatn::to_vec(&invalid).unwrap()).is_err());
        }
        let raw = bsatn::to_vec(&12_u8).unwrap();
        let checked = bsatn::from_slice::<StrahlerOrder>(&raw).unwrap();
        assert_eq!(bsatn::to_vec(&checked).unwrap(), raw);
    }
}
