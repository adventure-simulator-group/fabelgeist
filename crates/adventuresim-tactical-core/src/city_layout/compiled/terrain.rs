//! Grade the city's complete properties and supporting vista sample cells.
use super::*;
const CITY_EDGE_MARGIN_METRES: f32 = 50.0;
const CITY_LEVEL_BLEND_METRES: f32 = 100.0;
impl CitySceneLayout {
    pub fn level_vista(&self, vista: &mut crate::scene_input::VistaSample, elevation_metres: f32) {
        if self.distant.is_empty() && self.gardens.is_empty() {
            return;
        }
        let Some(extent) = self
            .distant
            .iter()
            .map(|b| b.centre_metres)
            .chain(self.playable.iter().map(|b| b.centre_metres))
            .chain(self.compounds.iter().flat_map(|c| c.plot.corners()))
            .chain(self.gardens.iter().flat_map(|g| g.plot.corners()))
            .map(Vec2::abs)
            .reduce(Vec2::max)
        else {
            return;
        };
        for lod in &mut vista.lods {
            // Include every interpolation vertex supporting an outer property.
            // Each LOD grades its own sample collar before the terrain blend.
            let level_extent = extent + Vec2::splat(CITY_EDGE_MARGIN_METRES + lod.spacing_metres);
            let centre = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1)) * 0.5;
            for (index, height) in lod.heights_metres.iter_mut().enumerate() {
                let grid = Vec2::new(
                    (index % usize::from(lod.width)) as f32,
                    (index / usize::from(lod.width)) as f32,
                );
                let point = (grid - centre) * lod.spacing_metres;
                let outside = (point.abs() - level_extent).max(Vec2::ZERO).length();
                let weight = (1.0 - outside / CITY_LEVEL_BLEND_METRES).clamp(0.0, 1.0);
                let smooth = weight * weight * (3.0 - 2.0 * weight);
                *height += (elevation_metres - *height) * smooth;
            }
        }
    }
}
