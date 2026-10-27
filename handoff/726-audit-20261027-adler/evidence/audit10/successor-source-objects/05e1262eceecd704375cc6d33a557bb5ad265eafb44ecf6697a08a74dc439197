//! One full-size frontage in the retained city, viewed through one perspective camera.
use super::preparation::{PreparationError, PreparationResult};
use super::protocol::PlaceId;
use adventuresim_building_generator::OpeningUse;
use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::prelude::*;
use adventuresim_tactical_core::scene_coordinates::{PlanDisplacement, Scene, ScenePlanPoint};
use bevy::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

const BUILDING_GAP_METRES: f32 = 4.0;
const MINIMUM_BAY_METRES: f32 = 18.0;
const CITY_CLEARANCE_METRES: f32 = 128.0;
const SKY_MARGIN_METRES: f32 = 8.0;
const GROUND_DEPTH_METRES: f32 = 512.0;
const EYE_HEIGHT_METRES: f32 = 1.7;
const FRAME_CENTRE_HEIGHT_FRACTION: f32 = 0.45;

#[derive(Clone, Serialize)]
pub(super) struct StreetBay {
    pub id: String,
    pub width: f32,
}

#[derive(Clone, Serialize)]
pub(super) struct Street {
    pub bays: Vec<StreetBay>,
    pub width: f32,
    pub height: f32,
    #[serde(skip)]
    pub front: Position<Scene>,
}

#[derive(Component)]
pub(super) struct StreetGround(Street);

struct BuildingRelocation {
    before: Transform,
    after: Transform,
}

impl Street {
    pub(super) fn arrange(
        input: &TacticalSceneInput,
        places: &[super::protocol::Place],
        selected: &HashMap<PlaceId, adventuresim_tactical_core::scene_input::SceneBuildingId>,
        generated: &mut crate::presentation::PreparedStrategicScene,
        backdrop: &crate::presentation::StrategicBackdrop,
    ) -> PreparationResult<Self> {
        let front_z = input
            .distant_buildings
            .iter()
            .map(|b| b.centre_metres.metres().y)
            .chain(
                generated
                    .buildings
                    .iter()
                    .map(|b| b.placement.centre_metres.metres().y),
            )
            .fold(0.0, f32::max)
            + CITY_CLEARANCE_METRES;
        let front_point = ScenePlanPoint::try_from(Vec2::new(0.0, front_z))?;
        let elevation = backdrop.required_height_at(front_point)?;
        let mut street = Self {
            bays: Vec::new(),
            width: 0.0,
            height: MINIMUM_BAY_METRES,
            front: Position::from_metres(Vec3::new(0.0, elevation.metres(), front_z))?,
        };
        let mut moves = HashMap::new();
        for place in places {
            let building = selected.get(&place.id).and_then(|id| {
                generated
                    .buildings
                    .iter_mut()
                    .find(|b| b.placement.id == *id)
            });
            let mut width = MINIMUM_BAY_METRES;
            if let Some(building) = building {
                let before = building.transform()?;
                let outward = building
                    .plan
                    .opening_assemblies
                    .iter()
                    .find(|opening| {
                        matches!(opening.use_kind, OpeningUse::Door | OpeningUse::Gate)
                            && opening.frame.outside_room.is_none()
                            && opening.sill_elevation_metres.abs() < 0.1
                    })
                    .map_or(Vec2::Y, |opening| opening.frame.outward);
                let yaw = -outward.x.atan2(outward.y);
                let orientation = BuildingOrientation::from_radians(yaw)
                    .ok_or(PreparationError::DoorwayOrientation)?;
                let extent = building.collision.bounds.extent()?.metres();
                let rotation = Quat::from_rotation_y(yaw);
                let size =
                    (rotation * Vec3::X * extent.x).abs() + (rotation * Vec3::Z * extent.z).abs();
                width = width.max(size.x + BUILDING_GAP_METRES);
                street.height = street.height.max(extent.y + SKY_MARGIN_METRES);
                building.placement.orientation = orientation;
                building.placement.centre_metres =
                    adventuresim_tactical_core::scene_coordinates::ScenePlanPoint::try_from(
                        Vec2::new(street.width + width * 0.5, front_z - size.z * 0.5),
                    )?;
                building.placement.base_elevation_metres = elevation;
                moves.insert(
                    building.placement.id,
                    BuildingRelocation {
                        before,
                        after: building.transform()?,
                    },
                );
            }
            street.bays.push(StreetBay {
                id: place.id.0.clone(),
                width,
            });
            street.width += width;
        }
        let centering = PlanDisplacement::try_from(Vec2::new(-street.width * 0.5, 0.0))?;
        for building in &mut generated.buildings {
            if let Some(relocation) = moves.get_mut(&building.placement.id) {
                let centre = building.placement.centre_metres;
                building.placement.centre_metres = centre.translated(centering)?;
                relocation.after = building.transform()?;
            }
        }
        relocate_interior_furniture(&mut generated.furniture, &moves)?;
        Ok(street)
    }

    pub(super) fn camera(&self) -> Transform {
        let target = self.front.metres() + Vec3::Y * self.height * FRAME_CENTRE_HEIGHT_FRACTION;
        Transform::from_translation(
            self.front.metres() + Vec3::new(0.0, EYE_HEIGHT_METRES, self.distance()),
        )
        .looking_at(target, Vec3::Y)
    }

    pub(super) fn vertical_fov(&self) -> f32 {
        2.0 * (self.height * 0.5 / self.distance()).atan()
    }

