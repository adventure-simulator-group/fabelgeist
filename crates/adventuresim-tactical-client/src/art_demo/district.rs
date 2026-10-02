//! Load the prepared production-generated settlement without validating every
//! building plan again on the browser's main thread.
use adventuresim_tactical_core::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct CityLayout {
    resident_population: u32,
    buildings: Vec<DistantBuildingPlacement>,
    streets: Vec<CityStreetPatch>,
    yards: Vec<CityYardPatch>,
    parishes: Vec<adventuresim_tactical_core::city_layout::CityParish>,
    compounds: Vec<CityCompound>,
    gardens: Vec<CityGarden>,
}

#[derive(Deserialize)]
pub(super) struct PreparedOutdoorFurniture {
    pub instances: Vec<GeneratedFurniture>,
    pub groups: Vec<FurnitureGroup>,
}

pub(super) fn curate(input: &mut TacticalSceneInput) -> Result<PreparedOutdoorFurniture, String> {
    let layout: CityLayout =
        serde_json::from_str(include_str!("../../../../assets/art-demo/city-layout.json"))
            .map_err(|error| error.to_string())?;
    bevy::log::info!(
        population = layout.resident_population,
        buildings = layout.buildings.len(),
        "Loaded art demo city layout"
    );
    input.buildings.clear();
    input.distant_buildings = layout.buildings;
    input.streets = layout.streets;
    input.yards = layout.yards;
    input.parishes = layout.parishes;
    input.compounds = layout.compounds;
    input.gardens = layout.gardens;
    input.validate().map_err(|error| error.to_string())?;
    serde_json::from_str(include_str!(
        "../../../../assets/art-demo/city-furniture.json"
    ))
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_city_keeps_thousands_of_inspectable_buildings_and_connected_surfaces() {
        let layout: CityLayout =
            serde_json::from_str(include_str!("../../../../assets/art-demo/city-layout.json"))
                .unwrap();
        assert_eq!(layout.resident_population, 30_000);
        let mut input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../assets/tactical-scenes/massive-city.json"
        ))
        .unwrap();
        let furniture = curate(&mut input).unwrap();
        assert!(!furniture.instances.is_empty());
        assert!(
            furniture
                .groups
                .iter()
                .any(|group| matches!(group.anchor, FurnitureAnchor::Market { .. }))
        );
        assert!(
            furniture
                .groups
                .iter()
                .any(|group| matches!(group.anchor, FurnitureAnchor::Building { .. }))
        );
        for instance in &furniture.instances {
            let FurnitureLocation::Outdoor { group_id } = instance.scene.location else {
                panic!("prepared city scenery must contain only outdoor furniture");
            };
            assert!(furniture.groups.iter().any(|group| group.id == group_id));
        }
        assert!(input.buildings.is_empty());
        assert!(input.distant_buildings.len() > 3_000);
        assert!(
            input
                .distant_buildings
                .iter()
                .any(|b| b.centre_metres.length() > 500.0)
        );
        assert!(input.streets.len() > 500);
        assert!(
            input
                .streets
                .iter()
                .any(|s| matches!(s, CityStreetPatch::Market { .. }))
        );
        input.validate().unwrap();
    }
}
