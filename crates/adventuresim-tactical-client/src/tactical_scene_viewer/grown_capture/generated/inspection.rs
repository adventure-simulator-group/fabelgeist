//! Recheck the complete physical census before selecting visible GPU demand.
use super::super::{
    background,
    ordinary_batches::inventory::{CurrentInventory, RootMember, RootOrigin, SourceBinding},
};
use super::*;
use crate::presentation::{
    GrownPartBinding, PresentedTree, TacticalGameplayCamera, VistaTerrainMesh,
};
use adventuresim_environment::render::grown::{GrownLeafMaterial, GrownWoodMaterial};
use adventuresim_tactical_core::prelude::{
    Collider, CollisionLayers, LayerMask, RigidBody, SceneObstacle, TACTICAL_TERRAIN_LAYER,
};
use bevy::{
    camera::{
        primitives::Aabb,
        visibility::{NoAutoAabb, VisibleEntities},
    },
    ecs::system::SystemParam,
    render::batching::NoAutomaticBatching,
};

use adventuresim_tactical_core::prelude::SceneObstacleOf;
#[derive(bevy::ecs::query::QueryData)]
struct Physics {
    kind: &'static SceneObstacle,
    frame: &'static Transform,
    global: &'static GlobalTransform,
    body: &'static RigidBody,
    layers: &'static CollisionLayers,
    collider: Has<Collider>,
    parent: Option<&'static ChildOf>,
    owner: &'static SceneObstacleOf,
    presented: Has<PresentedTree>,
    visibility: Option<&'static Visibility>,
    children: Option<&'static Children>,
}

#[derive(bevy::ecs::query::QueryData)]
struct Visual {
    local: &'static Transform,
    global: &'static GlobalTransform,
    parent: &'static ChildOf,
    children: &'static Children,
    visibility: &'static Visibility,
}

#[derive(bevy::ecs::query::QueryData)]
struct Part {
    binding: &'static GrownPartBinding,
    mesh: &'static Mesh3d,
    local: &'static Transform,
    global: &'static GlobalTransform,
    parent: &'static ChildOf,
    children: Option<&'static Children>,
    visibility: &'static Visibility,
    bounds: &'static Aabb,
    source_bounds: Has<NoAutoAabb>,
    singleton: Has<NoAutomaticBatching>,
    no_shadow: Has<bevy::light::NotShadowCaster>,
    wood: Option<&'static MeshMaterial3d<GrownWoodMaterial>>,
    leaves: Option<&'static MeshMaterial3d<GrownLeafMaterial>>,
}

#[derive(SystemParam)]
pub(in super::super) struct Inspector<'w, 's> {
    held: Option<Res<'w, census::HeldScene>>,
    history: Option<Res<'w, selection::SelectionHistory>>,
    demand: Option<Res<'w, demand::Demand>>,
    cache: TreeCacheInspection<'w, 's>,
    physics: Query<'w, 's, Physics>,
    obstacles: Query<'w, 's, Entity, With<SceneObstacle>>,
    presented: Query<'w, 's, Entity, With<PresentedTree>>,
    visual: Query<'w, 's, Visual>,
    parts: Query<'w, 's, Part>,
    part_entities: Query<'w, 's, Entity, With<GrownPartBinding>>,
    camera: Query<
        'w,
        's,
        (
            Entity,
            &'static Camera,
            &'static GlobalTransform,
            &'static Projection,
            &'static VisibleEntities,
        ),
        With<TacticalGameplayCamera>,
    >,
    wood: Option<Res<'w, Assets<GrownWoodMaterial>>>,
    leaves: Option<Res<'w, Assets<GrownLeafMaterial>>>,
    vista_visibility: Query<'w, 's, &'static Visibility, With<VistaTerrainMesh>>,
    support: cameras::SupportInspector<'w, 's>,
    background: background::Inspector<'w, 's>,
    vista: vista::Inspector<'w, 's>,
}

impl Inspector<'_, '_> {
    pub(in super::super) fn active(&self) -> bool {
        self.held.is_some()
    }

