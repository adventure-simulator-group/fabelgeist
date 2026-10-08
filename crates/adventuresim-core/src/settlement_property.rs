//! Physical home identity and bounded household allocation, independent of UI.
use fabelgeist_determinism::Seed;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{courtship::HousingTier, strategic_place::StrategicIdentityComponent};

pub use residence::{HomeCapacity, HomeSupplyRole, HouseholdRequest, ResidentCount};

mod residence;

const MAX_PROPERTY_CATALOG_BYTES: usize = 4 * 1024 * 1024;

/// A settlement-scoped front building. Rear stores are parts of its property,
/// not extra homes. Identity does not include a player, household or tenancy.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PropertyId(String);

/// One bounded vacant home in each price tier, reserved before housing the
/// settlement population. This is explicit game supply, not a vacancy estimate.
pub struct HousingMarketReserve(Vec<HousingTier>);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedHome {
    pub id: PropertyId,
    pub building_id: u64,
    pub tier: HousingTier,
    pub resident_capacity: HomeCapacity,
    #[serde(rename = "market_reserve")]
    pub supply_role: HomeSupplyRole,
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
    pub population: ResidentCount,
    pub homes: Vec<GeneratedHome>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseholdHome {
    pub household_id: String,
    pub property_id: PropertyId,
    pub residents: ResidentCount,
}

struct AvailableHome<'a> {
    home: &'a GeneratedHome,
    free: ResidentCount,
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

impl Default for HousingMarketReserve {
    fn default() -> Self {
        Self(HousingTier::ALL.to_vec())
    }
}

impl HousingMarketReserve {
    pub fn reserve(&mut self, tier: HousingTier) -> HomeSupplyRole {
        if let Some(index) = self.0.iter().position(|candidate| *candidate == tier) {
            self.0.remove(index);
            HomeSupplyRole::MarketReserve
        } else {
            HomeSupplyRole::PopulationHousing
        }
    }

    pub fn complete(&self) -> bool {
        self.0.is_empty()
    }
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

    pub fn validate(
        &self,
        settlement: &str,
        population: ResidentCount,
    ) -> Result<(), PropertyError> {
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
            if home.supply_role != supply.reserve(home.tier) {
                return Err(PropertyError::InvalidCatalog);
            }
            if home.supply_role == HomeSupplyRole::PopulationHousing {
                capacity += u64::from(home.resident_capacity.get());
            }
        }
        if !supply.complete() || capacity < u64::from(population.get()) {
            return Err(PropertyError::InsufficientCapacity);
        }
        Ok(())
    }

    /// Known families are allocated as groups. Remaining residents are bounded
    /// household counts, with no invented character rows or membership records.
    pub fn allocate(
        &self,
        families: &[HouseholdRequest],
    ) -> Result<Vec<HouseholdHome>, PropertyError> {
        self.validate(&self.settlement_id, self.population)?;
        let mut family_ids = BTreeSet::new();
        if families.iter().any(|request| {
            request.residents == ResidentCount::ZERO
                || request.household_id.is_empty()
                || !family_ids.insert(&request.household_id)
        }) {
            return Err(PropertyError::InvalidCatalog);
        }
        let known: u64 = families
            .iter()
            .map(|request| u64::from(request.residents.get()))
            .sum();
        if known > u64::from(self.population.get()) {
            return Err(PropertyError::HouseholdPopulation);
        }
        let mut remaining = self
            .homes
            .iter()
            .filter(|home| home.supply_role == HomeSupplyRole::PopulationHousing)
            .map(|home| AvailableHome {
                home,
                free: home.resident_capacity.residents(),
            })
            .collect::<Vec<_>>();
        let mut allocated = Vec::new();
        for request in families {
            let Some(available) = remaining
                .iter_mut()
                .find(|available| available.free >= request.residents)
            else {
                return Err(PropertyError::InsufficientCapacity);
            };
            available.free = available
                .free
                .checked_sub(request.residents)
                .ok_or(PropertyError::InsufficientCapacity)?;
            allocated.push(HouseholdHome {
                household_id: request.household_id.clone(),
                property_id: available.home.id.clone(),
                residents: request.residents,
            });
        }
        let mut abstract_population = self
            .population
            .checked_sub(ResidentCount::new(
                u32::try_from(known).map_err(|_| PropertyError::HouseholdPopulation)?,
            ))
            .ok_or(PropertyError::HouseholdPopulation)?;
        for AvailableHome { home, free } in remaining {
            let residents = free.min(abstract_population);
            if residents == ResidentCount::ZERO {
                continue;
            }
            allocated.push(HouseholdHome {
                household_id: format!("household:population:{}", home.id.as_str()),
                property_id: home.id.clone(),
                residents,
            });
            abstract_population = abstract_population
                .checked_sub(residents)
                .ok_or(PropertyError::HouseholdPopulation)?;
        }
        if abstract_population > ResidentCount::ZERO {
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

#[cfg(test)]
mod tests;
