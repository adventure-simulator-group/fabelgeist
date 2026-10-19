//! Native packed-record kernel. Source/model/world positions use metres.
//!
//! Source collar/frame/root-surface differences remain explicit in evidence;
//! only centerline topology and approximate shoot card area are converted.

use std::collections::BTreeMap;

use anyhow::{Result, ensure};
use bevy::prelude::*;
use fabelgeist_botany::{
    identity::{PlantIndex, SpeciesId},
    viewer::{
        sim::{BranchInstance, LeafInstance},
        tangent,
    },
};
use fabelgeist_determinism::StreamId;
use fabelgeist_math::units::Ratio;

use super::super::{TreeBranchSegment, TreeLeaf, geometry::branch_end_tangent, shoot_identity};
use super::evidence::LeafGroupEvidence;

const CLUSTER_SEED: StreamId = StreamId::new("visual.tree.bank.cluster");
// Analytic broadleaf blades have roughly this mean box area / size squared.
// This is an explicit approximation, not a botanical leaf-area measurement.
const MEAN_BLADE_BOX_AREA: Ratio = Ratio::from_ratio(0.25);
const SHOOT_TIP_OFFSET: Ratio = Ratio::from_ratio(0.45);

pub(super) struct PackedTree {
    pub branches: Vec<BranchInstance>,
    pub leaves: Vec<LeafInstance>,
    pub groups: Vec<LeafGroupEvidence>,
}

struct PackedFoliage {
    leaves: Vec<LeafInstance>,
    groups: Vec<LeafGroupEvidence>,
}

pub(super) fn pack(
    branches: &[TreeBranchSegment],
    cards: &[TreeLeaf],
    plant: PlantIndex,
    species: SpeciesId,
    offset_metres: Vec3,
    blades: u32,
) -> Result<PackedTree> {
    let word = u32::try_from(plant.index())?;
    let mut packed = Vec::new();
    for (axis, curve) in branches
        .split_inclusive(|branch| branch.is_limb_tip)
        .enumerate()
    {
        pack_curve(
            &mut packed,
            curve,
            word,
            u32::try_from(axis)?,
            offset_metres,
        )?;
    }
    let foliage = pack_leaves(branches, cards, word, species, offset_metres, blades)?;
    Ok(PackedTree {
        branches: packed,
        leaves: foliage.leaves,
        groups: foliage.groups,
    })
}

fn pack_curve(
    output: &mut Vec<BranchInstance>,
    curve: &[TreeBranchSegment],
    plant: u32,
    axis: u32,
    offset: Vec3,
) -> Result<()> {
    let Some(first) = curve.first() else {
        return Ok(());
    };
    let circumference = std::f32::consts::TAU * first.start_radius;
    ensure!(
        circumference.is_finite() && circumference > 0.0,
        "invalid source limb radius"
    );
    let mut tangent = (first.end - first.start).normalize();
    let mut distance = 0.0;
    for (index, branch) in curve.iter().enumerate() {
        let end_tangent = branch_end_tangent(branch, curve.get(index + 1));
        ensure!(
            tangent.is_finite() && end_tangent.is_finite(),
            "invalid source limb tangent"
        );
        let next_distance = distance + branch.start.distance(branch.end);
        let a = branch.start + offset;
        let b = branch.end + offset;
        output.push(BranchInstance {
            a: [a.x, a.y, a.z, branch.start_radius],
            b: [b.x, b.y, b.z, branch.end_radius],
            tangents: [encode(tangent), encode(end_tangent)],
            v0: distance / circumference,
            v1: next_distance / circumference,
            light: 1.0,
            light_start: 1.0,
            plant,
            axis,
            flags: u32::from(index == 0 && first.depth > 0),
            ..Default::default()
        });
        tangent = end_tangent;
        distance = next_distance;
    }
    Ok(())
}

fn pack_leaves(
    branches: &[TreeBranchSegment],
    cards: &[TreeLeaf],
    plant: u32,
    species: SpeciesId,
    offset: Vec3,
    blades: u32,
) -> Result<PackedFoliage> {
    ensure!(blades > 0, "source cluster requires blades");
    let shoots: BTreeMap<_, _> = branches
        .iter()
        .filter(|b| b.is_limb_tip)
        .map(|branch| (shoot_identity(branch), branch))
        .collect();
    let mut grouped: BTreeMap<u64, Vec<&TreeLeaf>> = BTreeMap::new();
    for card in cards {
        grouped.entry(card.shoot_id).or_default().push(card);
    }
    let mut leaves = Vec::new();
    let mut evidence = Vec::new();
    let mut crown_low = Vec3::splat(f32::INFINITY);
    let mut crown_high = Vec3::splat(f32::NEG_INFINITY);
    for (shoot, cards) in grouped {
        let branch = shoots
            .get(&shoot)
            .ok_or_else(|| anyhow::anyhow!("source foliage has no terminal shoot"))?;
        let group = LeafGroupEvidence::from_cards(shoot, &cards)?;
        let area: f32 = cards.iter().map(|leaf| leaf.length * leaf.width).sum();
        let size = (area / (blades as f32 * MEAN_BLADE_BOX_AREA.ratio())).sqrt();
        let axis = (branch.end - branch.start).normalize();
        let center = cards.iter().map(|leaf| leaf.center).sum::<Vec3>() / cards.len() as f32;
        let tip = center + axis * size * SHOOT_TIP_OFFSET.ratio() + offset;
        let low = Vec3::from_array(
            group
                .logical_rectangles
                .minimum_metres
                .map(|value| value.metres()),
        ) + offset;
        let high = Vec3::from_array(
            group
                .logical_rectangles
                .maximum_metres
                .map(|value| value.metres()),
        ) + offset;
        crown_low = crown_low.min(low);
        crown_high = crown_high.max(high);
        leaves.push(LeafInstance {
            center: tip.to_array(),
            size,
            axis: axis.to_array(),
            light: cards.iter().map(|leaf| leaf.shade).sum::<f32>() / cards.len() as f32,
            species: u32::from(species.word()),
            seed: CLUSTER_SEED.rng(shoot.into(), &[]).inclusive_unit_f32(),
            plant,
            ..Default::default()
        });
        evidence.push(group);
    }
    ensure!(!leaves.is_empty(), "source specimen has no foliage cards");
    let center = (crown_low + crown_high) * 0.5;
    let radius = center.distance(crown_high);
    for leaf in &mut leaves {
        leaf.crown = [center.x, center.y, center.z, radius];
    }
    Ok(PackedFoliage {
        leaves,
        groups: evidence,
    })
}

fn encode(direction: Vec3) -> u32 {
    // Required native vector adapter at the existing GPU packing kernel.
    tangent::pack(fabelgeist_math::Vec3::new(
        direction.x,
        direction.y,
        direction.z,
    ))
}
