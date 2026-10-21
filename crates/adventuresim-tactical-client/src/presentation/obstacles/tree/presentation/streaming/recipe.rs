//! Recipe-only demand and asset handoff, preserving its existing tier policy.

use super::*;
use anyhow::Context;
use bevy::ecs::system::SystemParam;

pub(super) struct RecipeDemand {
    mask: u8,
    selected_leaf: Option<TreeLeafRepresentation>,
}

#[derive(SystemParam)]
pub(in crate::presentation) struct RecipeStreamingAssets<'w> {
    meshes: ResMut<'w, Assets<Mesh>>,
    tree_materials: ResMut<'w, Assets<TacticalTreeImpostorMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    residency: ResMut<'w, RecipeTreeAssetResidencyDiagnostics>,
    prepared: PreparedTreeImpostors<'w>,
}

impl RecipeDemand {
    /// Native world-space metres and a dimensionless camera focal ratio enter
    /// the existing Recipe tier-selection numerical kernel at this boundary.
    pub(super) fn for_distance_metres(
        distance_metres: f32,
        focal_scale: f32,
        override_lod: &TreeLodRenderOverride,
    ) -> Self {
        let scaled = distance_metres / focal_scale.max(0.25);
        let mask = if let Some(lod) = override_lod.lod {
            (1 << (lod + 1)) | u8::from(lod < 4)
        } else {
            tree_streaming_mask(scaled)
        };
        let selected_leaf = override_lod
            .leaf
            .or_else(|| tree_streamed_leaf_representation(scaled));
        Self {
            mask,
            selected_leaf,
        }
    }
}

impl RecipeStreamingAssets<'_> {
    pub(super) fn stream(
        &mut self,
        commands: &mut Commands,
        entity: Entity,
        presentation: &mut RecipeTreeStream,
        demand: RecipeDemand,
        recipe: &mut RecipeTreeCache,
    ) -> bevy::prelude::Result {
        let RecipeDemand {
            mask,
            selected_leaf,
        } = demand;
        if presentation.active_mask == mask && presentation.active_leaf == selected_leaf {
            return Ok(());
        }
        let cached = recipe
            .variants
            .get_mut(&presentation.cache_key)
            .context("streamed tree cache entry remains resident")?;
        ensure_tree_assets_resident(
            cached,
            mask,
            selected_leaf,
            self.prepared.get(),
            &mut self.meshes,
            &mut self.tree_materials,
            &mut self.images,
            &mut self.residency,
        )?;
        let desired_leaf_mask = if mask & (1 << 1) == 0 {
            0
        } else {
            match selected_leaf {
                Some(TreeLeafRepresentation::TexturedMesh) => 1,
                Some(TreeLeafRepresentation::AlphaCard) => 2,
                None => 3,
            }
        };
        let new_mask = mask & !presentation.resident_mask;
        let new_leaf_mask = desired_leaf_mask & !presentation.resident_leaf_mask;
        children::spawn_streamed_tree_children(commands, entity, cached, new_mask, new_leaf_mask)?;
        presentation.resident_mask |= new_mask;
        presentation.resident_leaf_mask |= new_leaf_mask;
        presentation.active_mask = mask;
        presentation.active_leaf = selected_leaf;
        Ok(())
    }
}
