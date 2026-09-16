//! Authored parish jurisdictions, independent of church occupancy and shop demand.
use super::{BuildingDemand, BuildingUse, DemandShortfall, MAX_SERVICE_BUILDINGS};
use crate::SettlementEconomyProfile;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;

/// A local identity within one settlement, not a historical diocesan identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ParishId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParishPopulation(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParishProminence {
    PrincipalTown,
    Neighbourhood,
}

/// Physical programme selected independently of a simultaneous audience count.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum ChurchBuildingScale {
    Village,
    Neighbourhood,
    PrincipalTown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ParishBuildingRole {
    Church(ChurchBuildingScale),
    Residence,
    TownSchool,
}

impl ParishBuildingRole {
    pub const fn usage(self) -> BuildingUse {
        match self {
            Self::Church(_) => BuildingUse::ParishChurch,
            Self::Residence => BuildingUse::Rectory,
            Self::TownSchool => BuildingUse::School,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ParishProgramme {
    pub id: ParishId,
    pub population: ParishPopulation,
    pub prominence: ParishProminence,
    pub church_scale: ChurchBuildingScale,
}

/// Fictional town design input, not a measured historical population average.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthoredParishPolicy {
    pub target_population: NonZeroU32,
}

impl AuthoredParishPolicy {
    pub const CENTRAL_GERMAN_MARKET_TOWN: Self = Self {
        target_population: NonZeroU32::new(3_000).unwrap(),
    };
}

const MAX_PARISH_PROGRAMMES: u32 = (MAX_SERVICE_BUILDINGS / 3) as u32;
const VILLAGE_PARISH_POPULATION: u32 = 1_500;

pub(super) fn plan(
    population: u32,
    economy: &SettlementEconomyProfile,
    policy: AuthoredParishPolicy,
) -> (
    Vec<ParishProgramme>,
    Vec<BuildingDemand>,
    Vec<DemandShortfall>,
) {
    if population == 0
        || !BuildingUse::ParishChurch
            .definition()
            .eligible(population, economy)
    {
        return (Vec::new(), Vec::new(), Vec::new());
    }
    let desired = population.div_ceil(policy.target_population.get());
    let count = desired.min(MAX_PARISH_PROGRAMMES);
    let quotient = population / desired;
    let remainder = population % desired;
    let mut parishes = Vec::new();
    let mut buildings = Vec::new();
    let school = BuildingUse::School
        .definition()
        .eligible(population, economy);
    for ordinal in 0..count {
        let id = ParishId(ordinal);
        let prominence = if ordinal == 0 {
            ParishProminence::PrincipalTown
        } else {
            ParishProminence::Neighbourhood
        };
        let church_scale = if population < VILLAGE_PARISH_POPULATION {
            ChurchBuildingScale::Village
        } else if ordinal == 0 {
            ChurchBuildingScale::PrincipalTown
        } else {
            ChurchBuildingScale::Neighbourhood
        };
        parishes.push(ParishProgramme {
            id,
            population: ParishPopulation(quotient + u32::from(ordinal < remainder)),
            prominence,
            church_scale,
        });
        buildings.push(BuildingDemand::Parish {
            parish: id,
            role: ParishBuildingRole::Church(church_scale),
        });
        buildings.push(BuildingDemand::Parish {
            parish: id,
            role: ParishBuildingRole::Residence,
        });
        if ordinal == 0 && school {
            buildings.push(BuildingDemand::Parish {
                parish: id,
                role: ParishBuildingRole::TownSchool,
            });
        }
    }
    let assigned: u32 = parishes.iter().map(|p| p.population.0).sum();
    let shortfalls = (assigned < population)
        .then_some(DemandShortfall::Parish(ParishPopulation(
            population - assigned,
        )))
        .into_iter()
        .collect();
    (parishes, buildings, shortfalls)
}
