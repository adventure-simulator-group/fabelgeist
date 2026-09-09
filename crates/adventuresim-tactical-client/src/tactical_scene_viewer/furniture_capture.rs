//! Cameras and placement annotations only; production observers render all furniture.
use adventuresim_building_generator::furniture::FurnitureKind;
use adventuresim_tactical_core::prelude::*;
use bevy::prelude::*;

use super::capture_state::BuildingReviewCamera;

pub(super) const PROFILE: &str = "furniture-review";
const EYE_HEIGHT_METRES: f32 = 1.65;
const CAMERA_OBJECT_CLEARANCE_METRES: f32 = 3.0;

pub(super) fn spawn(commands: &mut Commands, layout: &FurnitureLayout) {
    for instance in &layout.instances {
        commands.spawn((
            Name::new(format!("Outdoor furniture {}", instance.scene.id.0)),
            instance.scene,
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            furniture_collider(instance.scene.key),
            Transform::from_translation(instance.position_metres)
                .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
        ));
    }
}

pub(super) fn setup(
    commands: &mut Commands,
    layout: &FurnitureLayout,
    terrain: &SceneTerrain,
    profile: &str,
    output: &std::path::Path,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Option<Vec<BuildingReviewCamera>> {
    if profile != PROFILE {
        return None;
    }
    let camera = |position, target| BuildingReviewCamera {
        position,
        target,
        plaster_raking_light: None,
    };
    let mut cameras = FurnitureKind::ALL
        .into_iter()
        .map(|kind| {
            let mut matching = layout
                .instances
                .iter()
                .filter(|instance| instance.scene.key.kind == kind);
            // Review the stable's last accepted group rather than the inn's
            // first group so the smaller frontage frames these low objects.
            let instance = if matches!(
                kind,
                FurnitureKind::TableBenchSet | FurnitureKind::HitchingTrough
            ) {
                matching.next_back()
            } else {
                matching.next()
            }
            .expect("furniture review must exercise every supported family");
            let bounds = instance.scene.key.recipe().bounds;
            let distance =
                bounds.plan_half_extents().max_element() + CAMERA_OBJECT_CLEARANCE_METRES;
            let offset = instance
                .orientation
                .local_to_world(Vec2::new(distance * 0.6, -distance));
            let point = instance.position_metres.xz() + offset;
            let height = terrain
                .height_at(point)
                .expect("furniture camera stays on playable terrain");
            camera(
                Vec3::new(point.x, height + EYE_HEIGHT_METRES, point.y),
                instance.position_metres + Vec3::Y * bounds.centre().y,
            )
        })
        .collect::<Vec<_>>();
    let market = layout
        .groups
        .iter()
        .filter(|group| group.kind == FurnitureGroupKind::Vendor)
        .map(|group| group.footprint.centre_metres)
        .collect::<Vec<_>>();
    assert!(
        !market.is_empty(),
        "furniture review requires generated market vendors"
    );
    let centre = market.iter().copied().sum::<Vec2>() / market.len() as f32;
    let target = Vec3::new(centre.x, terrain.height_at(centre).unwrap(), centre.y);
    cameras.push(camera(target + Vec3::new(48.0, 40.0, -58.0), target));
    cameras.push(camera(
        target + Vec3::new(0.0, EYE_HEIGHT_METRES, -8.0),
        target + Vec3::Z * 5.0,
    ));
    commands.insert_resource(super::furniture_readiness::ExpectedFurniture {
        instances: layout.instances.len(),
        batches: layout
            .instances
            .iter()
            .map(|instance| instance.scene.key.recipe().meshes.len())
            .sum(),
    });
    super::furniture_overlay::annotate(commands, layout, terrain, meshes, materials);
    let evidence = serde_json::json!({
        "instances": layout.instances.iter().map(|instance| serde_json::json!({
            "scene": instance.scene, "position_metres": instance.position_metres,
            "orientation": instance.orientation,
            "colliders": instance.scene.key.recipe().colliders.len(),
            "triangles": instance.scene.key.recipe().meshes.iter().map(|mesh| mesh.indices.len() / 3).sum::<usize>(),
        })).collect::<Vec<_>>(),
        "groups": layout.groups,
        "reserved_routes": layout.reserved_routes,
    });
    std::fs::write(
        output.join("furniture-layout.json"),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .expect("write authoritative furniture placement evidence");
    Some(cameras)
}
