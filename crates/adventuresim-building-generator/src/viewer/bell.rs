//! Bell detail preserves the canonical bronze shell and iron clapper batches.
use super::*;
use adventuresim_building_generator::{BuildingLodMaterial, ResolvedSolid};

pub(super) fn focus(plan: &BuildingPlan, origin: Vec2) -> Vec3 {
    let bell = plan
        .resolved_geometry
        .solids
        .iter()
        .find(|solid| solid.role == SolidRole::ChurchBell)
        .expect("church bell exists");
    bell.centre + Vec3::new(origin.x, bell.size.y * 0.3, origin.y)
}

pub(super) fn spawn(
    world: &mut World,
    plan: &BuildingPlan,
    solid: &ResolvedSolid,
    origin: Vec2,
    view: Option<ViewerView>,
) {
    let transform = Transform::from_translation(solid.centre + Vec3::new(origin.x, 0.0, origin.y));
    for batch in adventuresim_building_generator::compile_solid_detail(plan, solid).meshes {
        let bronze = batch.material == BuildingLodMaterial::Bronze;
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: if bronze {
                    Color::srgb(0.30, 0.21, 0.10)
                } else {
                    Color::srgb(0.09, 0.10, 0.11)
                },
                metallic: if bronze { 0.75 } else { 0.65 },
                perceptual_roughness: 0.65,
                ..default()
            });
        let faces = batch
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .map(|indices| {
                indices
                    .map(|i| batch.vertices[i as usize].position - solid.centre)
                    .to_vec()
            })
            .collect::<Vec<_>>();
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(flat_face_mesh(&faces));
        world.spawn((
            Name::new(format!(
                "resolved crown owner {} {:?}",
                solid.owner.0, solid.role
            )),
            ClosedSolid,
            GeometryOwner(solid.owner.0),
            ResolvedRenderItem {
                id: solid.id.0,
                fingerprint: stable_u64(&serde_json::to_vec(solid).unwrap()),
                local_half_size: solid.size * 0.5,
            },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            EditorBuildingEntity,
        ));
    }
    if view == Some(ViewerView::ChurchTowerBellUnderside) && solid.role == SolidRole::ChurchBell {
        // Photometry uses the actual bell envelope; the open suspension frame
        // would otherwise make sky determine the lighting quartiles.
        let centre = solid.centre + Vec3::new(origin.x, 0.0, origin.y);
        world.insert_resource(sample_polygon::SampleBounds {
            min: centre - solid.size * 0.5,
            max: centre + solid.size * 0.5,
        });
        // This diagnostic light exposes the cavity; production illumination is
        // assessed separately by tactical captures.
        world.spawn((
            PointLight {
                intensity: 220.0,
                range: 4.0,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_translation(focus(plan, origin) + Vec3::new(0.4, -0.7, -0.5)),
            EditorBuildingEntity,
        ));
    }
}
