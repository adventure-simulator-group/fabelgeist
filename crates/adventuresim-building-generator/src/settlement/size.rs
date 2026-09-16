//! Capacity bands selected before reserving a settlement building's physical plot.
use adventuresim_world_schema::settlement_buildings::{
    BuildingDemand, BuildingDemandPolicy, BuildingUse, ChurchBuildingScale, ParishBuildingRole,
    ServiceCapacity,
};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{BuildingArchetype, BuildingProgram, WorkplaceKind};

/// A physical programme band. Parish bands are authored independently of catchments.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum ServiceBuildingSize {
    Small,
    Medium,
    Large,
}

impl ServiceBuildingSize {
    pub fn for_demand(demand: BuildingDemand) -> Option<Self> {
        match demand {
            BuildingDemand::Service {
                usage, capacity, ..
            } => Self::for_capacity(usage, capacity),
            BuildingDemand::Parish {
                role: ParishBuildingRole::Church(scale),
                ..
            } => Some(match scale {
                ChurchBuildingScale::Village => Self::Small,
                ChurchBuildingScale::Neighbourhood => Self::Medium,
                ChurchBuildingScale::PrincipalTown => Self::Large,
            }),
            _ => None,
        }
    }

    pub fn for_capacity(usage: BuildingUse, capacity: ServiceCapacity) -> Option<Self> {
        WorkplaceKind::from_use(usage)?;
        let BuildingDemandPolicy::ServiceCatchment(range) = usage.definition().demand else {
            return None;
        };
        let span = range.maximum.0 - range.minimum.0 + 1;
        let band = capacity.0.saturating_sub(range.minimum.0).saturating_mul(3) / span;
        Some(match band {
            0 => Self::Small,
            1 => Self::Medium,
            _ => Self::Large,
        })
    }

    pub(crate) const fn extra_bays(self) -> u16 {
        match self {
            Self::Small => 0,
            Self::Medium => 1,
            Self::Large => 2,
        }
    }
}

impl BuildingProgram {
    /// Select a physical programme before plot reservation or representation compilation.
    pub fn with_service_size(mut self, size: ServiceBuildingSize) -> Self {
        if self.workplace_kind().is_some() {
            self.configure_workplace_size(size);
        } else if self.archetype == BuildingArchetype::ParishChurch {
            self.configure_small_church_size(size);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_ranges_select_monotonic_bands_for_workshops() {
        for usage in [BuildingUse::HorseMill] {
            let adventuresim_world_schema::settlement_buildings::BuildingDemandPolicy::ServiceCatchment(range) = usage.definition().demand else { panic!("expected service catchment"); };
            assert_eq!(
                ServiceBuildingSize::for_capacity(usage, range.minimum),
                Some(ServiceBuildingSize::Small)
            );
            assert_eq!(
                ServiceBuildingSize::for_capacity(usage, range.maximum),
                Some(ServiceBuildingSize::Large)
            );
            let mut previous = ServiceBuildingSize::Small;
            for capacity in range.minimum.0..=range.maximum.0 {
                let size =
                    ServiceBuildingSize::for_capacity(usage, ServiceCapacity(capacity)).unwrap();
                assert!(size >= previous);
                previous = size;
            }
        }
        assert_eq!(
            ServiceBuildingSize::for_capacity(BuildingUse::Dwelling, ServiceCapacity(100)),
            None
        );
        assert_eq!(
            ServiceBuildingSize::for_capacity(BuildingUse::Cathedral, ServiceCapacity(500)),
            None
        );
    }
}
