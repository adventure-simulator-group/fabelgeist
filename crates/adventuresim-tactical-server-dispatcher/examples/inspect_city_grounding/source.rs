//! Diagnostic source sampling; vista observations never become tactical support.
use adventuresim_tactical_core::{
    city_layout::grounding::SupportElevation,
    scene::SceneTerrain,
    scene_coordinates::ScenePlanPoint,
    scene_input::VistaSample,
    vista_surface::vista_triangle_height,
};

pub(super) struct InspectionSource<'a> {
    terrain: &'a SceneTerrain,
    vista: &'a VistaSample,
}

impl<'a> InspectionSource<'a> {
    pub(super) fn new(terrain: &'a SceneTerrain, vista: &'a VistaSample) -> Self {
        Self { terrain, vista }
    }

    pub(super) fn height_at(&self, point: ScenePlanPoint) -> Option<SupportElevation> {
        self.terrain.height_at(point).or_else(|| {
            self.vista.lods.iter().enumerate().find_map(|(index, lod)| {
                // The complete triangle/LOD stitch kernel consumes scene metres.
                SupportElevation::from_metres(vista_triangle_height(
                    lod,
                    self.vista.lods.get(index + 1),
                    self.terrain,
                    point.metres(),
                )?)
            })
        })
    }
}
