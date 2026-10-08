//! Physical home identity and bounded household allocation, independent of UI.
use fabelgeist_determinism::Seed;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{courtship::HousingTier, strategic_place::StrategicIdentityComponent};

const MAX_PROPERTY_CATALOG_BYTES: usize = 4 * 1024 * 1024;

/// A settlement-scoped front building. Rear stores are parts of its property,
/// not extra homes. Identity does not include a player, household or tenancy.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PropertyId(String);

impl PropertyId {
    pub fn new(settlement: &str, building_id: u64) -> Result<Self, PropertyError> {
        StrategicIdentityComponent::try_new(settlement)
            .map_err(|_| PropertyError::InvalidIdentity)?;
        if building_id == 0 {
            return Err(PropertyError::InvalidIdentity);
        }
        Ok(Self(format!(
            "property:{}:{settlement}:{building_id}",
            settlement.len()
        )))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One bounded vacant home in each price tier, reserved before housing the
/// settlement population. This is explicit game supply, not a vacancy estimate.
pub struct HousingMarketReserve(Vec<HousingTier>);

impl Default for HousingMarketReserve {
    fn default() -> Self {
        Self(HousingTier::ALL.to_vec())
    }
}

impl HousingMarketReserve {
    pub fn reserve(&mut self, tier: HousingTier) -> bool {
        if let Some(index) = self.0.iter().position(|candidate| *candidate == tier) {
            self.0.remove(index);
            true
        } else {
            false
        }
    }

    pub fn complete(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedHome {
    pub id: PropertyId,
    pub building_id: u64,
    pub tier: HousingTier,
    pub resident_capacity: u32,
    pub market_reserve: bool,
    pub east_metres: f32,
    pub north_metres: f32,
    pub yaw_radians: f32,
    pub width_metres: f32,
    pub depth_metres: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedHomeCatalog {
    pub settlement_id: String,
    pub seed: Seed,
    pub population: u32,
    pub homes: Vec<GeneratedHome>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PropertyError {
    #[error("invalid generated property identity")]
    InvalidIdentity,
    #[error("generated property catalog is malformed or too large")]
    InvalidCatalog,
    #[error("generated property scope or population does not match the settlement")]
    SettlementMismatch,
    #[error("generated homes cannot house the settlement population")]
    InsufficientCapacity,
    #[error("household population exceeds the settlement population")]
    HouseholdPopulation,
}

impl GeneratedHomeCatalog {
    pub fn digest(&self) -> Result<String, PropertyError> {
        use sha2::{Digest, Sha256};
        let bytes = serde_json::to_vec(self).map_err(|_| PropertyError::InvalidCatalog)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub fn parse(json: &str) -> Result<Self, PropertyError> {
        if json.len() > MAX_PROPERTY_CATALOG_BYTES {
            return Err(PropertyError::InvalidCatalog);
        }
        serde_json::from_str(json).map_err(|_| PropertyError::InvalidCatalog)
    }

    pub fn validate(&self, settlement: &str, population: u32) -> Result<(), PropertyError> {
        if self.settlement_id != settlement
            || self.population != population
            || self.seed != crate::settlement_population::settlement_building_seed(settlement)
        {
            return Err(PropertyError::SettlementMismatch);
        }
        let mut identities = BTreeSet::new();
        let mut supply = HousingMarketReserve::default();
        let mut capacity = 0_u64;
        for home in &self.homes {
            if home.id != PropertyId::new(settlement, home.building_id)?
                || !identities.insert(&home.id)
                || home.resident_capacity == 0
                || ![
                    home.east_metres,
                    home.north_metres,
                    home.yaw_radians,
                    home.width_metres,
                    home.depth_metres,
                ]
                .into_iter()
                .all(f32::is_finite)
                || home.width_metres <= 0.0
                || home.depth_metres <= 0.0
            {
                return Err(PropertyError::InvalidCatalog);
            }
            if home.market_reserve != supply.reserve(home.tier) {
                return Err(PropertyError::InvalidCatalog);
            }
            if !home.market_reserve {
                capacity += u64::from(home.resident_capacity);
            }
        }
        if !supply.complete() || capacity < u64::from(population) {
            return Err(PropertyError::InsufficientCapacity);
        }
        Ok(())
    }

    /// Known families are allocated as groups. Remaining residents are bounded
    /// household counts, with no invented character rows or membership records.
    pub fn allocate(
        &self,
        families: &[(String, u32)],
    ) -> Result<Vec<HouseholdHome>, PropertyError> {
        self.validate(&self.settlement_id, self.population)?;
        let mut family_ids = BTreeSet::new();
        if families
            .iter()
            .any(|(id, count)| *count == 0 || id.is_empty() || !family_ids.insert(id))
        {
            return Err(PropertyError::InvalidCatalog);
        }
        let known: u64 = families.iter().map(|(_, count)| u64::from(*count)).sum();
        if known > u64::from(self.population) {
            return Err(PropertyError::HouseholdPopulation);
        }
        let mut remaining = self
            .homes
            .iter()
            .filter(|home| !home.market_reserve)
            .map(|home| (home, home.resident_capacity))
            .collect::<Vec<_>>();
        let mut allocated = Vec::new();
        for (household_id, count) in families {
            let Some((home, free)) = remaining.iter_mut().find(|(_, free)| *free >= *count) else {
                return Err(PropertyError::InsufficientCapacity);
            };
            *free -= count;
            allocated.push(HouseholdHome {
                household_id: household_id.clone(),
                property_id: home.id.clone(),
                residents: *count,
            });
        }
        let mut abstract_population = self.population - known as u32;
        for (home, free) in remaining {
            let residents = free.min(abstract_population);
            if residents == 0 {
                continue;
            }
            allocated.push(HouseholdHome {
                household_id: format!("household:population:{}", home.id.as_str()),
                property_id: home.id.clone(),
                residents,
            });
            abstract_population -= residents;
        }
        if abstract_population > 0 {
            return Err(PropertyError::InsufficientCapacity);
        }
        if allocated
            .iter()
            .map(|home| &home.household_id)
            .collect::<BTreeSet<_>>()
            .len()
            != allocated.len()
        {
            return Err(PropertyError::InvalidCatalog);
        }
        Ok(allocated)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseholdHome {
    pub household_id: String,
    pub property_id: PropertyId,
    pub residents: u32,
}

#[cfg(test)]
mod tests;
