use crate::presentation::PlantLodInstance;
use adventuresim_plant_generator::{PlantLod, PlantSpecies};
use bevy::camera::visibility::VisibleEntities;
use bevy::prelude::*;
use serde::Serialize;

pub(super) type Instances<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static PlantLodInstance,
        &'static GlobalTransform,
        &'static Mesh3d,
    ),
    Without<crate::presentation::TacticalGameplayCamera>,
>;

#[derive(Clone, Serialize)]
pub(super) struct Observation {
    level: PlantLod,
    species: PlantSpecies,
    distance_metres: f32,
    in_visibility_range: bool,
    visible: bool,
    mesh_triangles: usize,
}

pub(super) fn observe(
    instances: &Instances,
    meshes: &Assets<Mesh>,
    visible_entities: &VisibleEntities,
    root: Option<Vec3>,
    eye: Vec3,
) -> Vec<Observation> {
    let Some(root) = root else { return Vec::new() };
    let mut rows = instances
        .iter()
        .filter(|(_, _, transform, _)| transform.translation().distance_squared(root) < 1e-8)
        .map(|(entity, instance, transform, mesh)| {
            let distance = eye.distance(transform.translation());
            Observation {
                level: instance.level,
                species: instance.species,
                distance_metres: distance,
                in_visibility_range: instance.range().is_visible_at_all(distance),
                visible: visible_entities
                    .get(std::any::TypeId::of::<Mesh3d>())
                    .contains(&entity),
                mesh_triangles: meshes
                    .get(&mesh.0)
                    .and_then(|m| m.indices())
                    .map_or(0, |i| i.len() / 3),
            }
        })
        .collect::<Vec<_>>();
    rows.sort_by_key(|r| r.level.index());
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn other_camera_or_shadow_visibility_does_not_contaminate_capture() {
        let mut world = World::new();
        let root = Vec3::new(2.0, 100.0, 3.0);
        let entities = [PlantLod::High, PlantLod::Low].map(|level| {
            world
                .spawn((
                    PlantLodInstance {
                        level,
                        species: PlantSpecies::ALL[0],
                    },
                    GlobalTransform::from_translation(root),
                    Mesh3d::default(),
                    ViewVisibility::VISIBLE,
                ))
                .id()
        });
        let cameras = entities.map(|entity| {
            let mut visible = VisibleEntities::default();
            visible
                .get_mut(std::any::TypeId::of::<Mesh3d>())
                .push(entity);
            visible
        });
        let mut state = bevy::ecs::system::SystemState::<Instances>::new(&mut world);
        let instances = state.get(&world).unwrap();
        let meshes = Assets::<Mesh>::default();
        let near = observe(&instances, &meshes, &cameras[0], Some(root), root + Vec3::Z);
        let far = observe(
            &instances,
            &meshes,
            &cameras[1],
            Some(root),
            root + Vec3::Z * 10.0,
        );
        assert!(near[0].visible && !near[1].visible);
        assert!(!far[0].visible && far[1].visible);
        assert!(near[0].in_visibility_range && !near[1].in_visibility_range);
        assert!(!far[0].in_visibility_range && far[1].in_visibility_range);
    }
}
