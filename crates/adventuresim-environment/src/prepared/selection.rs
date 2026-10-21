//! Deterministic representative selection in botanical shape space.

use std::collections::BTreeMap;

use bevy::prelude::Vec3;
use fabelgeist_botany::identity::SpeciesId;
use fabelgeist_botany::{
    species::Foliage,
    viewer::sim::{BranchInstance, LeafInstance},
};
use fabelgeist_math::units::Metres;

use super::form::TreeForm;
use super::{
    PreparedError, PreparedUnique, Result, TreeBounds, TreeParts, TreeRoot, VariantBudget,
};
use crate::render::{ForestDetail, levels::Level};

/// Covers Hermite control-point hulls plus the largest end radius.
const HERMITE_REACH: f32 = 1.0 / 3.0;
/// Covers frond droop and coarse needle sprays (including blade enlargement).
const LEAF_REACH: f32 = 8.0;

pub(super) struct SourceTree {
    pub root: TreeRoot,
    pub species: SpeciesId,
    pub branches: Vec<usize>,
    pub leaves: Vec<usize>,
    pub form: TreeForm,
}

impl SourceTree {
    pub fn parts(&self) -> TreeParts {
        TreeParts::from_counts(self.branches.len(), self.leaves.len())
    }
}

pub(super) fn trees(unique: &PreparedUnique) -> Result<Vec<SourceTree>> {
    let snapshot = &unique.snapshot;
    let mut trees: Vec<_> = unique
        .roots
        .iter()
        .zip(&snapshot.plant_bounds)
        .map(|(&root, plant)| {
            let species = u16::try_from(plant.species)
                .map_err(|_| PreparedError::Invalid("species word exceeds catalog identity"))?;
            Ok(SourceTree {
                root,
                species: SpeciesId::from_word(species),
                branches: Vec::new(),
                leaves: Vec::new(),
                form: TreeForm::measure(root.position, std::iter::empty(), std::iter::empty()),
            })
        })
        .collect::<Result<_>>()?;
    for (index, branch) in snapshot.branches.iter().enumerate() {
        let tree = trees
            .get_mut(branch.plant as usize)
            .ok_or(PreparedError::Invalid("branch plant reference"))?;
        tree.branches.push(index);
    }
    for (index, leaf) in snapshot.leaves.iter().enumerate() {
        let tree = trees
            .get_mut(leaf.plant as usize)
            .ok_or(PreparedError::Invalid("leaf plant reference"))?;
        tree.leaves.push(index);
    }
    for tree in &mut trees {
        tree.form = TreeForm::measure(
            tree.root.position,
            tree.branches.iter().map(|&index| &snapshot.branches[index]),
            tree.leaves.iter().map(|&index| &snapshot.leaves[index]),
        );
    }
    Ok(trees)
}

impl TreeBounds {
    pub(super) fn include_branch(&mut self, branch: &BranchInstance) {
        let a = Vec3::from_slice(&branch.a[..3]);
        let b = Vec3::from_slice(&branch.b[..3]);
        let reach = Vec3::splat(a.distance(b) * HERMITE_REACH + branch.a[3].max(branch.b[3]));
        self.include(a.min(b) - reach, a.max(b) + reach);
    }

    pub(super) fn include_leaf(
        &mut self,
        leaf: &LeafInstance,
        foliage: Foliage,
        detail: &ForestDetail,
    ) {
        let reach = if foliage == Foliage::Fronds {
            // leaf.wgsl: odd blade counts can extend beyond the frond's t=1.
            const FROND_LENGTH: f32 = 4.0;
            const MAX_DROOP: f32 = 0.3 + 0.35;
            const HALF_WIDTH: f32 = 0.5 * FROND_LENGTH * 0.3;
            Level::ALL
                .into_iter()
                .map(|level| {
                    let blades = level.shape(detail).blades;
                    let t = ((blades - 1) / 2 + 1) as f32 / (blades / 2).max(1) as f32;
                    FROND_LENGTH * (t + MAX_DROOP * t * t) + HALF_WIDTH
                })
                .fold(0.0, f32::max)
        } else {
            LEAF_REACH
        };
        let center = Vec3::from_array(leaf.center);
        let reach = Vec3::splat(leaf.size * reach);
        self.include(center - reach, center + reach);
    }

    fn include(&mut self, low: Vec3, high: Vec3) {
        self.low = Vec3::from_array(self.low.map(Metres::metres))
            .min(low)
            .to_array()
            .map(Metres::from_metres);
        self.high = Vec3::from_array(self.high.map(Metres::metres))
            .max(high)
            .to_array()
            .map(Metres::from_metres);
    }
}

pub(super) fn choose(trees: &[SourceTree], budget: VariantBudget) -> Result<Vec<usize>> {
    if matches!(budget, VariantBudget::AllPlants) {
        return Ok((0..trees.len()).collect());
    }
    let VariantBudget::PerSpecies(budget) = budget else {
        return Ok(Vec::new());
    };
    let mut groups: BTreeMap<_, Vec<usize>> = BTreeMap::new();
    for (index, tree) in trees.iter().enumerate() {
        groups.entry(tree.species).or_default().push(index);
    }
    let mut output = Vec::new();
    for group in groups.values() {
        let mut states = BTreeMap::new();
        for &index in group {
            states.entry(trees[index].parts()).or_insert(index);
        }
        if states.len() > usize::from(budget.get()) {
            return Err(PreparedError::Invalid(
                "species budget cannot preserve its distinct empty/woody/leafed states",
            ));
        }
        let mut selected: Vec<_> = states.values().copied().collect();
        while selected.len() < group.len().min(usize::from(budget.get())) {
            let next = group
                .iter()
                .copied()
                .filter(|index| !selected.contains(index))
                .max_by(|&a, &b| {
                    nearest(&trees[a], &selected, trees)
                        .total_cmp(&nearest(&trees[b], &selected, trees))
                        .then_with(|| b.cmp(&a))
                });
            let Some(next) = next else {
                break;
            };
            selected.push(next);
        }
        output.extend(selected);
    }
    Ok(output)
}

fn nearest(tree: &SourceTree, selected: &[usize], trees: &[SourceTree]) -> f32 {
    selected
        .iter()
        .filter(|&&index| tree.parts() == trees[index].parts())
        .map(|&index| tree.form.distance(trees[index].form))
        .fold(f32::INFINITY, f32::min)
}
