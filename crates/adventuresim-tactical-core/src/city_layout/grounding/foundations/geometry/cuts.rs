//! Immutable grading outlines retain exact polygon coordinates and order.
use super::*;

pub(in crate::city_layout::grounding::foundations) struct SourceCutRegions {
    outlines: Vec<Vec<bevy::math::DVec2>>,
    query: PlanarQueryIndex,
}

impl SourceCutRegions {
    pub fn from_outlines(outlines: Vec<Vec<bevy::math::DVec2>>) -> Self {
        let query = PlanarQueryIndex::from_bounds(
            outlines
                .iter()
                .map(|outline| {
                    PlanarBounds::from_points(outline.iter().copied())
                        .expect("declared grading regions contain finite outline vertices")
                })
                .collect(),
        );
        Self { outlines, query }
    }

    pub fn intersecting(
        &self,
        triangle: &GroundTriangle,
    ) -> impl Iterator<Item = &[bevy::math::DVec2]> {
        self.query
            .intersections(triangle.bounds())
            .into_iter()
            .map(|i| self.outlines[i].as_slice())
    }
}
