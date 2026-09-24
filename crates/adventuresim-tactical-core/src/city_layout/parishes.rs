//! Persisted ownership of church precincts and their resident catchments.
use super::*;
use adventuresim_world_schema::settlement_buildings::{ParishBuildingRole, ParishPopulation};
use serde::{Deserialize, Serialize};
#[cfg(test)]
mod tests;

pub const CITY_PARISH_PRECINCT_RADIUS_METRES: f32 = 90.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParishResidenceAllocation {
    pub building_id: u64,
    pub residents: ParishPopulation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityParish {
    pub programme: ParishProgramme,
    pub church_building_id: u64,
    pub rectory_building_id: u64,
    pub school_building_id: Option<u64>,
    pub residences: Vec<ParishResidenceAllocation>,
}

impl GeneratedCityLayout {
    pub fn parish_layout(&self) -> Result<Vec<CityParish>, CityCompileError> {
        let lots = &self.lots;
        let programmes = &self.parishes;
        let mut parishes = Vec::new();
        let mut centres = Vec::new();
        for &programme in programmes {
            let member = |expected| {
                lots.iter().find(|lot| matches!(lot.service,
            Some(BuildingDemand::Parish { parish, role }) if parish == programme.id && role == expected))
            };
            let church = member(ParishBuildingRole::Church(programme.church_scale)).ok_or(
                CityCompileError::Parish {
                    parish: programme.id,
                },
            )?;
            let rectory =
                member(ParishBuildingRole::Residence).ok_or(CityCompileError::Parish {
                    parish: programme.id,
                })?;
            centres.push(church.centre_metres);
            parishes.push(CityParish {
                programme: ParishProgramme {
                    population: ParishPopulation(0),
                    ..programme
                },
                church_building_id: church.id,
                rectory_building_id: rectory.id,
                school_building_id: member(ParishBuildingRole::TownSchool).map(|lot| lot.id),
                residences: Vec::new(),
            });
        }
        let mut remaining: u32 = programmes.iter().map(|parish| parish.population.0).sum();
        let mut houses = lots
            .iter()
            .filter(|lot| lot.service.is_none())
            .collect::<Vec<_>>();
        let count = parishes.len();
        for (index, parish) in parishes.iter_mut().enumerate() {
            let successors = count - index - 1;
            let target = remaining.div_ceil((successors + 1) as u32);
            houses.sort_by(|a, b| {
                b.centre_metres
                    .distance_squared(centres[index])
                    .total_cmp(&a.centre_metres.distance_squared(centres[index]))
                    .then_with(|| b.id.cmp(&a.id))
            });
            while parish.programme.population.0 < target && houses.len() > successors {
                let lot = houses.pop().expect("available house count was checked");
                // Every selected dwelling represents occupied housing. Reserve
                // at least one resident for each house still awaiting assignment.
                let residents = remaining
                    .saturating_sub(houses.len() as u32)
                    .min(lot.house_class.resident_capacity());
                remaining -= residents;
                parish.programme.population.0 += residents;
                parish.residences.push(ParishResidenceAllocation {
                    building_id: lot.id,
                    residents: ParishPopulation(residents),
                });
            }
        }
        if remaining > 0 {
            return Err(CityCompileError::Capacity {
                residents: remaining,
                services: 0,
            });
        }
        if let Some(parish) = parishes
            .iter()
            .find(|parish| parish.programme.population.0 == 0)
        {
            return Err(CityCompileError::Parish {
                parish: parish.programme.id,
            });
        }
        Ok(parishes)
    }
}
