//! Observer history of actual per-root selection; never drives the producer.

use super::*;
use adventuresim_environment::render::{
    ForestLodProjection, ProjectedRadiusPixels, source_bounds_radius,
};
use bevy::{
    camera::{CameraProjection, primitives::Aabb},
    render::view::ExtractedView,
};

#[derive(Resource, Default)]
pub(in super::super) struct SelectionHistory {
    checkpoint: Option<Arc<CheckpointSelection>>,
}

#[derive(Clone, PartialEq)]
pub(in super::super) struct CheckpointSelection {
    pub view: usize,
    pub camera: CameraSnapshot,
    roots: Vec<RootSelection>,
}

pub(super) struct SelectionCamera<'a> {
    pub entity: Entity,
    pub camera: &'a Camera,
    pub transform: &'a GlobalTransform,
    pub projection: &'a Projection,
}

/// Native camera framework values, sealed after projection/transform update.
#[derive(Clone, PartialEq)]
pub(in super::super) struct CameraSnapshot {
    pub entity: Entity,
    world_from_view: Mat4,
    clip_from_view: Mat4,
    viewport: UVec4,
}

#[derive(Clone, PartialEq)]
struct RootSelection {
    root: Entity,
    previous: Option<Level>,
    selected: Level,
    projected_radius: ProjectedRadiusPixels,
    group: Entity,
}

impl SelectionHistory {
    pub(super) fn inspect(
        &self,
        view: usize,
        held: &census::HeldScene,
        inspection: &TreeCacheInspection,
        source: SelectionCamera,
    ) -> Result<Arc<CheckpointSelection>> {
        let SelectionCamera {
            entity,
            camera,
            transform,
            projection,
        } = source;
        let Projection::Perspective(perspective) = projection else {
            anyhow::bail!("adaptive witness requires perspective camera");
        };
        ensure!(camera.is_active, "adaptive witness camera inactive");
        let viewport = camera
            .physical_viewport_rect()
            .context("adaptive witness viewport")?;
        let physical = viewport.size();
        let scale = ForestLodProjection::from_perspective(perspective, physical)
            .context("adaptive witness pixel projection")?;
        let library = held.assets.catalogue().bank_assets().bank().library();
        let mut roots = Vec::with_capacity(held.roots.len());
        for root in &held.roots {
            let identity = &root.attachment;
            let model = library
                .models
                .get(identity.source.variant.index())
                .context("adaptive witness source envelope")?;
            let low = Vec3::from_array(model.bounds.low.map(|value| value.metres()));
            let high = Vec3::from_array(model.bounds.high.map(|value| value.metres()));
            let origin = identity.frame.translation
                - Vec3::Y * (adventuresim_tactical_core::prelude::TREE_TRUNK_HEIGHT_METRES * 0.5);
            let bounds = Aabb::from_min_max(low + origin, high + origin);
            let pixels = scale
                .projected_radius(
                    transform.translation(),
                    &bounds,
                    source_bounds_radius(low, high),
                )
                .context("adaptive witness projected radius")?;
            let previous = self.previous(view, identity.root)?;
            let expected = Level::for_projected_radius(
                pixels,
                previous.unwrap_or(Level::Coarse),
                &library.detail,
            );
            let selected = inspection
                .grown_level(identity.root)?
                .context("adaptive selection unavailable")?;
            ensure!(
                selected == expected,
                "actual grown root level differs from measured camera/hysteresis"
            );
            let group = root
                .groups
                .iter()
                .find(|group| group.level == selected)
                .context("selected group not installed")?
                .entity;
            roots.push(RootSelection {
                root: identity.root,
                previous,
                selected,
                projected_radius: pixels,
                group,
            });
        }
        Ok(Arc::new(CheckpointSelection {
            view,
            camera: CameraSnapshot {
                entity,
                world_from_view: transform.to_matrix(),
                clip_from_view: perspective.get_clip_from_view(),
                viewport: UVec4::new(viewport.min.x, viewport.min.y, physical.x, physical.y),
            },
            roots,
        }))
    }

    pub(in super::super) fn seal(&mut self, checkpoint: Arc<CheckpointSelection>) -> Result<()> {
        if let Some(held) = &self.checkpoint {
            if held.view == checkpoint.view {
                ensure!(
                    held == &checkpoint,
                    "held adaptive camera/selection changed"
                );
                return Ok(());
            }
            ensure!(
                held.view.checked_add(1) == Some(checkpoint.view),
                "adaptive checkpoint skipped its prefix"
            );
        } else {
            ensure!(
                checkpoint.view == 0,
                "adaptive history must start with coarse warmup"
            );
        }
        self.checkpoint = Some(checkpoint);
        Ok(())
    }

    fn previous(&self, view: usize, root: Entity) -> Result<Option<Level>> {
        let Some(checkpoint) = &self.checkpoint else {
            return Ok(None);
        };
        let held = checkpoint
            .roots
            .iter()
            .find(|held| held.root == root)
            .context("adaptive history root missing")?;
        Ok(if checkpoint.view == view {
            held.previous
        } else {
            Some(held.selected)
        })
    }
}

impl CheckpointSelection {
    pub(in super::super) fn report(&self) -> serde_json::Value {
        serde_json::json!({"view":self.view,"camera":self.camera.report(),"roots":self.roots.iter().map(|root|serde_json::json!({"root_bits":root.root.to_bits(),"previous":root.previous,"selected":root.selected,"projected_radius_pixels":root.projected_radius,"group_entity_bits":root.group.to_bits()})).collect::<Vec<_>>(),"limits":"actual post-propagation camera/selected groups and independent shared-policy check; producer remains the only selection authority"})
    }
}

impl CameraSnapshot {
    pub(in super::super) fn check_extracted(&self, view: &ExtractedView) -> Result<()> {
        ensure!(
            view.world_from_view.to_matrix() == self.world_from_view
                && view.clip_from_view == self.clip_from_view
                && view.viewport == self.viewport,
            "adaptive render-extracted camera differs from selected camera"
        );
        // The bounded witness admits standard gameplay-camera extraction,
        // whose Bevy owner supplies no clip-from-world override.
        ensure!(
            view.clip_from_world.is_none(),
            "adaptive witness does not admit a clip-from-world override"
        );
        Ok(())
    }

    fn report(&self) -> serde_json::Value {
        serde_json::json!({"entity_bits":self.entity.to_bits(),"world_from_view":self.world_from_view.to_cols_array(),"clip_from_view":self.clip_from_view.to_cols_array(),"viewport":self.viewport.to_array(),"scope":"native post-update camera and actual render-extracted join, not GPU view-uniform bytes"})
    }
}
