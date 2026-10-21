//! Offline model localization and regional placement matching.

use fabelgeist_botany::viewer::sim::PlantInstance;
use fabelgeist_math::units::{Metres, Ratio};

use super::form::TreeForm;
use super::selection::{self, SourceTree};
use super::{
    ForestFrame, LibraryId, PreparedError, PreparedUnique, RecordSpan, Result, TreeLibrary,
    TreeModel, TreePlacement, TreePopulation, TreePose, TreeVisibility, TreeYaw, VariantBudget,
    VariantIndex,
};

pub(super) fn library(unique: &PreparedUnique, budget: VariantBudget) -> Result<TreeLibrary> {
    unique.validate()?;
    let trees = selection::trees(unique)?;
    let selected = selection::choose(&trees, budget)?;
    let mut library = TreeLibrary {
        species: unique.snapshot.species.clone(),
        branches: Vec::new(),
        leaves: Vec::new(),
        plants: Vec::new(),
        models: Vec::new(),
        detail: unique.detail,
    };
    for index in selected {
        append(&mut library, unique, &trees[index])?;
    }
    library.validate()?;
    Ok(library)
}

fn append(library: &mut TreeLibrary, unique: &PreparedUnique, tree: &SourceTree) -> Result<()> {
    let root = tree.root.position.map(Metres::metres);
    let ordinal = u32::try_from(library.models.len())
        .map_err(|_| PreparedError::Invalid("model index overflow"))?;
    let branches = span(library.branches.len(), tree.branches.len())?;
    let leaves = span(library.leaves.len(), tree.leaves.len())?;
    for &index in &tree.branches {
        let mut branch = unique.snapshot.branches[index];
        for (axis, root) in root.into_iter().enumerate() {
            branch.a[axis] -= root;
            branch.b[axis] -= root;
        }
        branch.plant = ordinal;
        library.branches.push(branch);
    }
    for &index in &tree.leaves {
        let mut leaf = unique.snapshot.leaves[index];
        for (axis, root) in root.into_iter().enumerate() {
            leaf.center[axis] -= root;
            leaf.crown[axis] -= root;
        }
        leaf.plant = ordinal;
        library.leaves.push(leaf);
    }
    let mut bounds = super::TreeBounds {
        low: [Metres::from_metres(0.0); 3],
        high: [Metres::from_metres(0.0); 3],
    };
    for branch in &library.branches[branches.start as usize..] {
        bounds.include_branch(branch);
    }
    for leaf in &library.leaves[leaves.start as usize..] {
        bounds.include_leaf(
            leaf,
            library.species[tree.species.index()].foliage,
            &library.detail,
        );
    }
    library.plants.push(PlantInstance {
        species: u32::from(tree.species.word()),
        ..Default::default()
    });
    library.models.push(TreeModel {
        source: tree.root.id,
        species: tree.species,
        bounds,
        branches,
        leaves,
    });
    Ok(())
}

fn span(start: usize, count: usize) -> Result<RecordSpan> {
    Ok(RecordSpan {
        start: u32::try_from(start).map_err(|_| PreparedError::Invalid("record start overflow"))?,
        count: u32::try_from(count).map_err(|_| PreparedError::Invalid("record count overflow"))?,
    })
}

pub(super) fn population(
    unique: PreparedUnique,
    library: &TreeLibrary,
    id: LibraryId,
) -> Result<TreePopulation> {
    let trees = selection::trees(&unique)?;
    let forms: Vec<_> = library
        .models
        .iter()
        .map(|model| {
            TreeForm::measure(
                [Metres::from_metres(0.0); 3],
                library.branches[model.branches.start as usize..][..model.branches.count as usize]
                    .iter(),
                library.leaves[model.leaves.start as usize..][..model.leaves.count as usize].iter(),
            )
        })
        .collect();
    let placements = trees
        .iter()
        .map(|tree| {
            let (index, model) = library
                .models
                .iter()
                .enumerate()
                .filter(|(_, model)| model.species == tree.species && model.parts() == tree.parts())
                .min_by(|(a_index, a), (b_index, b)| {
                    (a.source != tree.root.id)
                        .cmp(&(b.source != tree.root.id))
                        .then_with(|| {
                            tree.form
                                .distance(forms[*a_index])
                                .total_cmp(&tree.form.distance(forms[*b_index]))
                        })
                        .then_with(|| a_index.cmp(b_index))
                })
                .ok_or(PreparedError::Invalid(
                    "no matching species/leaf-state representative",
                ))?;
            let form = forms[index];
            let (horizontal, vertical, yaw) = if model.source == tree.root.id {
                (1.0, 1.0, TreeYaw::from_radians(0.0)?)
            } else {
                (
                    tree.form.width.metres() / form.width.metres(),
                    tree.form.height.metres() / form.height.metres(),
                    tree.form.yaw_from(form, tree.root.id)?,
                )
            };
            Ok(TreePlacement {
                id: tree.root.id,
                visibility: TreeVisibility::unbounded(),
                variant: VariantIndex(index),
                pose: TreePose::new(
                    tree.root.position,
                    Ratio::from_ratio(horizontal),
                    Ratio::from_ratio(vertical),
                    yaw,
                )?,
            })
        })
        .collect::<Result<_>>()?;
    let population = TreePopulation {
        library: id,
        placements,
        frame: ForestFrame::from_unique(&unique),
    };
    population.validate(library, id)?;
    Ok(population)
}
