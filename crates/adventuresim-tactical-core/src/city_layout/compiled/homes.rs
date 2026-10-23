//! Residence authority uses accepted physical positions at every scene partition.
use super::*;
use adventuresim_building_generator::CELL_SIZE_METRES;
use adventuresim_core::settlement_property::{
    GeneratedHome, GeneratedHomeCatalog, HousingMarketReserve, PropertyError, PropertyId,
};

impl CitySceneLayout {
    pub fn generated_homes(
        &self,
        settlement_id: &str,
        population: u32,
    ) -> Result<GeneratedHomeCatalog, PropertyError> {
        let mut homes = self
            .playable
            .iter()
            .map(|b| (b.id, b.centre_metres, b.orientation, b.program.clone()))
            .chain(
                self.distant
                    .iter()
                    .map(|b| (b.id, b.centre_metres, b.orientation, b.occupied_program())),
            )
            .filter(|(_, _, _, program)| program.usage == Some(BuildingUse::Dwelling))
            .collect::<Vec<_>>();
        homes.sort_by_key(|(id, ..)| *id);
        let mut market = HousingMarketReserve::default();
        let homes = homes
            .into_iter()
            .map(|(id, centre, orientation, program)| {
                let class = CityHouseClass::from_archetype(program.archetype)
                    .ok_or(PropertyError::InvalidCatalog)?;
                let tier = class.housing_tier();
                let (width, depth) = program.footprint.dimensions();
                Ok(GeneratedHome {
                    id: PropertyId::new(settlement_id, id.0)?,
                    building_id: id.0,
                    tier,
                    resident_capacity: class.resident_capacity(),
                    market_reserve: market.reserve(tier),
                    east_metres: centre.metres().x,
                    north_metres: centre.metres().y,
                    yaw_radians: orientation.yaw_radians(),
                    width_metres: f32::from(width) * CELL_SIZE_METRES,
                    depth_metres: f32::from(depth) * CELL_SIZE_METRES,
                })
            })
            .collect::<Result<Vec<_>, PropertyError>>()?;
        let catalog = GeneratedHomeCatalog {
            settlement_id: settlement_id.to_owned(),
            population,
            homes,
            seed: adventuresim_core::settlement_population::settlement_building_seed(settlement_id),
        };
        catalog.validate(settlement_id, population)?;
        Ok(catalog)
    }
}
