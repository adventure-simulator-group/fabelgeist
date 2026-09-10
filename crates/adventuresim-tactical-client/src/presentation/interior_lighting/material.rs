//! Standard PBR assets remain the source of truth; this renderer adds the shared daylight field.
use super::InteriorLightingGpu;
use bevy::{
    asset::AssetId,
    material::OpaqueRendererMethod,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::{render_resource::AsBindGroup, storage::ShaderBuffer},
    shader::ShaderRef,
};
use std::collections::HashMap;

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub(crate) struct InteriorExtension {
    #[storage(200, read_only)]
    pub(crate) field: Handle<ShaderBuffer>,
}

impl MaterialExtension for InteriorExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/tactical_interior_material.wgsl".into()
    }
}

pub(crate) type InteriorMaterial = ExtendedMaterial<StandardMaterial, InteriorExtension>;

/// Retains the authored material for inspection, edits, and surface effects.
#[derive(Component)]
pub(crate) struct InteriorMaterialSource(pub Handle<StandardMaterial>);

#[derive(Resource, Default)]
pub(super) struct InteriorMaterials(HashMap<AssetId<StandardMaterial>, Handle<InteriorMaterial>>);

pub(super) fn prepare_materials(
    mut commands: Commands,
    gpu: Res<InteriorLightingGpu>,
    sources: Res<Assets<StandardMaterial>>,
    mut materials: ResMut<Assets<InteriorMaterial>>,
    mut cache: ResMut<InteriorMaterials>,
    incoming: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut events: MessageReader<AssetEvent<StandardMaterial>>,
) {
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } => {
                if let Some(mut material) =
                    cache.0.get(id).and_then(|handle| materials.get_mut(handle))
                    && let Some(source) = sources.get(*id)
                {
                    material.base = source.clone();
                    material.base.opaque_render_method = OpaqueRendererMethod::Forward;
                }
            }
            AssetEvent::Unused { id } | AssetEvent::Removed { id } => {
                cache.0.remove(id);
            }
            _ => {}
        }
    }
    for (entity, handle) in &incoming {
        let Some(source) = sources.get(&handle.0) else {
            continue;
        };
        let material = cache.0.entry(handle.0.id()).or_insert_with(|| {
            let mut base = source.clone();
            base.opaque_render_method = OpaqueRendererMethod::Forward;
            materials.add(InteriorMaterial {
                base,
                extension: InteriorExtension {
                    field: gpu.buffer.clone(),
                },
            })
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert((
                InteriorMaterialSource(handle.0.clone()),
                MeshMaterial3d(material.clone()),
            ));
    }
}
