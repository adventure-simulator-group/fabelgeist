//! Fixture state only; production observers own every mesh, material and LOD.
use adventuresim_building_generator::{compile_operable_doors, compile_operable_windows};
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum OpeningTarget {
    Door,
    Glazed,
    Shutter,
    Barred,
    FixedGlazed,
}

pub(super) struct OpeningFrame {
    pub centre: adventuresim_building_generator::spatial_geometry::Position<
        adventuresim_building_generator::spatial_geometry::Architectural,
    >,
    pub tangent: adventuresim_building_generator::spatial_geometry::PlanDirection<
        adventuresim_building_generator::spatial_geometry::Architectural,
    >,
    pub outward: adventuresim_building_generator::spatial_geometry::PlanDirection<
        adventuresim_building_generator::spatial_geometry::Architectural,
    >,
}
impl OpeningFrame {
    fn from_metres(centre: Vec3, tangent: Vec2, outward: Vec2) -> Result<Self> {
        use adventuresim_building_generator::spatial_geometry::{PlanDirection, Position};
        Ok(Self {
            centre: Position::from_metres(centre)?,
            tangent: PlanDirection::from_normalized(tangent)?,
            outward: PlanDirection::from_normalized(outward)?,
        })
    }
}

impl OpeningTarget {
    pub(super) fn frame(
        self,
        plan: &adventuresim_building_generator::BuildingPlan,
    ) -> Result<OpeningFrame> {
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
                .ok_or("review fixed civilian glazing is absent")?;
            return OpeningFrame::from_metres(
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
            let door = compile_operable_doors(plan)?
                .into_iter()
                .next()
                .ok_or("review door is absent")?;
            return Ok(OpeningFrame {
                centre: door.closed_centre,
                tangent: door.tangent,
                outward: door.outward,
            });
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
            .ok_or("review window treatment is absent")?;
        OpeningFrame::from_metres(window.closed_centre, window.tangent, window.outward)
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
    pub(in crate::tactical_scene_viewer) fn from_scene(
        pose: adventuresim_tactical_core::scene_coordinates::SceneDoorPose,
    ) -> Self {
        Self {
            closed: Transform::from_translation(pose.leaf.closed_centre.metres())
                .with_rotation(pose.native_rotation()),
            hinge: pose.leaf.hinge_centre.metres(),
            angle: pose.leaf.open_angle_radians.radians(),
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
) -> Result {
    let transform = building.transform()?;
    let datum = building.geometry_datum()?;
    let origin = building.collision.bounds.centre()?.metres();
    let direction = |v: Vec2| transform.rotation * Vec3::new(v.x, 0.0, v.y);
    for leaf in compile_operable_doors(&building.plan)? {
        let pose = datum.door(leaf)?;
        let door = pose.leaf;
        let centre = door.closed_centre.metres();
        let closed = Transform::from_translation(centre).with_rotation(pose.native_rotation());
        commands.spawn((
            Name::new("Fixture door"),
            SceneDoor {
                building_id: building.placement.id,
                opening_id: door.opening.0,
                size_metres: door.size_metres,
                doorway_centre_metres: door.closed_centre,
                tangent: door.tangent.spatial(),
                outward: door.outward.spatial(),
            },
            closed,
            ReviewLeafPose {
                closed,
                hinge: door.hinge_centre.metres(),
                angle: door.open_angle_radians.radians(),
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
    Ok(())
}
