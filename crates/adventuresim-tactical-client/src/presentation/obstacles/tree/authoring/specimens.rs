//! Eight deterministic mature source specimens; no ecosystem or GPU context.

use std::sync::Arc;

use adventuresim_environment::{
    prepared::{PreparedUnique, TreeFocus, TreeLibrary, TreeRoot},
    render::ForestDetail,
};
use adventuresim_tactical_core::scene_input::TREE_TRUNK_HEIGHT_METRES;
use anyhow::{Context, Result};
use bevy::prelude::*;
use fabelgeist_botany::{
    Biome,
    identity::{PlantId, PlantIndex, SpeciesId},
    species::Foliage,
    viewer::sim::{
        BlendShapes, Litter, PlantInstance, Snapshot, SpeciesInfo, TerrainGeometry, TerrainVertex,
    },
};
use fabelgeist_determinism::StreamId;
use fabelgeist_math::units::Metres;

use super::super::{
    OAK_GNARLING_SHOWCASE, TreePresentationSpecies,
    source::{canopy_competition, playable_tree_source},
    specimen::oak_variant_seed,
};
use super::{
    evidence::{BranchEvidence, SpecimenEvidence},
    records,
};
use crate::presentation::SceneEnvironment;

const SPACING: Metres = Metres::from_metres(32.0);
const SOURCE_ID: StreamId = StreamId::new("visual.tree.bank.source");

pub(super) struct PreparedSpecimens {
    pub unique: PreparedUnique,
    pub evidence: Vec<SpecimenEvidence>,
}

pub(in crate::presentation) struct SourceSpecimen {
    pub id: PlantId,
    pub species: TreePresentationSpecies,
    pub seed: fabelgeist_determinism::Seed,
    slot: usize,
}

impl SourceSpecimen {
    pub(in crate::presentation) fn find(id: PlantId) -> Result<Self> {
        [
            TreePresentationSpecies::EnglishOak,
            TreePresentationSpecies::CommonBeech,
        ]
        .into_iter()
        .flat_map(|species| {
            (0..OAK_GNARLING_SHOWCASE.len()).map(move |slot| {
                let seed = oak_variant_seed(slot);
                Self {
                    id: model_id(species, seed),
                    species,
                    seed,
                    slot,
                }
            })
        })
        .find(|specimen| specimen.id == id)
        .context("source model has no production species/seed binding")
    }

    pub(in crate::presentation) fn records(
        &self,
        environment: &SceneEnvironment,
    ) -> (
        Vec<super::super::TreeBranchSegment>,
        Vec<super::super::TreeLeaf>,
    ) {
        playable_tree_source(
            self.species,
            self.seed,
            self.slot,
            canopy_competition(environment.canopy_bps),
            environment,
        )
    }
}

impl PreparedSpecimens {
    pub(super) fn bind_model_bounds(&mut self, library: &TreeLibrary) -> Result<()> {
        for (root, plant) in self
            .unique
            .roots
            .iter()
            .zip(&mut self.unique.snapshot.plant_bounds)
        {
            let model = library
                .models
                .iter()
                .find(|model| model.source == root.id)
                .ok_or_else(|| anyhow::anyhow!("source specimen missing from all-plants bank"))?;
            let origin = Vec3::from_array(root.position.map(Metres::metres));
            let low = Vec3::from_array(model.bounds.low.map(Metres::metres));
            let high = Vec3::from_array(model.bounds.high.map(Metres::metres));
            let center = (low + high) * 0.5;
            let world = center + origin;
            plant.sphere = [world.x, world.y, world.z, center.distance(high)];
            if self.unique.focus.is_none() {
                self.unique.focus = Some(TreeFocus {
                    root: root.position,
                    height: model.bounds.high[1],
                });
            }
        }
        self.unique.validate()?;
        Ok(())
    }
}

pub(super) fn source_normalization() -> Vec3 {
    Vec3::Y * (TREE_TRUNK_HEIGHT_METRES * 0.5)
}

