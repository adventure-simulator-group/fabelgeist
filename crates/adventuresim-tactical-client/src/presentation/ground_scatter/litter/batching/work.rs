//! Count accepted placements and expanded native buffer bytes, never RAM/VRAM.

use super::{LitterBatch, LitterPlacement};
use anyhow::{Context, Result};
use bevy::prelude::{Assets, Component, Mesh, info};
use serde::Serialize;
use std::collections::BTreeMap;

/// Counts are measured at the native Mesh/placement-vector boundary.
#[derive(Default, Clone, Serialize)]
struct LitterFamilyWork {
    patches: usize,
    logical_expanded_bytes: usize,
}

/// Accepted native placement counts; retained only after the spawn completes.
/// Logical expansion bytes describe the old expanded geometry, not RAM/VRAM.
#[derive(Component, Clone, Serialize)]
pub(in crate::presentation) struct AcceptedLitterWork {
    dry_leaves: LitterFamilyWork,
    twigs: LitterFamilyWork,
    woodland_plants: LitterFamilyWork,
    occupied_cells: usize,
}

impl LitterFamilyWork {
    fn include(&mut self, meshes: &Assets<Mesh>, placements: &[LitterPlacement]) -> Result<()> {
        self.patches = self
            .patches
            .checked_add(placements.len())
            .context("litter patch count overflow")?;
        for placement in placements {
            let mesh = meshes
                .get(&placement.source)
                .context("missing litter work prototype")?;
            // Check extraction state at the fallible Mesh boundary first.
            mesh.try_indices_option()?;
            let index_bytes = mesh.get_index_buffer_bytes().map_or(0, |bytes| bytes.len());
            let bytes = mesh
                .get_vertex_buffer_size()
                .checked_add(index_bytes)
                .context("litter native buffer size overflow")?;
            self.logical_expanded_bytes = self
                .logical_expanded_bytes
                .checked_add(bytes)
                .context("litter expanded buffer size overflow")?;
        }
        Ok(())
    }
}

pub(super) fn record(
    meshes: &Assets<Mesh>,
    cells: &BTreeMap<(i32, i32), LitterBatch>,
) -> Result<AcceptedLitterWork> {
    let mut leaves = LitterFamilyWork::default();
    let mut twigs = LitterFamilyWork::default();
    let mut plants = LitterFamilyWork::default();
    for cell in cells.values() {
        leaves.include(meshes, &cell.leaves)?;
        twigs.include(meshes, &cell.twigs)?;
        plants.include(meshes, &cell.plants)?;
    }
    info!(
        dry_leaf_patches = leaves.patches,
        twig_patches = twigs.patches,
        woodland_plant_patches = plants.patches,
        occupied_cells = cells.len(),
        dry_leaf_logical_expanded_bytes = leaves.logical_expanded_bytes,
        twig_logical_expanded_bytes = twigs.logical_expanded_bytes,
        woodland_plant_logical_expanded_bytes = plants.logical_expanded_bytes,
        "Recorded accepted tactical litter workload"
    );
    Ok(AcceptedLitterWork {
        dry_leaves: leaves,
        twigs,
        woodland_plants: plants,
        occupied_cells: cells.len(),
    })
}
