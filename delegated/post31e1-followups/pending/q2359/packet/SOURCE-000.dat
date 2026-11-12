//! Read back residency from the real render world, including GPU-only textures.
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems, mesh::RenderMesh, render_asset::RenderAssets,
        texture::GpuImage,
    },
};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Resource, Clone, Default)]
pub(super) struct GpuReadiness(Arc<Mutex<ResidentAssets>>);

#[derive(Default)]
struct ResidentAssets {
    meshes: HashSet<AssetId<Mesh>>,
    images: HashSet<AssetId<Image>>,
}

impl GpuReadiness {
    pub(super) fn install(app: &mut App) {
        let readiness = Self::default();
        app.insert_resource(readiness.clone());
        app.sub_app_mut(RenderApp)
            .insert_resource(readiness)
            .add_systems(Render, observe.in_set(RenderSystems::Cleanup));
    }

    pub(super) fn contains(
        &self,
        mesh: &Handle<Mesh>,
        mut textures: impl Iterator<Item = AssetId<Image>>,
    ) -> bool {
        let resident = self.0.lock().expect("GPU readiness lock");
        resident.meshes.contains(&mesh.id()) && textures.all(|id| resident.images.contains(&id))
    }
}

fn observe(
    readiness: Res<GpuReadiness>,
    meshes: Res<RenderAssets<RenderMesh>>,
    images: Res<RenderAssets<GpuImage>>,
) {
    let mut resident = readiness.0.lock().expect("GPU readiness lock");
    resident.meshes = meshes.iter().map(|(id, _)| id).collect();
    resident.images = images.iter().map(|(id, _)| id).collect();
}
