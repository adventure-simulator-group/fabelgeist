//! Chunk-instanced vista tree cards.
//!
//! A dense woodland vista resolves to tens of thousands of whole-tree
//! billboards - measured at 29,247 for `dense-woodland`, nearly all of them in
//! the 250 m LOD ring. As one entity each they dominated
//! `check_visibility_cpu_culling`, which sweeps every renderable entity once
//! per frame and runs serially on wasm. Chunked, the same stands cost one
//! entity per distinct card variant per wide ground cell.
//!
//! Every per-card difference fits `ChunkInstance`: position and uniform scale,
//! with no rotation at all (the shader yaws each card to the camera). The one
//! value that did not fit is the near cutoff, which scales with the card's own
//! height; `tactical_vista_tree_instanced.wgsl` evaluates it per instance.

use bevy::{
    ecs::system::SystemParam,
    prelude::{Asset, Assets, Commands, Handle, Image, Mesh, Name, ResMut, TypePath, Vec4},
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

use crate::presentation::chunk_instancing::{
    ChunkInstance, ChunkInstancedMaterial, ChunkSize, FadeBand, InstanceBatch, InstanceReach,
    enable_alpha_to_coverage,
};

use super::{VistaLod, VistaTerrain};
use crate::presentation::obstacles::tree::TacticalTreeImpostorMaterial;

const VISTA_TREE_SHADER: &str = "shaders/tactical_vista_tree_instanced.wgsl";
/// Vista rings span kilometres of sparse stands; wide cells keep the entity
/// count low enough for serial CPU culling on wasm.
const VISTA_CARD_CHUNK_SIZE: ChunkSize = ChunkSize(256.0);

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

impl ChunkInstancedMaterial for TacticalVistaTreeInstancedMaterial {
    fn shader() -> ShaderRef {
        VISTA_TREE_SHADER.into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: Self::Data,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // Cards are two-sided and cut out of the atlas through coverage.
        descriptor.primitive.cull_mode = None;
        enable_alpha_to_coverage(descriptor);
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
    instances: Vec<ChunkInstance>,
}

impl VistaCardBatches {
    pub(super) fn push(
        &mut self,
        mesh: &Handle<Mesh>,
        source: &Handle<TacticalTreeImpostorMaterial>,
        card_height: f32,
        instance: ChunkInstance,
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

    /// Spawns chunks per variant. `far_band` is the LOD ring's fade-out;
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
        let level = lod.level;
        for group in self.groups {
            let Some(source) = materials.impostor.get(&group.source) else {
                continue;
            };
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
            InstanceBatch {
                mesh: group.mesh,
                material,
                instances: group.instances,
                band: FadeBand::until(far.clone()),
                // Cards turn to face the camera, so their width reaches as
                // far as their height in every direction.
                reach: InstanceReach {
                    horizontal: group.card_height,
                    above: group.card_height,
                    below: 0.0,
                },
                chunk_size: VISTA_CARD_CHUNK_SIZE,
            }
            .spawn(commands, |chunk| {
                (
                    Name::new(format!("Instanced vista tree cards ({})", chunk.len())),
                    VistaTerrain(level),
                    super::VistaTreePresentation,
                )
            });
        }
    }
}