    pub(in super::super) fn check(&self, state: &SceneCaptureState) -> Result<CurrentInventory> {
        self.support.check(&state.digest)?;
        ensure!(
            !self.vista_visibility.is_empty()
                && self
                    .vista_visibility
                    .iter()
                    .all(|visibility| *visibility != Visibility::Hidden),
            "generated scene must retain visible vista terrain"
        );
        let held = self.held.as_deref().context("generated census missing")?;
        let actual = self.cache.grown_scene()?;
        ensure!(
            actual.scene() == held.assets.scene()
                && actual.selection() == held.assets.selection()
                && Arc::ptr_eq(actual.catalogue(), held.assets.catalogue()),
            "generated cache owner changed"
        );
        let wood = self.wood.as_deref().context("generated wood store")?;
        let leaves = self.leaves.as_deref().context("generated leaf store")?;
        for source in actual.catalogue().sources() {
            for level in [Level::Full, Level::Middle, Level::Coarse] {
                let assets = actual.catalogue().assets(*source, level)?;
                ensure!(
                    SourceBinding::from_assets(assets) == held.buffers,
                    "generated source buffers changed"
                );
                assets.validate_materials(wood, leaves)?;
            }
        }
        ensure!(
            self.obstacles.iter().count() == held.obstacles.len()
                && self.presented.iter().count() == held.roots.len(),
            "generated physical/presented census changed"
        );
        for obstacle in &held.obstacles {
            let actual = self.physics.get(obstacle.entity)?;
            ensure!(
                *actual.kind == obstacle.kind
                    && *actual.frame == obstacle.collider_frame
                    && actual.global.to_matrix() == obstacle.collider_frame.to_matrix()
                    && *actual.body == RigidBody::Static
                    && actual.collider
                    && actual.parent.is_none()
                    && actual.owner.0 == held.assets.scene()
                    && *actual.layers
                        == CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
                "original generated collider root changed"
            );
        }
        let (camera_entity, camera, transform, projection, visible_entities) =
            self.camera.single()?;
        let visible = visible_entities.get(std::any::TypeId::of::<Mesh3d>());
        let mut inventory = CurrentInventory {
            buffers: held.buffers.clone(),
            demand: super::super::ordinary_batches::inventory::DrawDemand::CameraVisible,
            ..default()
        };
        for root in &held.roots {
            let selected = self.root(root)?;
            inventory.roots.push(RootMember {
                entity: root.attachment.root,
                origin: RootOrigin::Generated {
                    source: root.attachment.source,
                },
                frame: root.attachment.frame.to_matrix(),
            });
            for part in root.parts() {
                self.part(part)?;
                if part.binding.level != selected {
                    ensure!(
                        !visible.contains(&part.member.entity),
                        "inactive grown level is natively visible"
                    );
                    continue;
                }
                if visible.contains(&part.member.entity) {
                    inventory.visible.push(part.member.entity);
                }
                inventory.members.push(part.member);
            }
        }
        ensure!(
            self.part_entities.iter().count()
                == held
                    .roots
                    .iter()
                    .map(|root| root.parts().count())
                    .sum::<usize>(),
            "extra/missing generated part entity"
        );
        self.vista.check(
            &held.vista,
            &actual,
            visible_entities,
            self.demand.as_deref(),
            &mut inventory,
        )?;
        inventory
            .members
            .sort_by_key(|member| member.entity.to_bits());
        inventory.visible.sort_by_key(|entity| entity.to_bits());
        inventory.background = self.background.collect(visible_entities)?;
        if actual.selection() == LevelSelection::Adaptive {
            inventory.selection = Some(
                self.history
                    .as_deref()
                    .context("adaptive selection history missing")?
                    .inspect(
                        state.view,
                        held,
                        &self.cache,
                        selection::SelectionCamera {
                            entity: camera_entity,
                            camera,
                            transform,
                            projection,
                        },
                    )?,
            );
        }
        inventory.validate_capacity()?;
        inventory.valid = true;
        Ok(inventory)
    }

