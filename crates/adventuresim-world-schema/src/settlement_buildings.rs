//! Building demand from the canonical economy, without a second service lottery.
//!
//! Capacities are gameplay catchments, not historical attendance or occupancy.
use crate::SettlementEconomyProfile;
use fabelgeist_determinism::{Seed, StreamId};

mod catalog;
mod parish;
pub use catalog::{BuildingDefinition, BuildingDistrict, BuildingEligibility, BuildingUse};
pub use parish::{
    AuthoredParishPolicy, ChurchBuildingScale, ParishBuildingRole, ParishId, ParishPopulation,
    ParishProgramme, ParishProminence,
};
use serde::{Deserialize, Serialize};

const CAPACITY_DOMAIN: StreamId = StreamId::new("settlement.building-capacity");
pub const MAX_SERVICE_BUILDINGS: usize = 4_096;

/// Settlement-local coordinate of one economic service establishment.
///
/// This value is safe inside a settlement generation pass. Persisted and
/// transported state must use [`BusinessId`], which supplies the settlement
/// scope explicitly.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct BusinessKey {
    pub usage: BuildingUse,
    pub ordinal: u32,
}

/// Globally scoped identity of one generated economic service establishment.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[cfg_attr(feature = "spacetimedb", derive(spacetimedb::SpacetimeType))]
pub struct BusinessId {
    pub settlement_id: String,
    pub key: BusinessKey,
}

impl BusinessId {
    pub fn new(settlement_id: impl Into<String>, key: BusinessKey) -> Self {
        Self {
            settlement_id: settlement_id.into(),
            key,
        }
    }
}

/// Approximate people served, distinct from residents housed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceCapacity(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityRange {
    pub minimum: ServiceCapacity,
    pub maximum: ServiceCapacity,
}

impl CapacityRange {
    pub const fn new(minimum: u32, maximum: u32) -> Self {
        assert!(minimum > 0 && minimum <= maximum);
        Self {
            minimum: ServiceCapacity(minimum),
            maximum: ServiceCapacity(maximum),
        }
    }

    fn sample(self, seed: Seed) -> ServiceCapacity {
        let width = u64::from(self.maximum.0) - u64::from(self.minimum.0) + 1;
        ServiceCapacity(
            self.minimum.0
                + seed
                    .rng()
                    .below(std::num::NonZeroU64::new(width).expect("nonempty capacity range"))
                    as u32,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildingDemandPolicy {
    ServiceCatchment(CapacityRange),
    SettlementInstitution(CivicInstitution),
    ParishChurch,
    ParishResidence,
    ParishSchool,
    EvidenceOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CivicInstitution {
    TownHall,
    WeighHouse,
    Prison,
}

impl CivicInstitution {
    pub const fn usage(self) -> BuildingUse {
        match self {
            Self::TownHall => BuildingUse::TownHall,
            Self::WeighHouse => BuildingUse::WeighHouse,
            Self::Prison => BuildingUse::Prison,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BuildingDemand {
    Service {
        usage: BuildingUse,
        ordinal: u32,
        capacity: ServiceCapacity,
    },
    Civic(CivicInstitution),
    Parish {
        parish: ParishId,
        role: ParishBuildingRole,
    },
}

impl BuildingDemand {
    pub const fn business_key(self) -> Option<BusinessKey> {
        match self {
            Self::Service { usage, ordinal, .. } => Some(BusinessKey { usage, ordinal }),
            Self::Civic(_) | Self::Parish { .. } => None,
        }
    }

    pub const fn usage(self) -> BuildingUse {
        match self {
            Self::Service { usage, .. } => usage,
            Self::Civic(institution) => institution.usage(),
            Self::Parish { role, .. } => role.usage(),
        }
    }

    pub const fn ordinal(self) -> u32 {
        match self {
            Self::Service { ordinal, .. } => ordinal,
            Self::Civic(_) => 0,
            Self::Parish { parish, .. } => parish.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DemandShortfall {
    Service {
        usage: BuildingUse,
        capacity: ServiceCapacity,
    },
    Civic(CivicInstitution),
    Parish(ParishPopulation),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementBuildingDemand {
    pub buildings: Vec<BuildingDemand>,
    pub parishes: Vec<ParishProgramme>,
    /// Explicit unmet demand when the bounded plan is exhausted.
    pub shortfalls: Vec<DemandShortfall>,
}

impl SettlementBuildingDemand {
    pub fn new(seed: u64, population: u32, economy: &SettlementEconomyProfile) -> Self {
        Self::with_parish_policy(
            seed,
            population,
            economy,
            AuthoredParishPolicy::CENTRAL_GERMAN_MARKET_TOWN,
        )
    }

    pub fn with_parish_policy(
        seed: u64,
        population: u32,
        economy: &SettlementEconomyProfile,
        policy: AuthoredParishPolicy,
    ) -> Self {
        let (parishes, buildings, shortfalls) = parish::plan(population, economy, policy);
        let mut plan = Self {
            buildings,
            parishes,
            shortfalls,
        };
        if population == 0 {
            return plan;
        }
        for usage in BuildingUse::ALL {
            let definition = usage.definition();
            if !definition.eligible(population, economy) {
                continue;
            }
            let range = match definition.demand {
                BuildingDemandPolicy::ServiceCatchment(range) => range,
                BuildingDemandPolicy::SettlementInstitution(institution) => {
                    if plan.buildings.len() < MAX_SERVICE_BUILDINGS {
                        plan.buildings.push(BuildingDemand::Civic(institution));
                    } else {
                        plan.shortfalls.push(DemandShortfall::Civic(institution));
                    }
                    continue;
                }
                _ => continue,
            };
            let mut covered = 0_u32;
            let mut ordinal = 0_u32;
            while covered < population {
                if plan.buildings.len() == MAX_SERVICE_BUILDINGS {
                    plan.shortfalls.push(DemandShortfall::Service {
                        usage,
                        capacity: ServiceCapacity(population - covered),
                    });
                    break;
                }
                let capacity =
                    range.sample(CAPACITY_DOMAIN.seed(seed, &[usage as u64, u64::from(ordinal)]));
                plan.buildings.push(BuildingDemand::Service {
                    usage,
                    ordinal,
                    capacity,
                });
                covered = covered.saturating_add(capacity.0);
                ordinal += 1;
            }
        }
        plan
    }
}

impl BuildingDefinition {
    pub fn eligible(self, population: u32, economy: &SettlementEconomyProfile) -> bool {
        if population < self.minimum_population {
            return false;
        }
        match self.eligibility {
            BuildingEligibility::EverySettlement => true,
            BuildingEligibility::Service(service) => economy.has_service(service),
            BuildingEligibility::Specialization(stock) => economy.specializations.contains(&stock),
            // Surviving architecture alone cannot establish an institution in 1544.
            BuildingEligibility::HistoricalEvidence => false,
        }
    }
}

#[cfg(test)]
mod tests;
