//! Plant roots use the same immutable soil triangles as collision and rendering.
use super::*;

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum GardenSupportError {
    #[error("property {property:?}, building {building}, plant {plant:?}: invalid garden identity")]
    Identity {
        property: crate::city_layout::CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
        plant: Option<GardenPlantId>,
    },
    #[error(
        "property {property:?}, building {building}, root {index} expected {expected:?}, received {actual:?}"
    )]
    Membership {
        property: crate::city_layout::CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
        index: usize,
        expected: Option<GardenPlantId>,
        actual: Option<GardenPlantId>,
    },
    #[error(
        "property {property:?}, building {building}, plant {plant:?} lacks soil support at {location_metres:?}"
    )]
    MissingSoil {
        property: crate::city_layout::CityPropertyId,
        building: crate::scene_input::SceneBuildingId,
        plant: GardenPlantId,
        location_metres: crate::scene_coordinates::ScenePlanPoint,
    },
}

impl SceneGarden {
    /// Project every accepted plant without moving, omitting or reseeding it.
    /// This binds roots only; route grade and enclosure clearance have separate
    /// acceptance checks. The complete geographic/support terrain is required.
    pub fn project(garden: CityGarden, terrain: &SceneTerrain) -> Result<Self, GardenSupportError> {
        let plant_support = garden
            .plants
            .iter()
            .map(|plant| {
                let elevation = terrain
                    .height_at(plant.centre_metres.metres())
                    .and_then(SupportElevation::from_metres)
                    .ok_or(GardenSupportError::MissingSoil {
                        property: garden.owner,
                        building: garden.front_building_id,
                        plant: plant.id,
                        location_metres: plant.centre_metres,
                    })?;
                Ok(GardenPlantSupport {
                    plant_id: plant.id,
                    elevation,
                })
            })
            .collect::<Result<_, GardenSupportError>>()?;
        Self::from_support(garden, plant_support)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_missing_root_rejects_the_whole_projection_with_exact_identity() {
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../../assets/tactical-scenes/garden-review.json"
        ))
        .unwrap();
        let garden = input.gardens[1].clone();
        let terrain = SceneTerrain::from_heightmap(3, 3, 1.0, vec![0.0; 9]).unwrap();
        assert_eq!(
            SceneGarden::project(garden.clone(), &terrain).unwrap_err(),
            GardenSupportError::MissingSoil {
                property: garden.owner,
                building: garden.front_building_id,
                plant: garden.plants[0].id,
                location_metres: garden.plants[0].centre_metres,
            }
        );
    }
    #[test]
    fn replicated_soil_elevations_reject_nonfinite_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let bytes = postcard::to_allocvec(&value).unwrap();
            assert!(postcard::from_bytes::<SupportElevation>(&bytes).is_err());
        }
        let elevation = SupportElevation::from_metres(-17.25).unwrap();
        let bytes = postcard::to_allocvec(&elevation).unwrap();
        assert_eq!(
            postcard::from_bytes::<SupportElevation>(&bytes).unwrap(),
            elevation
        );
    }
}
