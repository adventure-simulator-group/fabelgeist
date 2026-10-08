//! Persisted ownership of church precincts and their resident catchments.
use super::*;
use adventuresim_world_schema::settlement_buildings::{ParishBuildingRole, ParishPopulation};
use serde::{Deserialize, Serialize};

pub const CITY_PARISH_PRECINCT_RADIUS_METRES: f32 = 90.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParishResidenceAllocation {
    pub building_id: crate::scene_input::SceneBuildingId,
    pub residents: ParishPopulation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityParish {
    pub programme: ParishProgramme,
    pub church_building_id: crate::scene_input::SceneBuildingId,
    pub rectory_building_id: crate::scene_input::SceneBuildingId,
    pub school_building_id: Option<crate::scene_input::SceneBuildingId>,
    pub residences: Vec<ParishResidenceAllocation>,
}

impl GeneratedCityLayout {
    pub fn parish_layout(&self) -> CityCompileResult<Vec<CityParish>> {
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
                church_building_id: church.front_building_id(),
                rectory_building_id: rectory.front_building_id(),
                school_building_id: member(ParishBuildingRole::TownSchool)
                    .map(|lot| lot.front_building_id()),
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
                    .metres()
                    .distance_squared(centres[index].metres())
                    .total_cmp(
                        &a.centre_metres
                            .metres()
                            .distance_squared(centres[index].metres()),
                    )
                    .then_with(|| b.id.cmp(&a.id))
            });
            while parish.programme.population.0 < target && houses.len() > successors {
                let lot = houses.pop().ok_or(CityCompileError::Parish {
                    parish: parish.programme.id,
                })?;
                // Every selected dwelling represents occupied housing. Reserve
                // at least one resident for each house still awaiting assignment.
                let residents = remaining
                    .saturating_sub(houses.len() as u32)
                    .min(lot.house_class.resident_capacity().get());
                remaining -= residents;
                parish.programme.population.0 += residents;
                parish.residences.push(ParishResidenceAllocation {
                    building_id: lot.front_building_id(),
                    residents: ParishPopulation(residents),
                });
            }
        }
        if remaining > 0 {
            return Err(CityCompileError::Capacity {
                residents: ResidentCount::new(remaining),
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
#[cfg(test)]
mod tests;
