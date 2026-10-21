//! Actual view, mesh and material keys through ordinary Bevy specialization.

use super::*;
use bevy::{
    material::{
        MaterialProperties,
        key::{ErasedMaterialPipelineKey, ErasedMeshPipelineKey},
    },
    mesh::MeshVertexBufferLayoutRef,
    pbr::{PreparedMaterial, ViewKeyCache, alpha_mode_pipeline_key},
    render::{
        camera::ExtractedCamera,
        erased_render_asset::ErasedRenderAssets,
        mesh::RenderMesh,
        render_asset::RenderAssets,
        render_resource::{CachedPipelineState, PipelineCache},
        view::ExtractedView,
    },
    shader::ShaderCacheError,
};

struct Demand {
    target: DetailTarget,
    view_key: MeshPipelineKey,
    layout: MeshVertexBufferLayoutRef,
    key: ErasedMaterialPipelineKey,
    properties: Arc<MaterialProperties>,
}

pub(super) fn prepare(world: &mut World) -> bevy::prelude::Result {
    let readiness = world.resource::<TerrainPipelineReadiness>().clone();
    let Some(request) = readiness.request.clone() else {
        return Ok(());
    };
    let was_ready = readiness.is_ready_for(request.scene, request.camera);
    // Keep invalidation and replacement atomic to main-world observers.
    let mut state = request
        .state
        .lock()
        .map_err(|_| "terrain preparation lock")?;
    *state = Preparation::Pending;
    let Some(demands) = collect(world, &request)? else {
        return Ok(());
    };
    let mut prepared = Vec::with_capacity(demands.len());
    let mut ready = true;
    for demand in demands {
        let specialize = demand
            .properties
            .base_specialize
            .ok_or("required terrain material has no color specializer")?;
        let pipeline = specialize(world, demand.key, &demand.layout, &demand.properties)?;
        let cache = world.resource::<PipelineCache>();
        match cache.get_render_pipeline_state(pipeline) {
            // These states are retried by Bevy after shader assets/imports arrive.
            CachedPipelineState::Err(
                ShaderCacheError::ShaderNotLoaded(_)
                | ShaderCacheError::ShaderImportNotYetAvailable,
            ) => {}
            CachedPipelineState::Err(error) => {
                return Err(format!("required terrain color pipeline failed: {error:?}").into());
            }
            _ => {}
        }
        ready &= cache.get_render_pipeline(pipeline).is_some();
        prepared.push(PreparedDetail {
            target: demand.target,
            view_key: demand.view_key,
            pipeline,
            depth_bias: demand.properties.depth_bias,
        });
    }
    if ready {
        *state = Preparation::Ready(prepared)
    }
    drop(state);
    if ready && !was_ready {
        debug!(preparation = %readiness.report(), "Required terrain color pipelines prepared");
    }
    Ok(())
}

fn collect(
    world: &World,
    request: &RequiredPipelines,
) -> bevy::prelude::Result<Option<Vec<Demand>>> {
    let render_camera = request.render_camera.id();
    if world.get::<ExtractedCamera>(render_camera).is_none() {
        return Ok(None);
    }
    let Some(view) = world.get::<ExtractedView>(render_camera) else {
        return Ok(None);
    };
    if view.retained_view_entity.main_entity != request.camera.into() {
        return Err("required terrain camera render mapping changed".into());
    }
    let Some(view_key) = world
        .resource::<ViewKeyCache>()
        .get(&view.retained_view_entity)
        .copied()
    else {
        return Ok(None);
    };
    let meshes = world.resource::<RenderAssets<RenderMesh>>();
    let materials = world.resource::<ErasedRenderAssets<PreparedMaterial>>();
    let mut demands = Vec::with_capacity(request.targets.len());
    for target in &request.targets {
        let Some(mesh) = meshes.get(target.mesh) else {
            return Ok(None);
        };
        let Some(material) = materials.get(target.material.untyped()) else {
            return Ok(None);
        };
        let properties = material.properties.clone();
        if properties.alpha_mode != AlphaMode::Opaque {
            return Err("required terrain detail must remain opaque".into());
        }
        let mut mesh_key = view_key
            | MeshPipelineKey::from_bits_retain(mesh.key_bits.bits())
            | properties
                .mesh_pipeline_key_bits
                .downcast::<MeshPipelineKey>();
        mesh_key.insert(alpha_mode_pipeline_key(
            properties.alpha_mode,
            &Msaa::from_samples(view_key.msaa_samples()),
        ));
        demands.push(Demand {
            target: target.clone(),
            view_key,
            layout: mesh.layout.clone(),
            key: ErasedMaterialPipelineKey {
                type_id: target.material.untyped().type_id(),
                mesh_key: ErasedMeshPipelineKey::new(mesh_key),
                material_key: properties.material_key.clone(),
            },
            properties,
        });
    }
    Ok(Some(demands))
}
