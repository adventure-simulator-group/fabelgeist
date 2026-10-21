//! Reserve exact native batch capacity before the original Bevy transforms/merges.

use anyhow::{Context, Result, bail, ensure};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, VertexAttributeValues},
    prelude::{Assets, Handle, Mesh, Transform},
};

/// Original patch frame in its 24-metre batch cell; UV1 remains patch-local.
pub(crate) struct LitterPlacement {
    pub(super) source: Handle<Mesh>,
    pub(super) frame: Transform,
}

/// Checked renderer-buffer extents for one complete cell/family mesh.
struct LitterCapacity {
    vertices: usize,
    indices: usize,
}

impl LitterCapacity {
    fn from_placements(meshes: &Assets<Mesh>, placements: &[LitterPlacement]) -> Result<Self> {
        let mut capacity = Self {
            vertices: 0,
            indices: 0,
        };
        for placement in placements {
            let source = meshes
                .get(&placement.source)
                .context("missing litter prototype")?;
            capacity.vertices = capacity
                .vertices
                .checked_add(source.count_vertices())
                .context("litter vertex capacity overflow")?;
            let Some(Indices::U32(indices)) = source.try_indices_option()? else {
                bail!("litter prototypes require native u32 indices");
            };
            capacity.indices = capacity
                .indices
                .checked_add(indices.len())
                .context("litter index capacity overflow")?;
        }
        ensure!(
            u32::try_from(capacity.vertices).is_ok(),
            "litter batch exceeds native index range"
        );
        Ok(capacity)
    }

    fn reserve(&self, mesh: &mut Mesh) -> Result<()> {
        for (_, values) in mesh.try_attributes_mut()? {
            let additional = self
                .vertices
                .checked_sub(values.len())
                .context("litter prototype channel exceeds batch capacity")?;
            match values {
                VertexAttributeValues::Float32x2(values) => values.try_reserve_exact(additional)?,
                VertexAttributeValues::Float32x3(values) => values.try_reserve_exact(additional)?,
                VertexAttributeValues::Float32x4(values) => values.try_reserve_exact(additional)?,
                _ => bail!("unsupported litter prototype vertex channel"),
            }
        }
        let Some(Indices::U32(indices)) = mesh.try_indices_mut_option()? else {
            bail!("litter batch requires native u32 indices");
        };
        indices.try_reserve_exact(
            self.indices
                .checked_sub(indices.len())
                .context("litter prototype indices exceed batch capacity")?,
        )?;
        Ok(())
    }
}

pub(super) fn assemble(
    meshes: &Assets<Mesh>,
    placements: &[LitterPlacement],
) -> Result<Option<Mesh>> {
    let Some((first, remaining)) = placements.split_first() else {
        return Ok(None);
    };
    let capacity = LitterCapacity::from_placements(meshes, placements)?;
    let mut batch = meshes
        .get(&first.source)
        .context("missing first litter prototype")?
        .clone();
    batch.asset_usage = RenderAssetUsages::RENDER_WORLD;
    capacity.reserve(&mut batch)?;
    batch.try_transform_by(first.frame)?;
    for placement in remaining {
        let source = meshes
            .get(&placement.source)
            .context("missing litter prototype")?;
        // Preserve Bevy's exact transform/normal and index-rebasing kernels.
        let transformed = source.clone().try_transformed_by(placement.frame)?;
        batch.merge(&transformed)?;
    }
    Ok(Some(batch))
}
