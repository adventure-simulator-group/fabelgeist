//! Fixture state only; production observers own every mesh, material and LOD.
use adventuresim_building_generator::{compile_operable_doors, compile_operable_windows};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::super::buildings::building_transform;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum OpeningTarget {
    Door,
    Glazed,
    Shutter,
    Barred,
    FixedGlazed,
}

impl OpeningTarget {
    pub(super) fn frame(
        self,
        plan: &adventuresim_building_generator::BuildingPlan,
    ) -> (Vec3, Vec2, Vec2) {
        if matches!(self, Self::FixedGlazed) {
            use adventuresim_building_generator::{ClosureKind, ClosureState, WallSourceId};
            let opening = plan
                .opening_assemblies
                .iter()
                .find(|opening| {
                    opening.closure.state == ClosureState::Closed
                        && opening.closure.layers.contains(&ClosureKind::LeadedGlazing)
                        && plan.wall_assemblies.iter().any(|wall| {
                            wall.id == opening.host_wall
                                && wall.frame.outside_room.is_none()
                                && !matches!(wall.source, WallSourceId::RoofGable { .. })
                        })
                })
                .expect("review fixed civilian glazing");
            return (
                Vec3::new(
                    opening.frame.origin.x,
                    opening.sill_elevation_metres + opening.profile.clear_height_metres() * 0.5,
                    opening.frame.origin.y,
                ),
                opening.frame.tangent,
                opening.frame.outward,
            );
        }

        if matches!(self, Self::Door) {
            let door = compile_operable_doors(plan)
                .into_iter()
                .next()
                .expect("review door");
            return (door.closed_centre, door.tangent, door.outward);
        }
        let window = compile_operable_windows(plan)
            .into_iter()
            .find(|w| match self {
                Self::Glazed => {
                    w.leaf == adventuresim_building_generator::WindowLeafKind::LeadedGlass
                        && !w.barred
                }
                Self::Shutter => {
                    w.leaf == adventuresim_building_generator::WindowLeafKind::TimberShutter
                }
                Self::Barred => w.barred,
                Self::Door | Self::FixedGlazed => false,
            })
            .expect("review requires selected window treatment");
        (window.closed_centre, window.tangent, window.outward)
    }
}

#[derive(Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum OpeningPose {
    #[default]
    Closed,
    Open,
}

#[derive(Component)]
pub(in crate::tactical_scene_viewer) struct ReviewLeafPose {
    closed: Transform,
    hinge: Vec3,
    angle: f32,
}

impl ReviewLeafPose {
    pub(in crate::tactical_scene_viewer) fn boundary_gate(
        door: adventuresim_building_generator::DoorSpec,
        elevation: Vec3,
    ) -> Self {
        Self {
            closed: Transform::from_translation(door.closed_centre + elevation)
                .with_rotation(Quat::from_rotation_y(door.closed_yaw_radians)),
            hinge: door.hinge_centre + elevation,
            angle: door.open_angle_radians,
        }
    }

    fn transform(&self, pose: OpeningPose) -> Transform {
        let rotation = Quat::from_rotation_y(match pose {
            OpeningPose::Closed => 0.0,
            OpeningPose::Open => self.angle,
        });
        Transform {
            translation: self.hinge + rotation * (self.closed.translation - self.hinge),
            rotation: rotation * self.closed.rotation,
            ..self.closed
        }
    }
}

pub(super) fn select_pose(
    state: Option<Res<super::super::capture_state::SceneCaptureState>>,
    fixture: Option<Res<super::ReviewFixture>>,
    mut leaves: Query<(&ReviewLeafPose, &mut Transform)>,
) {
    let (Some(state), Some(fixture)) = (state, fixture) else {
        return;
    };
    let super::super::view_specs::CapturePose::CityExterior { camera } =
        state.views[state.view].pose
    else {
        return;
    };
    let pose = fixture.views[usize::from(camera)].openings;
    for (leaf, mut transform) in &mut leaves {
        *transform = leaf.transform(pose);
    }
}

pub(in crate::tactical_scene_viewer) fn spawn_openings(
    commands: &mut Commands,
    building: &GeneratedBuilding,
) {
    let transform = building_transform(building);
    let origin = building.collision.bounds.centre();
    let direction = |v: Vec2| transform.rotation * Vec3::new(v.x, 0.0, v.y);
    for door in compile_operable_doors(&building.plan) {
        let centre = transform.transform_point(door.closed_centre - origin);
        let closed = Transform::from_translation(centre)
            .with_rotation(transform.rotation * Quat::from_rotation_y(door.closed_yaw_radians));
        commands.spawn((
            Name::new("Fixture door"),
            SceneDoor {
                building_id: building.placement.id,
                opening_id: door.opening.0,
                size_metres: door.size_metres,
                doorway_centre_metres: centre,
                tangent: direction(door.tangent),
                outward: direction(door.outward),
            },
            closed,
            ReviewLeafPose {
                closed,
                hinge: transform.transform_point(door.hinge_centre - origin),
                angle: door.open_angle_radians,
            },
        ));
    }
    for window in compile_operable_windows(&building.plan) {
        let centre = transform.transform_point(window.closed_centre - origin);
        let closed = Transform::from_translation(centre)
            .with_rotation(transform.rotation * Quat::from_rotation_y(window.closed_yaw_radians));
        commands.spawn((
            Name::new("Fixture window"),
            SceneWindow {
                leaf: window.leaf,
                building_id: building.placement.id,
                opening_id: window.opening.0,
                size_metres: window.size_metres,
                opening_centre_metres: centre,
                tangent: direction(window.tangent),
                outward: direction(window.outward),
                barred: window.barred,
            },
            closed,
            ReviewLeafPose {
                closed,
                hinge: transform.transform_point(window.hinge_centre - origin),
                angle: window.open_angle_radians,
            },
        ));
    }
}
