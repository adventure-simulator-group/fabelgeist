//! World-space pickup and door targeting shared by grab input and outlines.
use super::EquipmentHand;

use adventuresim_tactical_core::prelude::*;
use bevy::{
    ecs::system::SystemParam,
    math::bounding::{Aabb3d, RayCast3d},
    prelude::*,
};
use bevy_mod_outline::OutlineVolume;

use crate::{
    presentation::{GrabTargetOutline, TacticalGameplayCamera},
    targeting::auto_aim_candidate,
};

use super::{GrabSelection, GrabSession, PICKUP_RANGE_M};

#[derive(SystemParam)]
pub(super) struct WorldGrabTargets<'w, 's> {
    cameras: Query<'w, 's, &'static GlobalTransform, With<TacticalGameplayCamera>>,
    scene_items: Query<
        'w,
        's,
        (
            Entity,
            &'static GlobalTransform,
            &'static TacticalEquipmentPhysical,
        ),
        With<TacticalSceneItem>,
    >,
    doors: Query<'w, 's, (Entity, &'static GlobalTransform, &'static SceneDoor)>,
    windows: Query<'w, 's, (Entity, &'static GlobalTransform, &'static SceneWindow)>,
    spatial: SpatialQuery<'w, 's>,
}

impl WorldGrabTargets<'_, '_> {
    pub(super) fn pointed(&self, actor: &GlobalTransform) -> Option<GrabSelection> {
        let camera = self.cameras.single().ok()?;
        let origin = actor.translation() + Vec3::Y * 0.6;
        auto_aim_candidate(
            camera.translation(),
            camera.forward().as_vec3(),
            actor.translation(),
            PICKUP_RANGE_M,
            self.scene_items
                .iter()
                .filter_map(|(entity, transform, physical)| {
                    let position = transform.transform_point(-physical.anchor_offset_m);
                    (self.visible(origin, position) && !self.behind_closed_leaf(origin, position))
                        .then_some((GrabSelection::SceneItem(entity), position, entity.to_bits()))
                })
                .chain(self.doors.iter().filter_map(|(entity, transform, door)| {
                    can_grab_door_from_inside(
                        actor.translation(),
                        door.doorway_centre_metres.metres(),
                        door.tangent.vector(),
                        door.outward.vector(),
                        door.size_metres.metres().x * 0.5,
                    )
                    .then(|| transform.translation())
                    .filter(|position| self.visible(origin, *position))
                    .map(|position| (GrabSelection::Door(entity), position, entity.to_bits()))
                }))
                .chain(
                    self.windows
                        .iter()
                        .filter_map(|(entity, transform, window)| {
                            can_grab_window_from_inside(
                                actor.translation(),
                                window.opening_centre_metres.metres(),
                                window.tangent.vector(),
                                window.outward.vector(),
                                window.size_metres.metres().x * 0.5,
                            )
                            .then(|| transform.translation())
                            .filter(|position| self.visible(origin, *position))
                            .map(|position| {
                                (GrabSelection::Window(entity), position, entity.to_bits())
                            })
                        }),
                ),
        )
    }

    fn visible(&self, origin: Vec3, position: Vec3) -> bool {
        let sight = position - origin;
        let distance = sight.length();
        distance > 0.0
            && Dir3::new(sight / distance).is_ok_and(|direction| {
                self.spatial
                    .cast_ray(
                        origin,
                        direction,
                        distance,
                        true,
                        &SpatialQueryFilter::from_mask(TACTICAL_TERRAIN_LAYER),
                    )
                    .is_none()
            })
    }

    /// The server's item sight also stops at door and window leaves. The client
    /// has no leaf colliders, because its character controller would then
    /// mispredict door passage, so leaves are tested from their replicated
    /// poses instead.
    fn behind_closed_leaf(&self, origin: Vec3, position: Vec3) -> bool {
        self.doors
            .iter()
            .map(|(_, pose, door)| (pose, door.size_metres.metres()))
            .chain(
                self.windows
                    .iter()
                    .map(|(_, pose, window)| (pose, window.size_metres.metres())),
            )
            .any(|(pose, size)| leaf_occludes(origin, position, pose, size))
    }
}

/// Whether the sight segment crosses a leaf box of `size` centred on `pose`.
fn leaf_occludes(origin: Vec3, position: Vec3, pose: &GlobalTransform, size: Vec3) -> bool {
    let (_, rotation, centre) = pose.to_scale_rotation_translation();
    let inverse = rotation.inverse();
    let sight = inverse * (position - origin);
    let Ok(direction) = Dir3::new(sight) else {
        return false;
    };
    RayCast3d::new(inverse * (origin - centre), direction, sight.length())
        .aabb_intersection_at(&Aabb3d::new(Vec3::ZERO, size * 0.5))
        .is_some()
}

pub(super) fn world_grab_selection(
    selection: Option<GrabSelection>,
    hand_occupied: bool,
    pointed: Option<GrabSelection>,
) -> Option<GrabSelection> {
    if hand_occupied
        || !matches!(
            selection,
            None | Some(
                GrabSelection::SceneItem(_) | GrabSelection::Door(_) | GrabSelection::Window(_)
            )
        )
    {
        return selection;
    }
    pointed
}

pub(super) fn update_pickup_outlines(
    session: Res<GrabSession>,
    mut outlines: Query<(&GrabTargetOutline, &mut OutlineVolume)>,
) {
    for (outline, mut volume) in &mut outlines {
        volume.visible = grab_target_outline_selected(outline.0, session.selection);
    }
}

pub(super) fn grab_target_outline_selected(
    target: Entity,
    selection: Option<GrabSelection>,
) -> bool {
    matches!(
        selection,
        Some(
            GrabSelection::SceneItem(selected)
                | GrabSelection::Door(selected)
                | GrabSelection::Window(selected)
        ) if selected == target
    )
}

pub(super) fn held_item(
    actor: Entity,
    hand: EquipmentHand,
    items: &Query<(Entity, &ItemOf, Option<&EquipSlot>)>,
) -> Option<Entity> {
    items.iter().find_map(|(entity, owner, slot)| {
        (owner.0 == actor && slot.is_some_and(|slot| *slot == hand.slot())).then_some(entity)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEAF_SIZE_M: Vec3 = Vec3::new(1.0, 2.0, 0.05);

    fn leaf(rotation: Quat) -> GlobalTransform {
        GlobalTransform::from(Transform::from_translation(Vec3::Y).with_rotation(rotation))
    }

    #[test]
    fn closed_leaf_hides_an_item_on_its_far_side() {
        let eyes = Vec3::new(0.0, 1.5, -1.0);
        let item = Vec3::new(0.2, 0.1, 0.8);
        assert!(leaf_occludes(
            eyes,
            item,
            &leaf(Quat::IDENTITY),
            LEAF_SIZE_M
        ));
    }

    #[test]
    fn swung_open_leaf_or_a_near_side_item_stays_visible() {
        let eyes = Vec3::new(0.0, 1.5, -1.0);
        let open = Transform::from_xyz(-0.5, 1.0, 0.5)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2));
        assert!(!leaf_occludes(
            eyes,
            Vec3::new(0.2, 0.1, 0.8),
            &GlobalTransform::from(open),
            LEAF_SIZE_M
        ));
        assert!(!leaf_occludes(
            eyes,
            Vec3::new(0.2, 0.1, -0.4),
            &leaf(Quat::IDENTITY),
            LEAF_SIZE_M
        ));
    }
}
