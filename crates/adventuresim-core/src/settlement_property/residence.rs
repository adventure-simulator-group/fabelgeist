//! Counts and supply roles shared by physical planning and household allocation.
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// A bounded population count; an empty settlement or remainder may be zero.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ResidentCount(u32);

/// An accepted home always has space for at least one resident.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HomeCapacity(NonZeroU32);

/// One request keeps a household together without inventing character records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HouseholdRequest {
    pub household_id: String,
    pub residents: ResidentCount,
}

/// Catalogue JSON represents this role as its explicit market-reserve Boolean.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(from = "bool", into = "bool")]
pub enum HomeSupplyRole {
    PopulationHousing,
    MarketReserve,
}

impl ResidentCount {
    pub const ZERO: Self = Self(0);
    pub const fn new(count: u32) -> Self {
        Self(count)
    }
    pub const fn get(self) -> u32 {
        self.0
    }
    pub fn saturating_add_capacity(self, capacity: HomeCapacity) -> Self {
        Self(self.0.saturating_add(capacity.get()))
    }
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }
    pub fn checked_sub(self, other: Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }
}

impl std::fmt::Display for ResidentCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}

impl HomeCapacity {
    pub const fn new(capacity: NonZeroU32) -> Self {
        Self(capacity)
    }
    pub const fn get(self) -> u32 {
        self.0.get()
    }
    pub const fn residents(self) -> ResidentCount {
        ResidentCount::new(self.get())
    }
}

impl From<bool> for HomeSupplyRole {
    fn from(market_reserve: bool) -> Self {
        if market_reserve {
            Self::MarketReserve
        } else {
            Self::PopulationHousing
        }
    }
}
impl From<HomeSupplyRole> for bool {
    fn from(role: HomeSupplyRole) -> Self {
        role == HomeSupplyRole::MarketReserve
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counts_and_roles_preserve_wire_values_and_checked_capacity() {
        assert_eq!(
            serde_json::from_str::<ResidentCount>("0").unwrap(),
            ResidentCount::ZERO
        );
        assert!(serde_json::from_str::<HomeCapacity>("0").is_err());
        assert!(serde_json::from_str::<HomeCapacity>("-1").is_err());
        let capacity: HomeCapacity = serde_json::from_str("6").unwrap();
        assert_eq!(capacity.get(), 6);
        assert_eq!(serde_json::to_string(&capacity).unwrap(), "6");
        assert_eq!(
            serde_json::to_string(&HomeSupplyRole::MarketReserve).unwrap(),
            "true"
        );
        assert_eq!(
            serde_json::from_str::<HomeSupplyRole>("false").unwrap(),
            HomeSupplyRole::PopulationHousing
        );
        assert!(
            ResidentCount::new(2)
                .checked_sub(ResidentCount::new(3))
                .is_none()
        );
        assert_eq!(
            ResidentCount::new(u32::MAX)
                .saturating_add_capacity(capacity)
                .get(),
            u32::MAX
        );
    }
}
