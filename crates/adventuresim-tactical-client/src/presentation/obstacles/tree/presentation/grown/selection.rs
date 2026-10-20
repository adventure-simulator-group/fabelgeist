//! Select installed grown levels after the actual camera has propagated.

use super::streaming::GrownLevelGroup;
use super::*;
use crate::presentation::TacticalGameplayCamera;
use crate::presentation::obstacles::tree::presentation::{
    StreamedTreePresentation, TreePresentationCache,
};
use adventuresim_environment::render::{ForestLodProjection, source_bounds_radius};
use adventuresim_tactical_core::prelude::SceneObstacleOf;
use bevy::{camera::primitives::Aabb, ecs::system::SystemParam};

#[derive(SystemParam)]
pub(in crate::presentation) struct GrownSelectionAssets<'w, 's> {
    scenes: Query<'w, 's, &'static SceneEnvironment>,
    frames: Query<'w, 's, &'static ChildOf, With<GrownVisualFrame>>,
    roots: Query<
        'w,
        's,
        (
            &'static Transform,
            Option<&'static SceneObstacleOf>,
            Has<ChildOf>,
        ),
    >,
    groups: Query<'w, 's, (&'static GrownLevelGroup, &'static mut Visibility)>,
}

impl GrownLevels<Entity> {
    fn select(
        &self,
        root: Entity,
        selected: Option<Level>,
        assets: &mut GrownSelectionAssets,
    ) -> Result<()> {
        for level in [Level::Full, Level::Middle, Level::Coarse] {
            let (binding, mut visibility) = assets.groups.get_mut(*self.get(level))?;
            ensure!(
                binding.root == root && binding.level == level,
                "grown detail group identity changed"
            );
            let desired = if selected == Some(level) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *visibility != desired {
                *visibility = desired;
            }
        }
        Ok(())
    }
}

pub(in crate::presentation) fn install_grown_streaming(app: &mut App) {
    app.add_systems(
        PostUpdate,
        stream_grown_tree_levels
            .after(bevy::transform::TransformSystems::Propagate)
            .after(bevy::camera::CameraUpdateSystems)
            .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate)
            .before(bevy::camera::visibility::VisibilitySystems::CheckVisibility),
    );
}

fn stream_grown_tree_levels(
    cameras: Query<(&Camera, &GlobalTransform, &Projection), With<TacticalGameplayCamera>>,
    mut trees: Query<(Entity, &mut StreamedTreePresentation)>,
    cache: Res<TreePresentationCache>,
    mut assets: GrownSelectionAssets,
) -> bevy::prelude::Result {
    let mut active = cameras.iter().filter(|(camera, _, _)| camera.is_active);
    let view = active.next().and_then(|(camera, transform, projection)| {
        if active.next().is_some() {
            return None;
        }
        let Projection::Perspective(perspective) = projection else {
            return None;
        };
        let projection =
            ForestLodProjection::from_perspective(perspective, camera.physical_viewport_size()?)?;
        Some((projection, transform.translation()))
    });
    for (entity, mut presentation) in &mut trees {
        let StreamedTreePresentation::Grown(stream) = &mut *presentation else {
            continue;
        };
        let environment = assets.scenes.get(stream.scene)?;
        let resident = cache.grown_for_scene(stream.scene, environment)?;
        let (root, owner, parented) = assets.roots.get(entity)?;
        stream.verify(
            resident,
            root,
            owner,
            parented,
            assets.frames.get(stream.visual)?.parent(),
            entity,
        )?;
        if resident.selection != LevelSelection::Adaptive {
            continue;
        }
        let selected = view
            .map(|(projection, eye)| select_level(stream, resident, projection, eye))
            .transpose()?
            .flatten();
        let groups = stream
            .groups
            .as_ref()
            .context("adaptive grown groups missing")?;
        groups.select(entity, selected, &mut assets)?;
        stream.level = selected;
    }
    Ok(())
}

fn select_level(
    stream: &GrownTreeStream,
    cache: &GrownTreeCache,
    projection: ForestLodProjection,
    eye: Vec3,
) -> Result<Option<Level>> {
    let library = cache.catalogue.bank_assets().bank().library();
    let model = library
        .models
        .get(stream.source.variant.index())
        .context("grown LOD source bounds")?;
    // The library corners and physical-root translation meet in world metres.
    // Remove the collider-centre normalization at its existing visual owner.
    let low = Vec3::from_array(model.bounds.low.map(|value| value.metres()));
    let high = Vec3::from_array(model.bounds.high.map(|value| value.metres()));
    let translation =
        stream.root_frame.translation - super::super::super::source::source_normalization();
    let bounds = Aabb::from_min_max(low + translation, high + translation);
    Ok(projection
        .projected_radius(eye, &bounds, source_bounds_radius(low, high))
        .map(|pixels| {
            Level::for_projected_radius(
                pixels,
                stream.level.unwrap_or(Level::Coarse),
                &library.detail,
            )
        }))
}
