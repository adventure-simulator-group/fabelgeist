//! Reconstruct accepted support without recomputing outdoor scene products.
use super::*;

/// Complete geographic context after accepted property support is installed.
/// Consumers retain every placement while selecting their presentation products.
#[derive(Debug)]
pub struct SupportedSceneTerrain {
    pub terrain: SceneTerrain,
    pub ground: SceneGround,
    pub buildings: Vec<GeneratedBuilding>,
    pub obstacles: Vec<GeneratedObstacle>,
    pub repairs: SceneRepairReport,
}

impl TacticalSceneInput {
    /// Validate the complete physical input and reconstruct its bound support.
    /// Offline furniture consumers use this phase instead of deleting buildings
    /// to avoid regenerating furniture at runtime.
    pub fn prepare_supported_terrain(
        &self,
        building_recipes: &mut GeneratedBuildingRecipes,
    ) -> Result<SupportedSceneTerrain, SceneInputError> {
        self.validate()?;
        let UngradedSceneTerrain {
            mut terrain,
            ground,
            buildings,
            mut obstacles,
            mut repairs,
        } = self.prepare_geographic_terrain(building_recipes)?;
        buildings::validate_building_pads(&buildings)?;
        compounds::validate_generated(&self.compounds, &buildings, &self.streets)?;
        if let Some(projection) = &self.grounding {
            let source = crate::city_layout::grounding::GeographicSurface::from_presented_scene(
                &terrain,
                &self.vista,
            )
            .ok_or_else(|| {
                SceneInputError::Validation("geographic support source is invalid".into())
            })?;
            let support = projection.reconstruct(&source, &self.physical_placements())?;
            terrain = terrain.with_property_surface(support);
        }
        // Geographic preparation is already bound. Exclude physical obstacles
        // intersecting accepted construction/access, without resampling source
        // relief or choosing a different property floor.
        support_obstacles::remove(self, &terrain, &mut obstacles, &mut repairs);
        Ok(SupportedSceneTerrain {
            terrain,
            ground,
            buildings,
            obstacles,
            repairs,
        })
    }
}
