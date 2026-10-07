//! Load the prepared production-generated settlement without validating every
//! building plan again on the browser's main thread.
use adventuresim_tactical_core::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CityLayout {
    resident_population: u32,
    schema_version: u16,
    generation_version: u16,
    grounding: adventuresim_tactical_core::city_layout::CityGroundingProjection,
    buildings: Vec<DistantBuildingPlacement>,
    establishments: Vec<SceneEstablishment>,
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
    if layout.schema_version != input.schema_version
        || layout.generation_version != input.generation_version
    {
        return Err(
            "prepared city uses a different scene format; regenerate the city assets".into(),
        );
    }
    input.grounding = Some(layout.grounding);
    input.establishments = layout.establishments;
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
    use adventuresim_tactical_core::scene_input::GeneratedBuildingRecipes;
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
                .any(|b| b.centre_metres.metres().length() > 500.0)
        );
        assert!(input.streets.len() > 500);
        assert!(
            input
                .streets
                .iter()
                .any(|s| matches!(s, CityStreetPatch::Market { .. }))
        );
        input.validate().unwrap();
        let before = input.digest().unwrap();
        let terrain = input
            .prepare_supported_terrain(&mut GeneratedBuildingRecipes::default())
            .unwrap();
        let members: std::collections::BTreeSet<_> = terrain
            .terrain
            .property_surface()
            .unwrap()
            .foundations()
            .iter()
            .flat_map(|foundation| foundation.member_building_ids().iter().copied())
            .collect();
        assert_eq!(
            members,
            input
                .distant_buildings
                .iter()
                .map(|building| building.id)
                .collect()
        );
        assert_eq!(input.digest().unwrap(), before);
        for group in &furniture.groups {
            if let FurnitureAnchor::Building { id } = group.anchor {
                assert!(members.contains(&id));
            }
        }
    }
}