    pub(super) fn spawn_ground(&self, commands: &mut Commands, root: Entity) {
        commands.spawn((StreetGround(self.clone()), ChildOf(root)));
    }
    fn distance(&self) -> f32 {
        // Keep a normal horizontal lens across the wide strip, rather than an
        // extreme wide-angle lens obtained by retaining the old vertical FOV.
        let fov = crate::presentation::TacticalCameraSetup::default()
            .vertical_fov_degrees
            .to_radians();
        self.width.max(MINIMUM_BAY_METRES) * 0.5 / (fov * 0.5).tan()
    }
}

pub(super) fn build_ground(
    mut commands: Commands,
    grounds: Query<(Entity, &StreetGround), Added<StreetGround>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, ground) in &grounds {
        let street = &ground.0;
        commands.entity(entity).insert((
            Mesh3d(
                meshes.add(
                    Plane3d::default()
                        .mesh()
                        .size(street.width + GROUND_DEPTH_METRES, GROUND_DEPTH_METRES),
                ),
            ),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.25, 0.23, 0.19),
                perceptual_roughness: 1.0,
                ..default()
            })),
            Transform::from_translation(street.front.metres() + Vec3::Y * 0.01),
        ));
    }
}

fn relocate_interior_furniture(
    furniture: &mut FurnitureLayout,
    moves: &HashMap<adventuresim_tactical_core::scene_input::SceneBuildingId, BuildingRelocation>,
) -> PreparationResult<()> {
    for furniture in &mut furniture.instances {
        if let FurnitureLocation::Interior { building_id, .. } = furniture.scene.location
            && let Some(relocation) = moves.get(&building_id)
        {
            let local = relocation
                .before
                .compute_affine()
                .inverse()
                .transform_point3(furniture.position_metres.metres());
            furniture.position_metres =
                adventuresim_building_generator::spatial_geometry::Position::from_metres(
                    relocation.after.transform_point(local),
                )?;
            let delta = (relocation.after.rotation * relocation.before.rotation.inverse())
                .to_euler(EulerRot::YXZ)
                .0;
            furniture.orientation =
                BuildingOrientation::from_radians(furniture.orientation.yaw_radians() + delta)
                    .ok_or(PreparationError::FurnitureOrientation)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic_scene::protocol::{Place, PlaceKind};

    #[test]
    fn frontage_keeps_full_size_buildings_and_their_furniture_together() {
        let input: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../assets/tactical-scenes/massive-city.json"
        ))
        .unwrap();
        let flat: TacticalSceneInput = serde_json::from_str(include_str!(
            "../../../../assets/tactical-scenes/flat-dry-grassland.json"
        ))
        .unwrap();
        let mut generated = crate::presentation::PreparedStrategicScene::generate(&flat).unwrap();
        let backdrop =
            crate::presentation::StrategicBackdrop::new(&flat, flat.vista.lods.len()).unwrap();
        generated.buildings = input
            .buildings
            .iter()
            .take(2)
            .map(|placement| {
                let plan = adventuresim_building_generator::generate(&placement.program).unwrap();
                let collision =
                    adventuresim_building_generator::compile_building_collision(&plan).unwrap();
                GeneratedBuilding {
                    placement: placement.clone(),
                    plan,
                    collision,
                }
            })
            .collect();
        generated
            .furniture
            .furnish_interiors(&generated.buildings)
            .unwrap();
        let places: Vec<_> = generated
            .buildings
            .iter()
            .map(|b| Place {
                id: PlaceId(b.placement.id.to_string()),
                kind: PlaceKind::Inn,
            })
            .collect();
        let selected = generated
            .buildings
            .iter()
            .map(|b| (PlaceId(b.placement.id.to_string()), b.placement.id))
            .collect();
        let originals: HashMap<_, _> = generated
            .buildings
            .iter()
            .map(|b| (b.placement.id, b.transform().unwrap()))
            .collect();
        let furniture = generated.furniture.instances.clone();
        let street =
            Street::arrange(&input, &places, &selected, &mut generated, &backdrop).unwrap();
        let mut bay_left = -street.width * 0.5;
        let mut previous_right = bay_left;
        for (building, bay) in generated.buildings.iter().zip(&street.bays) {
            let pose = building.transform().unwrap();
            let bay_centre = bay_left + bay.width * 0.5;
            assert!(
                (pose.translation.x - bay_centre).abs() < 0.001,
                "building {} is at {}, expected centered bay at {}",
                building.placement.id,
                pose.translation.x,
                bay_centre,
            );
            bay_left += bay.width;
            assert_eq!(pose.scale, Vec3::ONE);
            let extent =
                building.collision.bounds.max().metres() - building.collision.bounds.min().metres();
            let size = (pose.rotation * Vec3::X * extent.x).abs()
                + (pose.rotation * Vec3::Z * extent.z).abs();
            assert!(pose.translation.x - size.x * 0.5 >= previous_right);
            previous_right = pose.translation.x + size.x * 0.5;
            assert!((pose.translation.z + size.z * 0.5 - street.front.metres().z).abs() < 0.01);
            for (before, after) in furniture.iter().zip(&generated.furniture.instances) {
                if matches!(before.scene.location, FurnitureLocation::Interior { building_id, .. } if building_id == building.placement.id)
                {
                    let old_local = originals[&building.placement.id]
                        .compute_affine()
                        .inverse()
                        .transform_point3(before.position_metres.metres());
                    let new_local = pose
                        .compute_affine()
                        .inverse()
                        .transform_point3(after.position_metres.metres());
                    assert!(old_local.distance(new_local) < 0.001);
                }
            }
        }
        assert!(previous_right <= street.width * 0.5);
        assert_eq!(street.camera().translation.x, 0.0);
        assert!(street.camera().translation.z > street.front.metres().z);
        assert!(
            (street.camera().translation.y - street.front.metres().y - EYE_HEIGHT_METRES).abs()
                < 0.001
        );
    }
}