    fn root(&self, root: &census::Root) -> Result<Level> {
        let held = &root.attachment;
        let selected = self
            .cache
            .grown_level(held.root)?
            .context("grown selected level awaits a valid camera")?;
        ensure!(
            self.cache.grown_root(held.root)? == *held
                && self
                    .held
                    .as_ref()
                    .is_some_and(|scene| match scene.assets.selection() {
                        adventuresim_environment::render::LevelSelection::Fixed(level) =>
                            selected == level,
                        adventuresim_environment::render::LevelSelection::Adaptive => true,
                    }),
            "generated stream attachment changed"
        );
        let physics = self.physics.get(held.root)?;
        ensure!(
            physics.presented
                && (matches!(
                    physics.visibility,
                    Some(Visibility::Inherited | Visibility::Visible)
                ) || physics.visibility == Some(&Visibility::Hidden)
                    && self
                        .demand
                        .as_deref()
                        .is_some_and(|demand| demand.allows_hidden(held.root)))
                && physics
                    .children
                    .is_some_and(|children| children.len() == 1 && children.contains(&held.visual)),
            "generated root visual ownership/visibility changed"
        );
        let visual = self.visual.get(held.visual)?;
        let offset =
            -Vec3::Y * (adventuresim_tactical_core::prelude::TREE_TRUNK_HEIGHT_METRES * 0.5);
        ensure!(
            *visual.local == Transform::from_translation(offset)
                && visual.global.to_matrix()
                    == Transform::from_translation(held.frame.translation + offset).to_matrix()
                && visual.parent.parent() == held.root
                && *visual.visibility == Visibility::Inherited
                && visual.children.len()
                    == if root.groups.len() == 1 {
                        root.parts().count()
                    } else {
                        root.groups.len()
                    }
                && if root.groups.len() == 1 {
                    root.parts()
                        .all(|part| visual.children.contains(&part.member.entity))
                } else {
                    root.groups
                        .iter()
                        .all(|group| visual.children.contains(&group.entity))
                },
            "generated visual frame/hierarchy changed"
        );
        if root.groups.len() > 1 {
            for group in &root.groups {
                let actual = self.visual.get(group.entity)?;
                ensure!(
                    *actual.local == Transform::IDENTITY
                        && actual.global.to_matrix() == visual.global.to_matrix()
                        && actual.parent.parent() == held.visual
                        && *actual.visibility
                            == if group.level == selected {
                                Visibility::Inherited
                            } else {
                                Visibility::Hidden
                            }
                        && actual.children.len() == group.parts.len()
                        && group
                            .parts
                            .iter()
                            .all(|part| actual.children.contains(&part.member.entity)),
                    "grown retained level group frame/visibility/hierarchy changed"
                );
            }
        }
        Ok(selected)
    }

    fn part(&self, held: &census::Part) -> Result<()> {
        let actual = self.parts.get(held.member.entity)?;
        let material = match (actual.wood, actual.leaves) {
            (Some(wood), None) => wood.0.id().untyped(),
            (None, Some(leaves)) => leaves.0.id().untyped(),
            _ => anyhow::bail!("generated material component ambiguous"),
        };
        ensure!(
            *actual.binding == held.binding
                && actual.mesh.0.id() == held.member.mesh
                && material == held.member.material
                && *actual.local == Transform::IDENTITY
                && actual.global.to_matrix() == held.member.transform
                && actual.parent.parent() == held.parent
                && actual.children.is_none_or(|children| children.is_empty())
                && *actual.visibility == Visibility::Inherited
                && actual.singleton
                    == self
                        .demand
                        .as_deref()
                        .is_some_and(|demand| demand.allows_singleton(held.member.entity))
                && !actual.no_shadow
                && actual.source_bounds
                && actual.bounds.center == held.bounds.center
                && actual.bounds.half_extents == held.bounds.half_extents,
            "generated part mesh/material/frame/bounds/policy changed"
        );
        Ok(())
    }
}