pub(super) fn prepare(environment: &SceneEnvironment) -> Result<PreparedSpecimens> {
    let detail = ForestDetail::default();
    let mut snapshot = empty_snapshot();
    let mut roots = Vec::new();
    let mut evidence = Vec::new();
    let species = [
        TreePresentationSpecies::EnglishOak,
        TreePresentationSpecies::CommonBeech,
    ];
    for (row, species) in species.into_iter().enumerate() {
        let species_id = SpeciesId::from_word(u16::try_from(row)?);
        snapshot.species.push(species_info(species));
        for slot in 0..OAK_GNARLING_SHOWCASE.len() {
            let seed = oak_variant_seed(slot);
            let id = model_id(species, seed);
            let plant = PlantIndex::from_word(snapshot.plant_bounds.len());
            let root = Vec3::new(
                slot as f32 * SPACING.metres(),
                0.0,
                row as f32 * SPACING.metres(),
            );
            let (branches, cards) = playable_tree_source(
                species,
                seed,
                slot,
                canopy_competition(environment.canopy_bps),
                environment,
            );
            let packed = records::pack(
                &branches,
                &cards,
                plant,
                species_id,
                root + source_normalization(),
                detail.leaf_blades,
            )?;
            evidence.push(SpecimenEvidence {
                source_id: id.word(),
                species,
                authored_seed: seed.to_u64(),
                source_leaf_cards: cards.len(),
                centerline_runs: branches
                    .split_inclusive(|branch| branch.is_limb_tip)
                    .map(|curve| curve.iter().map(BranchEvidence::from).collect())
                    .collect(),
                leaf_groups: packed.groups,
            });
            snapshot.branches.extend(packed.branches);
            snapshot.leaves.extend(packed.leaves);
            snapshot.plant_bounds.push(PlantInstance {
                species: u32::from(species_id.word()),
                ..Default::default()
            });
            roots.push(TreeRoot {
                id,
                position: root.to_array().map(Metres::from_metres),
            });
        }
    }
    snapshot.plants = roots.len();
    snapshot.modules = snapshot.branches.len();
    let unique = PreparedUnique {
        snapshot,
        roots,
        detail,
        label: "Production mature oak/beech source adapter pilot".into(),
        focus: None,
    };
    unique.validate()?;
    Ok(PreparedSpecimens { unique, evidence })
}

fn empty_snapshot() -> Snapshot {
    // Authored isolated review floor, not production terrain or a timing fixture.
    let terrain = TerrainGeometry {
        vertices: [
            [-64.0, 0.0, -64.0],
            [160.0, 0.0, -64.0],
            [-64.0, 0.0, 96.0],
            [160.0, 0.0, 96.0],
        ]
        .map(|position| TerrainVertex {
            position,
            normal: [0.0, 1.0, 0.0],
            color: [0.35, 0.40, 0.28],
        })
        .to_vec(),
        indices: vec![0, 2, 1, 1, 2, 3],
    };
    Snapshot {
        scene_generation: 1,
        time: 0.0,
        plants: 0,
        modules: 0,
        step_ms: 0.0,
        species: Vec::new(),
        plant_bounds: Vec::new(),
        branches: Vec::new(),
        leaves: Vec::new(),
        blend: BlendShapes::default(),
        fillets: false,
        fillet_size: 0.0,
        litter: Litter::default(),
        hierarchy: Default::default(),
        terrain: Some(Arc::new(terrain)),
        structures: None,
        bounds: [[-64.0, -32.0, -64.0], [160.0, 128.0, 96.0]],
        biome: Biome::TemperateSeasonalForest,
    }
}

pub(super) fn model_id(
    species: TreePresentationSpecies,
    seed: fabelgeist_determinism::Seed,
) -> PlantId {
    PlantId::from_word(SOURCE_ID.seed(seed, &[species.cache_salt()]).to_u64())
}

pub(super) fn species_info(species: TreePresentationSpecies) -> SpeciesInfo {
    let style = species.bake_style();
    SpeciesInfo {
        name: species.name().into(),
        bark: style.bark_srgb.map(|value| value / 255.0),
        leaf: style.leaf_srgb.map(|value| value / 255.0),
        foliage: Foliage::Broadleaf,
        plants: OAK_GNARLING_SHOWCASE.len(),
    }
}
