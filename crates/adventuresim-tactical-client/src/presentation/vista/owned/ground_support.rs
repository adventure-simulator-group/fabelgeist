//! Prepare paving support from the same owned surface and displayed extent.
use super::*;
use crate::presentation::vista::streets::GroundSupport;
use adventuresim_tactical_core::city_layout::grounding::BoundedSettlementTerrain;

impl GroundSupport {
    pub(in crate::presentation::vista) fn add_owned_region(
        &mut self,
        surface: &BoundedSettlementTerrain,
        half_extent: Vec2,
        transition_collar: Option<TerrainTransitionCollar>,
    ) {
        let regions = [[-half_extent, half_extent]];
        let presentation = GroundPresentation::in_rectangles(surface, &regions);
        self.add_triangles(
            presentation
                .triangles(transition_collar)
                .flat_map(|triangle| {
                    clip::PreparedTriangle::new(triangle).in_rectangle(-half_extent, half_extent)
                }),
            Vec3::ZERO,
        );
    }
}
