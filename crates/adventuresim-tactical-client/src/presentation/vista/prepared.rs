//! Complete visible ring meshes before replacing the retained presentation.
use super::*;

pub(super) struct PreparedVistaLod<'a> {
    pub scenery: VistaSceneryRange,
    pub lod: &'a VistaLod,
    pub coarser: Option<&'a VistaLod>,
    /// Native grid clipping bounds retained with their upload-ready mesh batch.
    pub inner_half_extent: Vec2,
    pub meshes: Vec<Mesh>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum VistaSceneryRange {
    Near,
    Distant,
}

impl<'a> PreparedVistaLod<'a> {
    pub(super) fn prepare_all(
        lods: &[&'a VistaLod],
        playable_half_extent: Vec2,
        terrain: &SceneTerrain,
        environment: &SceneEnvironment,
        landform: Option<&TerrainLandformRecipe>,
    ) -> VistaMeshResult<Vec<Self>> {
        let mut inner_half_extent = playable_half_extent;
        let mut prepared = Vec::new();
        for (source_index, lod) in lods.iter().copied().enumerate() {
            let coarser = lods.get(source_index + 1).copied();
            let meshes = vista_lod_meshes_with_morph(
                lod,
                inner_half_extent,
                coarser,
                Some(terrain),
                (source_index == 0).then_some(environment),
                environment.weather,
                landform.map(|recipe| recipe.transition_collar()),
            )?;
            if meshes.is_empty() {
                continue;
            }
            let scenery = match source_index {
                0 | 1 => VistaSceneryRange::Near,
                _ => VistaSceneryRange::Distant,
            };
            prepared.push(Self { scenery, lod, coarser, inner_half_extent, meshes });
            let half_extent = f32::from(lod.width.saturating_sub(1)) * lod.spacing_metres * 0.5;
            inner_half_extent = Vec2::new(
                half_extent,
                f32::from(lod.depth.saturating_sub(1)) * lod.spacing_metres * 0.5,
            );
        }
        Ok(prepared)
    }
}
