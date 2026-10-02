//! Verify actual owned specimen transforms and GPU residency before capture.
use crate::presentation::{ManagedGardenPlant, TacticalTreeLeafCardMaterial};
use adventuresim_tactical_core::prelude::SceneGarden;
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub(super) struct GardenObservation<'w, 's> {
    gardens: Query<'w, 's, (&'static SceneGarden, &'static GlobalTransform)>,
    plants: Query<
        'w,
        's,
        (
            &'static ManagedGardenPlant,
            &'static GlobalTransform,
            &'static Children,
        ),
    >,
    branches: Query<
        'w,
        's,
        (
            &'static Mesh3d,
            &'static crate::presentation::interior_lighting::InteriorMaterialSource,
        ),
    >,
    leaves: Query<
        'w,
        's,
        (
            &'static Mesh3d,
            &'static MeshMaterial3d<TacticalTreeLeafCardMaterial>,
        ),
    >,
    leaf_assets: Res<'w, Assets<TacticalTreeLeafCardMaterial>>,
}
impl GardenObservation<'_, '_> {
    pub(super) fn check(
        &self,
        gpu: &crate::tactical_scene_viewer::gpu_readiness::GpuReadiness,
    ) -> Result<(), &'static str> {
        let expected = self
            .gardens
            .iter()
            .flat_map(|(scene, transform)| {
                scene
                    .garden
                    .plants
                    .iter()
                    .map(move |plant| (plant, transform))
            })
            .collect::<Vec<_>>();
        if self.plants.iter().count() != expected.len() {
            return Err("garden plant count");
        }
        for (plant, base) in expected {
            let Some((_, transform, children)) = self
                .plants
                .iter()
                .find(|(marker, _, _)| marker.0 == plant.id)
            else {
                return Err("garden plant identity");
            };
            let actual = transform.compute_transform();
            let position =
                base.transform_point(Vec3::new(plant.centre_metres.x, 0.0, plant.centre_metres.y));
            if actual.translation.distance(position) > 0.001
                || actual.scale.distance(Vec3::splat(plant.scale.value())) > 0.001
                || actual
                    .rotation
                    .angle_between(Quat::from_rotation_y(plant.orientation.yaw_radians()))
                    > 0.001
            {
                return Err("garden plant transform");
            }
            let mut branch_count = 0;
            let mut leaf_count = 0;
            for child in children.iter() {
                if let Ok((mesh, _)) = self.branches.get(child) {
                    if !gpu.contains(&mesh.0, std::iter::empty()) {
                        return Err("garden branch GPU mesh");
                    }
                    branch_count += 1;
                }
                if let Ok((mesh, handle)) = self.leaves.get(child) {
                    let Some(material) = self.leaf_assets.get(&handle.0) else {
                        return Err("garden leaf material");
                    };
                    if !gpu.contains(
                        &mesh.0,
                        [
                            &material.opacity,
                            &material.front_albedo,
                            &material.back_albedo,
                            &material.front_normal,
                            &material.back_normal,
                            &material.arm,
                        ]
                        .into_iter()
                        .map(Handle::id),
                    ) {
                        return Err("garden leaf GPU mesh or textures");
                    }
                    leaf_count += 1;
                }
            }
            if branch_count != 1 || leaf_count != 1 {
                return Err("garden branch/leaf child count");
            }
        }
        Ok(())
    }
}
