//! Observational JSON boundary for source geometry before approximation.

use anyhow::{Result, bail, ensure};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use fabelgeist_math::units::Metres;
use serde::Serialize;

use super::super::{
    TreeBranchSegment, TreeLeaf, TreePresentationSpecies, procedural_woody_cambered_leaf_mesh,
    procedural_woody_leaf_card_mesh,
};

#[derive(Serialize)]
pub(super) struct SpecimenEvidence {
    pub source_id: u64,
    pub species: TreePresentationSpecies,
    pub authored_seed: u64,
    pub source_leaf_cards: usize,
    pub centerline_runs: Vec<Vec<BranchEvidence>>,
    pub leaf_groups: Vec<LeafGroupEvidence>,
}

#[derive(Serialize)]
pub(super) struct BranchEvidence {
    start_metres: [Metres; 3],
    end_metres: [Metres; 3],
    start_radius: Metres,
    end_radius: Metres,
    depth: u8,
}

#[derive(Serialize)]
pub(super) struct LeafGroupEvidence {
    pub shoot_id: u64,
    pub cards: usize,
    pub logical_rectangles: SourceEnvelope,
    pub flat_cards: SourceEnvelope,
    pub cambered_cards: SourceEnvelope,
}

#[derive(Serialize)]
pub(super) struct SourceEnvelope {
    pub minimum_metres: [Metres; 3],
    pub maximum_metres: [Metres; 3],
}

impl From<&TreeBranchSegment> for BranchEvidence {
    fn from(branch: &TreeBranchSegment) -> Self {
        Self {
            start_metres: branch.start.to_array().map(Metres::from_metres),
            end_metres: branch.end.to_array().map(Metres::from_metres),
            start_radius: Metres::from_metres(branch.start_radius),
            end_radius: Metres::from_metres(branch.end_radius),
            depth: branch.depth,
        }
    }
}

impl LeafGroupEvidence {
    pub(super) fn from_cards(shoot_id: u64, cards: &[&TreeLeaf]) -> Result<Self> {
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for leaf in cards {
            let extent = leaf.right.abs() * leaf.width * 0.5 + leaf.up.abs() * leaf.length * 0.5;
            minimum = minimum.min(leaf.center - extent).min(leaf.petiole_start);
            maximum = maximum.max(leaf.center + extent).max(leaf.petiole_start);
        }
        let source: Vec<_> = cards.iter().map(|leaf| **leaf).collect();
        Ok(Self {
            shoot_id,
            cards: cards.len(),
            logical_rectangles: SourceEnvelope::from_extrema(minimum, maximum),
            flat_cards: SourceEnvelope::from_mesh(&procedural_woody_leaf_card_mesh(&source))?,
            cambered_cards: SourceEnvelope::from_mesh(&procedural_woody_cambered_leaf_mesh(
                &source,
            ))?,
        })
    }
}

impl SourceEnvelope {
    fn from_extrema(minimum: Vec3, maximum: Vec3) -> Self {
        Self {
            minimum_metres: minimum.to_array().map(Metres::from_metres),
            maximum_metres: maximum.to_array().map(Metres::from_metres),
        }
    }

    pub(super) fn from_mesh(mesh: &Mesh) -> Result<Self> {
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            bail!("source mesh has no native metre positions");
        };
        ensure!(!positions.is_empty(), "source mesh is empty");
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for position in positions {
            let point = Vec3::from_array(*position);
            ensure!(point.is_finite(), "source mesh has nonfinite positions");
            minimum = minimum.min(point);
            maximum = maximum.max(point);
        }
        Ok(Self::from_extrema(minimum, maximum))
    }
}
