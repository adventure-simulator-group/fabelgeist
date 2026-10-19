//! Full production geometry for a source-bound native content comparison.

use adventuresim_tactical_core::prelude::SceneTerrain;
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use fabelgeist_determinism::Seed;
use fabelgeist_math::units::Metres;

use super::super::source::source_normalization;
use super::super::{
    TreePresentationSpecies,
    materials::{
        beech_bark_material, beech_leaf_material, oak_bark_material, oak_leaf_material,
        tree_bud_material,
    },
    procedural_oak_bud_mesh, procedural_woody_branch_mesh, procedural_woody_cambered_leaf_mesh,
};
use super::evidence::SourceEnvelope;
use crate::presentation::{
    ProceduralTextureAssets, SceneEnvironment, TacticalTreeBarkMaterial,
    TacticalTreeLeafCardMaterial, terrain_heightmap_image,
};

mod presentation;
pub(crate) use presentation::{TreeSourceMeshBinding, TreeSourcePresentation};

pub(crate) struct TreeSourceGeometry {
    pub id: PlantId,
    pub species: TreePresentationSpecies,
    pub species_record: fabelgeist_botany::viewer::sim::SpeciesInfo,
    pub seed: Seed,
    pub branches: usize,
    pub cards: usize,
    pub minimum: [Metres; 3],
    pub maximum: [Metres; 3],
    wood: Mesh,
    foliage: Mesh,
    buds: Option<Mesh>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum TreeSourcePart {
    Wood,
    Foliage,
    Buds,
}

impl TreeSourceGeometry {
    pub(crate) fn find(id: PlantId, environment: &SceneEnvironment) -> Result<Self> {
        let specimen = super::specimens::SourceSpecimen::find(id)?;
        let species = specimen.species;
        let seed = specimen.seed;
        let (branches, leaves) = specimen.records(environment);
        let wood = procedural_woody_branch_mesh(&branches, u8::MAX);
        let foliage = procedural_woody_cambered_leaf_mesh(&leaves);
        let buds = procedural_oak_bud_mesh(&branches);
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for mesh in [&wood, &foliage, &buds]
            .into_iter()
            .filter(|mesh| mesh.count_vertices() > 0)
        {
            let envelope = SourceEnvelope::from_mesh(mesh)?;
            minimum = minimum.min(Vec3::from_array(
                envelope.minimum_metres.map(Metres::metres),
            ));
            maximum = maximum.max(Vec3::from_array(
                envelope.maximum_metres.map(Metres::metres),
            ));
        }
        ensure!(
            minimum.is_finite() && maximum.is_finite(),
            "source bounds unavailable"
        );
        Ok(Self {
            id,
            species,
            species_record: super::specimens::species_info(species),
            seed,
            branches: branches.len(),
            cards: leaves.len(),
            minimum: (minimum + source_normalization())
                .to_array()
                .map(Metres::from_metres),
            maximum: (maximum + source_normalization())
                .to_array()
                .map(Metres::from_metres),
            wood,
            foliage,
            buds: (buds.count_vertices() > 0).then_some(buds),
        })
    }
}
