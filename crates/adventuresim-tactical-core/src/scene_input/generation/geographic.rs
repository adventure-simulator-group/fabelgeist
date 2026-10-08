//! Complete scene context prepares geographic relief before property grading.
use super::*;

/// Geographic preparation retains the scene's complete asset reservations.
/// Floors and owned support are installed only after this surface is prepared.
#[derive(Debug)]
pub struct UngradedSceneTerrain {
    pub terrain: SceneTerrain,
    pub ground: SceneGround,
    pub buildings: Vec<GeneratedBuilding>,
    pub obstacles: Vec<GeneratedObstacle>,
    pub repairs: SceneRepairReport,
}

impl TacticalSceneInput {
    /// Prepare the exact fine terrain used as the source for bounded grading.
    /// No scene clone, synthetic source or omitted property changes the ground
    /// cover or obstacle context. This operation does not seat buildings.
    pub fn prepare_geographic_terrain(
        &self,
        recipes: &mut GeneratedBuildingRecipes,
    ) -> Result<UngradedSceneTerrain, SceneInputError> {
        self.validate_source()?;
        let buildings = buildings::prepare_buildings(&self.buildings, recipes)?;
        let (width, depth, spacing, mut heights, mut environment) =
            upsample_playable_grid(&self.playable);
        let mut repairs =
            prepare_terrain(self, width, depth, spacing, &mut heights, &mut environment)?;
        let coarse = SceneTerrain::from_heightmap(width, depth, spacing, heights).ok_or(
            SceneInputError::Validation(SceneValidationError::GeographicHeightmap),
        )?;
        let mut obstacles = generated_obstacles(self);
        remove_reserved_obstacles(self, &mut obstacles, &mut repairs);
        let reservations = reservation_pads(self, &buildings)?;
        remove_building_obstacles(self, &coarse, &reservations, &mut obstacles, &mut repairs);
        let ground = build_scene_ground(
            width,
            depth,
            spacing,
            &environment,
            &coarse,
            &obstacles,
            self.playable.spacing_metres,
            &buildings,
            &self.streets,
            &self.yards,
        )?;
        // Geographic repair and microrelief precede property grading. No pad
        // selects a floor, suppresses source relief or levels adjacent land.
        let terrain = refine_authoritative_terrain(
            self.seed,
            &coarse,
            &ground,
            &obstacles,
            self.playable.spacing_metres,
            self.weather.ground_moisture_bps,
            &[],
        )?;
        Ok(UngradedSceneTerrain {
            terrain,
            ground,
            buildings,
            obstacles,
            repairs,
        })
    }
}

fn reservation_pads(
    input: &TacticalSceneInput,
    buildings: &[GeneratedBuilding],
) -> Result<Vec<buildings::BuildingPad>, SceneInputError> {
    let mut pads: Vec<_> = buildings
        .iter()
        .map(|building| Ok(buildings::BuildingPad {
            centre: building.placement.centre_metres.metres(),
            half_extents: building.collision.bounds.plan_half_extents()?.metres(),
            orientation: building.placement.orientation,
            elevation_metres: building.placement.base_elevation_metres,
        }))
        .collect::<Result<Vec<_>, adventuresim_building_generator::spatial_geometry::GeometryError>>()?;
    // Only horizontal occupancy removes obstacles. Compound members may have
    // different floors, and garden soil need not share its house's elevation.
    pads.extend(
        input
            .compounds
            .iter()
            .map(|property| property.plot)
            .chain(input.gardens.iter().map(|garden| garden.plot))
            .map(|plot| buildings::BuildingPad {
                centre: plot.centre_metres(),
                half_extents: plot.dimensions_metres() * 0.5,
                orientation: plot.orientation(),
                elevation_metres: crate::city_layout::grounding::SupportElevation::ZERO,
            }),
    );
    Ok(pads)
}

#[cfg(test)]
mod tests;
