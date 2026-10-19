//! Full production geometry for a source-bound native content comparison.

use adventuresim_tactical_core::prelude::SceneTerrain;
use anyhow::{Context, Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::identity::PlantId;
use fabelgeist_determinism::Seed;
use fabelgeist_math::units::Metres;

use super::super::{
    OAK_GNARLING_SHOWCASE, TreePresentationSpecies,
    materials::{
        beech_bark_material, beech_leaf_material, oak_bark_material, oak_leaf_material,
        tree_bud_material,
    },
    procedural_oak_bud_mesh, procedural_woody_branch_mesh, procedural_woody_cambered_leaf_mesh,
    source::{canopy_competition, playable_tree_source},
    specimen::oak_variant_seed,
};
use super::{
    evidence::SourceEnvelope,
    specimens::{model_id, source_normalization},
};
use crate::presentation::{
    ProceduralTextureAssets, SceneEnvironment, TacticalTreeBarkMaterial,
    TacticalTreeLeafCardMaterial, terrain_heightmap_image,
};

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
        for species in [
            TreePresentationSpecies::EnglishOak,
            TreePresentationSpecies::CommonBeech,
        ] {
            for slot in 0..OAK_GNARLING_SHOWCASE.len() {
                let seed = oak_variant_seed(slot);
                if model_id(species, seed) != id {
                    continue;
                }
                let (branches, leaves) = playable_tree_source(
                    species,
                    seed,
                    slot,
                    canopy_competition(environment.canopy_bps),
                    environment,
                );
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
                return Ok(Self {
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
                });
            }
        }
        anyhow::bail!("source model has no production species/seed binding")
    }

    /// Native Bevy geometry boundary; root is area-world metres at ground level.
    pub(crate) fn spawn(self, world: &mut World, root_metres: Vec3) -> Result<()> {
        let terrain = world
            .query::<&SceneTerrain>()
            .single(world)
            .context("one actual production terrain for source contact")?
            .clone();
        let textures = world.resource::<ProceduralTextureAssets>().clone();
        let heightmap = world
            .resource_mut::<Assets<Image>>()
            .add(terrain_heightmap_image(&terrain));
        let range = Vec2::new(terrain.minimum_height(), terrain.maximum_height());
        let bark = match self.species {
            TreePresentationSpecies::EnglishOak => {
                oak_bark_material(&textures, heightmap, range, &terrain)
            }
            TreePresentationSpecies::CommonBeech => {
                beech_bark_material(&textures, heightmap, range, &terrain)
            }
        };
        let leaf = match self.species {
            TreePresentationSpecies::EnglishOak => oak_leaf_material(&textures),
            TreePresentationSpecies::CommonBeech => beech_leaf_material(&textures),
        };
        let bark = world
            .resource_mut::<Assets<TacticalTreeBarkMaterial>>()
            .add(bark);
        let leaf = world
            .resource_mut::<Assets<TacticalTreeLeafCardMaterial>>()
            .add(leaf);
        let wood = world.resource_mut::<Assets<Mesh>>().add(self.wood);
        let foliage = world.resource_mut::<Assets<Mesh>>().add(self.foliage);
        let transform = Transform::from_translation(root_metres + source_normalization());
        world.spawn((
            TreeSourcePart::Wood,
            Mesh3d(wood),
            MeshMaterial3d(bark),
            transform,
            Visibility::Visible,
        ));
        world.spawn((
            TreeSourcePart::Foliage,
            Mesh3d(foliage),
            MeshMaterial3d(leaf),
            transform,
            Visibility::Visible,
        ));
        if let Some(mesh) = self.buds {
            let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
            let material = world
                .resource_mut::<Assets<StandardMaterial>>()
                .add(tree_bud_material(self.species));
            world.spawn((
                TreeSourcePart::Buds,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                transform,
                Visibility::Visible,
            ));
        }
        Ok(())
    }
}
