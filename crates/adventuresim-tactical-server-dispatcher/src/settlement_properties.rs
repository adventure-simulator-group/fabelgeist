//! UI-independent property catalogue from the production frontage planner.
use adventuresim_core::{
    reputation::effective_population, settlement_property::GeneratedHomeCatalog,
};
use adventuresim_tactical_core::city_layout::CitySite;
use adventuresim_world_schema::SettlementEconomyProfile;

pub fn generated_homes(
    settlement_id: &str,
    population_level: i32,
    population_estimate: u32,
    economy: &SettlementEconomyProfile,
) -> Result<GeneratedHomeCatalog, String> {
    let seed = adventuresim_core::settlement_population::settlement_building_seed(settlement_id);
    let population = effective_population(population_level, population_estimate);
    let layout = CitySite::central_german_market_town().generate(seed, population, economy);
    if layout.unhoused_population > 0
        || !layout.unplaced_services.is_empty()
        || !layout.demand_shortfalls.is_empty()
    {
        return Err("Generated settlement property capacity is incomplete".into());
    }
    layout
        .compile(seed)
        .map_err(|error| error.to_string())?
        .partition(None)
        .map_err(|error| error.to_string())?
        .generated_homes(settlement_id, population)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement_buildings::{SettlementSceneProfile, place_settlement_buildings};
    #[test]
    fn homes_resolve_to_unscaled_production_buildings_at_every_partition() {
        let economy = SettlementEconomyProfile::stage_placeholder();
        for (id, population) in [("village", 900), ("town", 6500), ("viabundus-2337", 12000)] {
            let homes = generated_homes(id, 1, population, &economy).unwrap();
            let profile = SettlementSceneProfile {
                id: id.into(),
                population_level: 1,
                population_estimate: population,
                economy: economy.clone(),
                operators: Vec::new(),
            };
            for radius in [35.0, 90.0] {
                let scene = place_settlement_buildings(&profile, radius).unwrap();
                for home in &homes.homes {
                    let (centre, orientation, program) = scene
                        .playable
                        .iter()
                        .find(|building| building.id == home.building_id)
                        .map(|building| {
                            (
                                building.centre_metres,
                                building.orientation,
                                building.program.clone(),
                            )
                        })
                        .or_else(|| {
                            scene
                                .distant
                                .iter()
                                .find(|building| building.id == home.building_id)
                                .map(|building| {
                                    (
                                        building.centre_metres,
                                        building.orientation,
                                        building.occupied_program(),
                                    )
                                })
                        })
                        .unwrap();
                    assert_eq!(
                        centre,
                        bevy::math::Vec2::new(home.east_metres, home.north_metres)
                    );
                    assert_eq!(orientation.yaw_radians(), home.yaw_radians);
                    assert_eq!(
                        f32::from(program.footprint.dimensions().0)
                            * adventuresim_building_generator::CELL_SIZE_METRES,
                        home.width_metres
                    );
                    assert_eq!(
                        f32::from(program.footprint.dimensions().1)
                            * adventuresim_building_generator::CELL_SIZE_METRES,
                        home.depth_metres
                    );
                }
            }
            assert_eq!(
                homes
                    .allocate(&[])
                    .unwrap()
                    .iter()
                    .map(|home| home.residents)
                    .sum::<u32>(),
                population
            );
        }
    }
}
