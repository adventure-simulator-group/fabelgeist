//! Share static patch geometry behind the original cell visibility gate.

use super::{super::GroundScatterLayer, LitterPlacement};
use anyhow::{Context, Result, ensure};
use bevy::{
    asset::RenderAssetUsages,
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    mesh::VertexAttributeValues,
    pbr::Material,
    prelude::*,
};

pub(in crate::presentation) use visibility::SharedLitterPlugin;

mod visibility;

/// Native renderer inputs for one original static litter cell/family.
pub(super) struct SharedLitterCell<'a> {
    pub placements: &'a [LitterPlacement],
    pub frame: Transform,
    pub layer: GroundScatterLayer,
    pub range: bevy::camera::visibility::VisibilityRange,
    pub label: &'static str,
}

impl SharedLitterCell<'_> {
    pub(super) fn spawn<M: Material>(
        &self,
        commands: &mut Commands,
        meshes: &Assets<Mesh>,
        material: &Handle<M>,
    ) -> Result<()> {
        let Some(bounds) = self.bounds(meshes)? else {
            return Ok(());
        };
        let cell = commands
            .spawn((
                Name::new(self.label),
                self.layer,
                self.frame,
                self.range.clone(),
                bounds,
                Visibility::Inherited,
            ))
            .id();
        let mut patches = Vec::with_capacity(self.placements.len());
        for placement in self.placements {
            patches.push(
                commands
                    .spawn((
                        Name::new(self.label),
                        self.layer,
                        visibility::SharedLitterPatch,
                        ChildOf(cell),
                        Mesh3d(placement.source.clone()),
                        MeshMaterial3d(material.clone()),
                        placement.frame,
                        Visibility::Inherited,
                        bevy::light::NotShadowCaster,
                        // Only the original cell gate decides per-view membership.
                        NoFrustumCulling,
                    ))
                    .id(),
            );
        }
        commands
            .entity(cell)
            .insert(visibility::LitterCellVisibility::new(patches));
        Ok(())
    }

    fn bounds(&self, meshes: &Assets<Mesh>) -> Result<Option<Aabb>> {
        if self.placements.is_empty() {
            return Ok(None);
        }
        let mut minimum = Vec3::splat(f32::INFINITY);
        let mut maximum = Vec3::splat(f32::NEG_INFINITY);
        for placement in self.placements {
            let mesh = meshes
                .get(&placement.source)
                .context("missing shared litter prototype")?;
            ensure!(
                mesh.asset_usage.contains(RenderAssetUsages::all()),
                "shared litter prototypes require CPU and renderer residency"
            );
            let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.try_attribute_option(Mesh::ATTRIBUTE_POSITION)?
            else {
                anyhow::bail!("shared litter prototype requires native f32x3 positions");
            };
            ensure!(!positions.is_empty(), "empty shared litter prototype");
            // Same cell-local point kernel as Mesh::try_transform_by, without
            // duplicating buffers or touching UV metadata.
            for position in positions {
                let point = placement.frame.transform_point(Vec3::from_array(*position));
                minimum = minimum.min(point);
                maximum = maximum.max(point);
            }
        }
        Ok(Some(Aabb::from_min_max(minimum, maximum)))
    }
}
