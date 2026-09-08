//! Building demand from the canonical economy, without a second service lottery.
//!
//! Capacities are gameplay catchments, not historical attendance or occupancy.
use crate::SettlementEconomyProfile;
use fabelgeist_determinism::mix64;

mod catalog;
pub use catalog::{BuildingDefinition, BuildingDistrict, BuildingEligibility, BuildingUse};

const CAPACITY_DOMAIN: u64 = 0x6361_7061_6369_7479;
pub const MAX_SERVICE_BUILDINGS: usize = 4_096;

/// Approximate people served, distinct from residents housed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    fn sample(self, seed: u64) -> ServiceCapacity {
        let width = u64::from(self.maximum.0) - u64::from(self.minimum.0) + 1;
        ServiceCapacity(self.minimum.0 + (mix64(seed) % width) as u32)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildingDemand {
    pub usage: BuildingUse,
    pub ordinal: u32,
    pub capacity: ServiceCapacity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettlementBuildingDemand {
    pub buildings: Vec<BuildingDemand>,
    /// Explicit unmet demand when the bounded plan is exhausted.
    pub shortfalls: Vec<(BuildingUse, ServiceCapacity)>,
}

impl SettlementBuildingDemand {
    pub fn new(seed: u64, population: u32, economy: &SettlementEconomyProfile) -> Self {
        let mut plan = Self {
            buildings: Vec::new(),
            shortfalls: Vec::new(),
        };
        if population == 0 {
            return plan;
        }
        for usage in BuildingUse::ALL {
            let definition = usage.definition();
            if !definition.eligible(population, economy) {
                continue;
            }
            let mut covered = 0_u32;
            let mut ordinal = 0_u32;
            while covered < population {
                if plan.buildings.len() == MAX_SERVICE_BUILDINGS {
                    plan.shortfalls
                        .push((usage, ServiceCapacity(population - covered)));
                    break;
                }
                let capacity = definition.capacity.sample(
                    seed ^ CAPACITY_DOMAIN ^ (usage as u64).rotate_left(23) ^ u64::from(ordinal),
                );
                plan.buildings.push(BuildingDemand {
                    usage,
                    ordinal,
                    capacity,
                });
                covered = covered.saturating_add(capacity.0);
                ordinal += 1;
                if definition.singleton {
                    break;
                }
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
