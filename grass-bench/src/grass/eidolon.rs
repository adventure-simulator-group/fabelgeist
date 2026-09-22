//! The bevy_eidolon grass path: the game's renderer as-is. One batch entity
//! per tier carrying every tuft of the patch; eidolon's compute pass culls
//! per instance (frustum + fade band) and draws indirect.

use std::sync::Arc;

use bevy::camera::{primitives::Aabb, visibility::NoFrustumCulling};
use bevy::light::NotShadowCaster;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use bevy_eidolon::prelude::*;
use bevy_eidolon::prepass::CullComputeCamera;

use super::{GRASS_EIDOLON_SHADER_HANDLE, GrassEntity, TIER_COUNT, Tier, TierMesh, fitted_aabb, tier_params};
use crate::settings::{BenchSettings, InstancingMode};

/// The instanced grass material: Fabelgeist's `TacticalGrassInstancedMaterial`.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[uniform(0, GrassUniform)]
pub struct GrassMaterial {
    pub wind: Vec4,
    pub interaction: Vec4,
    pub interaction_motion: Vec4,
    pub params: Vec4,
    pub shading: Vec4,
}

#[derive(Clone, Default, ShaderType)]
pub struct GrassUniform {
    wind: Vec4,
    interaction: Vec4,
    interaction_motion: Vec4,
    params: Vec4,
    shading: Vec4,
}

impl From<&GrassMaterial> for GrassUniform {
    fn from(material: &GrassMaterial) -> Self {
        Self {
            wind: material.wind,
            interaction: material.interaction,
            interaction_motion: material.interaction_motion,
            params: material.params,
            shading: material.shading,
        }
    }
}

impl InstancedMaterial for GrassMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(GRASS_EIDOLON_SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GRASS_EIDOLON_SHADER_HANDLE)
    }

    /// The game's grass renders without a prepass.
    fn disable_prepass(&self) -> bool {
        true
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Thin ribbons are shaded double-sided, and the LOD dither converts
        // to hardware coverage samples under MSAA.
        descriptor.primitive.cull_mode = None;
        if descriptor.multisample.count > 1 {
            descriptor.multisample.alpha_to_coverage_enabled = true;
        }
        Ok(())
    }
}

pub struct GrassEidolonPlugin;

impl Plugin for GrassEidolonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InstancedMaterialCorePlugin,
            GpuComputeCullCorePlugin,
            InstancedMaterialPlugin::<GrassMaterial>::default(),
            GpuCullComputePlugin::<GrassMaterial>::default(),
        ))
        .add_systems(Update, sync_cull_camera);
    }
}

/// The camera drives eidolon's per-instance compute cull while the eidolon
/// path is selected; the marker is removed otherwise so the other modes carry
/// none of its per-frame work.
fn sync_cull_camera(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    without: Query<Entity, (With<Camera3d>, Without<CullComputeCamera>)>,
    with: Query<Entity, (With<Camera3d>, With<CullComputeCamera>)>,
) {
    if settings.instancing == InstancingMode::Eidolon {
        for camera in &without {
            // try_insert: the settings system may have respawned this camera in
            // the same frame (wasm panics on a despawned target).
            commands.entity(camera).try_insert(CullComputeCamera);
        }
    } else {
        for camera in &with {
            commands.entity(camera).try_remove::<CullComputeCamera>();
        }
    }
}

/// One batch entity per tier, the exact bundle Fabelgeist's
/// `spawn_tuft_batches` uses.
pub fn spawn(
    commands: &mut Commands,
    materials: &mut Assets<GrassMaterial>,
    tiers: &[Tier; TIER_COUNT],
    tier_meshes: &[TierMesh],
    batches: [Vec<InstanceData>; TIER_COUNT],
    grass_shadows: bool,
) {
    for ((tier, mesh), instances) in tiers.iter().zip(tier_meshes).zip(batches) {
        if instances.is_empty() {
            continue;
        }
        let params = tier_params(tier);
        let material = materials.add(GrassMaterial {
            wind: params.wind,
            interaction: params.interaction,
            interaction_motion: params.interaction_motion,
            params: params.params,
            shading: params.shading,
        });
        let aabb: Aabb = fitted_aabb(&instances, tier.footprint());
        let mut entity = commands.spawn((
            Name::new(format!("grass eidolon {} ({} tufts)", tier.name, instances.len())),
            GrassEntity,
            GpuCullCompute,
            // Batches span the whole patch: CPU frustum culling could only hide
            // them wholesale, and a culled frame makes eidolon free and
            // re-upload the retained instance buffers. Culling belongs solely
            // to the GPU compute pass.
            NoFrustumCulling,
            Mesh3d(mesh.handle.clone()),
            InstancedMeshMaterial(material),
            aabb,
            InstanceMaterialData {
                instances: Arc::new(instances),
                color: LinearRgba::WHITE,
                visibility_range: tier.visibility_range(),
            },
            Transform::default(),
            Visibility::Inherited,
        ));
        if !(tier.casts_shadows && grass_shadows) {
            entity.insert(NotShadowCaster);
        }
    }
}
