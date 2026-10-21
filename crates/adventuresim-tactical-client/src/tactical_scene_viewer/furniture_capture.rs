//! Cameras and placement annotations only; production observers render all furniture.
use adventuresim_building_generator::furniture::FurnitureKind;
use adventuresim_tactical_core::{prelude::*, scene_coordinates::ScenePlanPoint};
use bevy::prelude::*;
use adventuresim_building_generator::spatial_geometry::Position;

use super::capture_state::BuildingReviewCamera;

pub(super) const PROFILE: &str = "furniture-review";
const EYE_HEIGHT_METRES: f32 = 1.65;
const CAMERA_OBJECT_CLEARANCE_METRES: f32 = 3.0;

#[derive(Debug, thiserror::Error)]
enum FurnitureCaptureError {
    #[error("furniture review requires generated market vendors")]
    MissingVendors,
}

pub(super) fn spawn(commands: &mut Commands, layout: &FurnitureLayout) -> Result {
    for instance in &layout.instances {
        commands.spawn((
            Name::new(format!("Furniture {}", instance.scene.id.0)),
            instance.scene,
            RigidBody::Static,
            CollisionLayers::new(TACTICAL_TERRAIN_LAYER, LayerMask::ALL),
            furniture_collider(instance.scene.key)?,
            Transform::from_translation(instance.position_metres.metres())
                .with_rotation(Quat::from_rotation_y(instance.orientation.yaw_radians())),
        ));
    }
    Ok(())
}

pub(super) fn setup(
    commands: &mut Commands,
    layout: &FurnitureLayout,
    terrain: &SceneTerrain,
    profile: &str,
    output: &std::path::Path,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Result<Option<Vec<BuildingReviewCamera>>> {
    write_layout(layout, output)?;
    if profile != PROFILE {
        return Ok(None);
    }
    let camera = |position, target| BuildingReviewCamera { position, target, plaster_raking_light: None };
    let mut cameras = FurnitureKind::OUTDOOR
        .into_iter()
        .map(|kind| {
            let mut matching = layout
                .instances
                .iter()
                .filter(|instance| instance.scene.key.kind() == kind);
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
            .ok_or_else(|| format!("furniture review lacks {kind:?}"))?;
            let bounds = instance.scene.key.recipe()?.bounds;
            let distance =
                bounds.plan_half_extents()?.metres().max_element() + CAMERA_OBJECT_CLEARANCE_METRES;
            let offset = instance
                .orientation
                .local_to_world(Vec2::new(distance * 0.6, -distance));
            let point = ScenePlanPoint::try_from(instance.position_metres.metres().xz() + offset)?;
            let height = terrain.required_height_at(point)?;
            Ok(camera(
                Position::from_metres(Vec3::new(
                    point.metres().x,
                    height.metres() + EYE_HEIGHT_METRES,
                    point.metres().y,
                ))?,
                Position::from_metres(instance.position_metres.metres() + Vec3::Y * bounds.centre()?.metres().y)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let market = layout
        .groups
        .iter()
        .filter(|group| group.kind == FurnitureGroupKind::Vendor)
        .map(|group| group.footprint.centre().metres())
        .collect::<Vec<_>>();
    if market.is_empty() {
        return Err(FurnitureCaptureError::MissingVendors.into());
    }
    let centre = market.iter().copied().sum::<Vec2>() / market.len() as f32;
    let point = ScenePlanPoint::try_from(centre)?;
    let target = Position::from_metres(Vec3::new(
        centre.x,
        terrain.required_height_at(point)?.metres(),
        centre.y,
    ))?;
    cameras.push(camera(Position::from_metres(target.metres() + Vec3::new(48.0, 40.0, -58.0))?, target));
    cameras.push(camera(
        Position::from_metres(Vec3::new(
            -10.0,
            terrain
                .required_height_at(ScenePlanPoint::try_from(Vec2::new(-10.0, -30.0))?)?
                .metres()
                + EYE_HEIGHT_METRES,
            -30.0,
        ))?,
        Position::from_metres(Vec3::new(
            15.0,
            terrain
                .required_height_at(ScenePlanPoint::try_from(Vec2::new(15.0, -30.0))?)?
                .metres(),
            -30.0,
        ))?,
    ));
    let junction = Position::from_metres(Vec3::new(
        -30.0,
        terrain
            .required_height_at(ScenePlanPoint::try_from(Vec2::splat(-30.0))?)?
            .metres(),
        -30.0,
    ))?;
    cameras.push(camera(Position::from_metres(junction.metres() + Vec3::new(-21.0, 24.0, -23.0))?, junction));
    cameras.push(camera(Position::from_metres(junction.metres() + Vec3::new(-9.0, 7.0, -10.0))?, junction));
    commands.insert_resource(super::furniture_readiness::ExpectedFurniture {
        instances: layout.instances.len() + layout.distant_instances.len(),
        batches: layout
            .instances
            .iter()
            .chain(&layout.distant_instances)
            .map(|instance| instance.scene.key.recipe().map(|r| r.meshes.len()))
            .collect::<std::result::Result<Vec<_>, _>>()?
            .into_iter()
            .sum(),
    });
    super::furniture_overlay::annotate(commands, layout, terrain, meshes, materials);
    Ok(Some(cameras))
}

fn write_layout(layout: &FurnitureLayout, output: &std::path::Path) -> Result {
    let instances = layout.instances.iter().map(|instance| {
        let recipe = instance.scene.key.recipe()?;
        Ok::<_, adventuresim_building_generator::furniture::FurnitureRecipeError>(serde_json::json!({
            "scene": instance.scene, "position_metres": instance.position_metres, "orientation": instance.orientation,
            "colliders": recipe.colliders.len(), "triangles": recipe.meshes.iter().map(|mesh| mesh.indices.len() / 3).sum::<usize>(),
        }))
    }).collect::<std::result::Result<Vec<_>, _>>()?;
    let evidence = serde_json::json!({
        "instances": instances,
        "distant_instances": layout.distant_instances,
        "groups": layout.groups,
        "reserved_routes": layout.reserved_routes,
        "interiors": layout.interiors,
    });
    std::fs::write(
        output.join("furniture-layout.json"),
        serde_json::to_vec_pretty(&evidence)?,
    )?;
    Ok(())
}
