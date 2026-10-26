//! GPU-instanced vista tree cards.
//!
//! A dense woodland vista resolves to tens of thousands of whole-tree
//! billboards - measured at 29,247 for `dense-woodland`, nearly all of them in
//! the 250 m LOD ring. As one entity each they dominated
//! `check_visibility_cpu_culling`, which sweeps every renderable entity once
//! per frame and runs serially on wasm. Batched, the same stands cost one
//! entity per distinct card variant.
//!
//! Every per-card difference fits `InstanceData`: position and uniform scale,
//! with no rotation at all (the shader yaws each card to the camera). The one
//! value that did not fit is the near cutoff, which scales with the card's own
//! height; `tactical_vista_tree_instanced.wgsl` evaluates it per instance.

use std::sync::Arc;

use bevy::{
    camera::{primitives::Aabb, visibility::NoFrustumCulling},
    color::LinearRgba,
    ecs::system::SystemParam,
    prelude::{
        Asset, Assets, Commands, Handle, Image, Mesh, Mesh3d, Name, ResMut, Transform, TypePath,
        Vec4, Visibility,
    },
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
use bevy_eidolon::prelude::*;

use super::{VistaLod, VistaTerrain};
use crate::presentation::obstacles::tree::TacticalTreeImpostorMaterial;

const VISTA_TREE_SHADER: &str = "shaders/tactical_vista_tree_instanced.wgsl";

/// Instanced twin of `TacticalTreeImpostorMaterial`, restricted to the
/// whole-tree camera-facing representation every vista stand uses.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(in crate::presentation) struct TacticalVistaTreeInstancedMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(in crate::presentation) baked_color: Handle<Image>,
    /// Unscaled card height, deterministic seed, wind strength, wind speed.
    /// The per-entity material carried the representation level in x; every
    /// vista card is level 4, so x now carries the height the near cutoff
    /// needs.
    #[uniform(2)]
    pub(in crate::presentation) parameters: Vec4,
    #[uniform(2)]
    pub(in crate::presentation) lighting: Vec4,
    #[uniform(2)]
    pub(in crate::presentation) ambient: Vec4,
}

impl InstancedMaterial for TacticalVistaTreeInstancedMaterial {
    fn prepass_shader() -> ShaderRef {
        VISTA_TREE_SHADER.into()
    }

    /// The prepass runs this material's own vertex function and repeats its
    /// discards, so its depth matches the main pass exactly.
    fn prepass_fragment() -> bool {
        true
    }

    fn vertex_shader() -> ShaderRef {
        VISTA_TREE_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        VISTA_TREE_SHADER.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Cards are two-sided and cut out of the atlas through coverage.
        descriptor.primitive.cull_mode = None;
        crate::presentation::enable_instanced_alpha_to_coverage(descriptor);
        Ok(())
    }
}

/// The two material stores a vista card batch draws from, as one parameter so
/// the caller's signature does not grow with the instanced twin.
#[derive(SystemParam)]
pub(in crate::presentation) struct VistaTreeMaterials<'w> {
    pub(in crate::presentation) impostor: ResMut<'w, Assets<TacticalTreeImpostorMaterial>>,
    pub(in crate::presentation) cards: ResMut<'w, Assets<TacticalVistaTreeInstancedMaterial>>,
}

/// Collects instances per distinct card variant while the lattice is walked.
#[derive(Default)]
pub(super) struct VistaCardBatches {
    groups: Vec<VistaCardGroup>,
}

struct VistaCardGroup {
    mesh: Handle<Mesh>,
    source: Handle<TacticalTreeImpostorMaterial>,
    card_height: f32,
    instances: Vec<InstanceData>,
}

impl VistaCardBatches {
    pub(super) fn push(
        &mut self,
        mesh: &Handle<Mesh>,
        source: &Handle<TacticalTreeImpostorMaterial>,
        card_height: f32,
        instance: InstanceData,
    ) {
        if let Some(group) = self
            .groups
            .iter_mut()
            .find(|group| &group.source == source && &group.mesh == mesh)
        {
            group.instances.push(instance);
            return;
        }
        self.groups.push(VistaCardGroup {
            mesh: mesh.clone(),
            source: source.clone(),
            card_height,
            instances: vec![instance],
        });
    }

    /// Spawns one batch per variant. `far_band` is the LOD ring's fade-out;
    /// the near end stays zero because the shader applies each card's own
    /// cutoff, which varies with its instance scale.
    pub(super) fn spawn(
        self,
        commands: &mut Commands<'_, '_>,
        lod: &VistaLod,
        materials: &mut VistaTreeMaterials<'_>,
    ) {
        // Cards in a ring share the ring's fade-out; only the near cutoff
        // varies per card, and the shader derives that from the instance scale.
        let far = super::vista_tree_visibility(lod.spacing_metres, 1.0, 1.0).end_margin;
        let far_band = (far.start, far.end);
        let level = lod.level;
        for group in self.groups {
            let Some(source) = materials.impostor.get(&group.source) else {
                continue;
            };
            let instances = Arc::new(group.instances);
            let material = materials.cards.add(TacticalVistaTreeInstancedMaterial {
                baked_color: source.baked_color.clone(),
                parameters: Vec4::new(
                    group.card_height,
                    source.parameters.y,
                    source.parameters.z,
                    source.parameters.w,
                ),
                lighting: source.lighting,
                ambient: source.ambient,
            });
            let aabb = card_batch_aabb(&instances, group.card_height);
            commands.spawn((
                Name::new(format!("Instanced vista tree cards ({})", instances.len())),
                VistaTerrain(level),
                super::VistaTreePresentation,
                GpuCullCompute,
                // Whole-ring batches: culling belongs to the GPU compute pass,
                // and a CPU-culled frame would free the retained instance
                // buffers (see the grass batches).
                NoFrustumCulling,
                Mesh3d(group.mesh),
                aabb,
                InstanceMaterialData {
                    instances,
                    color: LinearRgba::WHITE,
                    visibility_range: Vec4::new(0.0, 0.001, far_band.0, far_band.1),
                },
                InstancedMeshMaterial(material),
                Transform::default(),
                Visibility::Inherited,
            ));
        }
    }
}

/// Bounds every card in the batch. Cards are camera-facing, so the horizontal
/// half-extent uses the card height too rather than an authored width.
fn card_batch_aabb(instances: &[InstanceData], card_height: f32) -> Aabb {
    let mut minimum = bevy::math::Vec3::splat(f32::INFINITY);
    let mut maximum = bevy::math::Vec3::splat(f32::NEG_INFINITY);
    let mut widest = 1.0_f32;
    for instance in instances {
        minimum = minimum.min(instance.position);
        maximum = maximum.max(instance.position);
        widest = widest.max(instance.scale);
    }
    let reach = card_height * widest;
    let minimum = minimum - bevy::math::Vec3::new(reach, 0.0, reach);
    let maximum = maximum + bevy::math::Vec3::new(reach, reach, reach);
    Aabb::from_min_max(minimum, maximum)
}
