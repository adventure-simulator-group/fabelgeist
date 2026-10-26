//! Main-pass and depth-prepass pipelines of chunk-instanced materials.
//!
//! Both start from Bevy's mesh pipelines, replace bind group 2 with the
//! chunk band and bind group 3 with the material, and append the
//! per-instance vertex stream. Chunks never cast shadows.
use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use bevy::{
    core_pipeline::{core_3d::CORE_3D_DEPTH_FORMAT, prepass::prepass_target_descriptors},
    mesh::{MeshVertexBufferLayoutRef, VertexBufferLayout},
    pbr::{MeshPipeline, MeshPipelineKey, MeshPipelineViewLayoutKey, PrepassPipeline},
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayoutDescriptor, CompareFunction, DepthBiasState, DepthStencilState, Face,
            FragmentState, MultisampleState, PrimitiveState, RenderPipelineDescriptor,
            SpecializedMeshPipeline, SpecializedMeshPipelineError, StencilState, VertexAttribute,
            VertexFormat, VertexState, VertexStepMode,
        },
        renderer::RenderDevice,
    },
};

use super::{ChunkBandLayout, ChunkInstance, ChunkInstancedMaterial};

/// The pipeline key: the view and mesh key plus the material's own data.
pub(super) struct ChunkPipelineKey<M: ChunkInstancedMaterial> {
    pub mesh_key: MeshPipelineKey,
    pub material_key: M::Data,
}

/// Main-pass pipeline inputs of one material.
#[derive(Resource)]
pub(super) struct ChunkMaterialPipeline<M: ChunkInstancedMaterial> {
    mesh_pipeline: MeshPipeline,
    band_layout: BindGroupLayoutDescriptor,
    pub material_layout: BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    material: PhantomData<M>,
}

/// Depth-prepass pipeline inputs of one material.
#[derive(Resource)]
pub(super) struct ChunkPrepassPipeline<M: ChunkInstancedMaterial> {
    view_layout_motion_vectors: BindGroupLayoutDescriptor,
    view_layout_no_motion_vectors: BindGroupLayoutDescriptor,
    empty_layout: BindGroupLayoutDescriptor,
    band_layout: BindGroupLayoutDescriptor,
    material_layout: BindGroupLayoutDescriptor,
    shader: Handle<Shader>,
    material: PhantomData<M>,
}

impl<M: ChunkInstancedMaterial> Clone for ChunkPipelineKey<M> {
    fn clone(&self) -> Self {
        Self {
            mesh_key: self.mesh_key,
            material_key: self.material_key.clone(),
        }
    }
}

impl<M: ChunkInstancedMaterial> PartialEq for ChunkPipelineKey<M> {
    fn eq(&self, other: &Self) -> bool {
        self.mesh_key == other.mesh_key && self.material_key == other.material_key
    }
}

impl<M: ChunkInstancedMaterial> Eq for ChunkPipelineKey<M> {}

impl<M: ChunkInstancedMaterial> Hash for ChunkPipelineKey<M> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.mesh_key.hash(state);
        self.material_key.hash(state);
    }
}

impl<M: ChunkInstancedMaterial> SpecializedMeshPipeline for ChunkMaterialPipeline<M> {
    type Key = ChunkPipelineKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self.mesh_pipeline.specialize(key.mesh_key, layout)?;
        let mut view_key = MeshPipelineViewLayoutKey::from(key.mesh_key);
        if key
            .mesh_key
            .contains(MeshPipelineKey::MOTION_VECTOR_PREPASS)
        {
            view_key |= MeshPipelineViewLayoutKey::MOTION_VECTOR_PREPASS;
        }
        // Groups 0 and 1 stay Bevy's view layouts; the per-entity mesh
        // uniforms at group 2 give way to the chunk band.
        descriptor.layout.truncate(2);
        if let Some(view_layout) = descriptor.layout.first_mut() {
            *view_layout = self
                .mesh_pipeline
                .get_view_layout(view_key)
                .main_layout
                .clone();
        }
        descriptor
            .layout
            .extend([self.band_layout.clone(), self.material_layout.clone()]);
        // The prepass may have skipped a chunk whose prepass pipeline was
        // still compiling, so the main pass always tests and writes depth.
        if let Some(depth) = descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(true);
            depth.depth_compare = Some(CompareFunction::GreaterEqual);
        }
        descriptor
            .vertex
            .shader_defs
            .push("VISIBILITY_RANGE_DITHER".into());
        descriptor.vertex.shader = self.shader.clone();
        if let Some(fragment) = descriptor.fragment.as_mut() {
            if let Some(Some(target)) = fragment.targets.first_mut() {
                target.blend = None;
            }
            fragment.shader_defs.push("VISIBILITY_RANGE_DITHER".into());
            fragment.shader = self.shader.clone();
        }
        M::specialize(&mut descriptor, layout, key.material_key)?;
        descriptor.vertex.buffers.push(instance_buffer_layout());
        Ok(descriptor)
    }
}

impl<M: ChunkInstancedMaterial> SpecializedMeshPipeline for ChunkPrepassPipeline<M> {
    type Key = ChunkPipelineKey<M>;

    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayoutRef,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut shader_defs = vec!["PREPASS_PIPELINE".into(), "VISIBILITY_RANGE_DITHER".into()];
        // Mesh attributes bind at the forward locations even without a normal
        // prepass, so a material's prepass runs its forward vertex function.
        let mut vertex_attributes = Vec::new();
        for (attribute, location, defs) in [
            (Mesh::ATTRIBUTE_POSITION, 0, &["VERTEX_POSITIONS"][..]),
            (Mesh::ATTRIBUTE_NORMAL, 1, &["VERTEX_NORMALS"][..]),
            (Mesh::ATTRIBUTE_UV_0, 2, &["VERTEX_UVS", "VERTEX_UVS_A"][..]),
            (Mesh::ATTRIBUTE_UV_1, 3, &["VERTEX_UVS_B"][..]),
            (Mesh::ATTRIBUTE_TANGENT, 4, &["VERTEX_TANGENTS"][..]),
            (Mesh::ATTRIBUTE_COLOR, 5, &["VERTEX_COLORS"][..]),
        ] {
            if layout.0.contains(attribute) {
                shader_defs.extend(defs.iter().map(|def| (*def).into()));
                vertex_attributes.push(attribute.at_shader_location(location));
            }
        }
        let normal_prepass = key.mesh_key.contains(MeshPipelineKey::NORMAL_PREPASS);
        let motion_vectors = key
            .mesh_key
            .contains(MeshPipelineKey::MOTION_VECTOR_PREPASS);
        if normal_prepass {
            shader_defs.extend([
                "NORMAL_PREPASS".into(),
                "NORMAL_PREPASS_OR_DEFERRED_PREPASS".into(),
            ]);
        }
        if motion_vectors {
            shader_defs.extend([
                "MOTION_VECTOR_PREPASS".into(),
                "MOTION_VECTOR_PREPASS_OR_DEFERRED_PREPASS".into(),
            ]);
        }
        let mut targets = prepass_target_descriptors(normal_prepass, motion_vectors, false);
        if targets.iter().all(Option::is_none) {
            targets.clear();
        } else {
            shader_defs.push("PREPASS_FRAGMENT".into());
        }
        let view_layout = if motion_vectors {
            self.view_layout_motion_vectors.clone()
        } else {
            self.view_layout_no_motion_vectors.clone()
        };
        let mut descriptor = RenderPipelineDescriptor {
            label: Some("chunk_instanced_prepass_pipeline".into()),
            layout: vec![
                view_layout,
                self.empty_layout.clone(),
                self.band_layout.clone(),
                self.material_layout.clone(),
            ],
            vertex: VertexState {
                shader: self.shader.clone(),
                entry_point: Some("vertex".into()),
                shader_defs: shader_defs.clone(),
                buffers: vec![layout.0.get_layout(&vertex_attributes)?],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs,
                entry_point: Some("fragment".into()),
                targets,
            }),
            primitive: PrimitiveState {
                topology: key.mesh_key.primitive_topology(),
                strip_index_format: key.mesh_key.strip_index_format(),
                cull_mode: Some(Face::Back),
                ..default()
            },
            depth_stencil: Some(DepthStencilState {
                format: CORE_3D_DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::GreaterEqual),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState {
                count: key.mesh_key.msaa_samples(),
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            ..default()
        };
        M::specialize(&mut descriptor, layout, key.material_key)?;
        descriptor.vertex.buffers.push(instance_buffer_layout());
        Ok(descriptor)
    }
}

impl<M: ChunkInstancedMaterial> FromWorld for ChunkMaterialPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let material_layout = BindGroupLayoutDescriptor::new(
            format!(
                "chunk_instanced_material_layout_{}",
                std::any::type_name::<M>()
            ),
            &M::bind_group_layout_entries(render_device, false),
        );
        Self {
            mesh_pipeline: world.resource::<MeshPipeline>().clone(),
            band_layout: world.resource::<ChunkBandLayout>().0.clone(),
            material_layout,
            shader: material_shader::<M>(world),
            material: PhantomData,
        }
    }
}

impl<M: ChunkInstancedMaterial> FromWorld for ChunkPrepassPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let prepass = world.resource::<PrepassPipeline>();
        let main = world.resource::<ChunkMaterialPipeline<M>>();
        Self {
            view_layout_motion_vectors: prepass.view_layout_motion_vectors.clone(),
            view_layout_no_motion_vectors: prepass.view_layout_no_motion_vectors.clone(),
            empty_layout: prepass.empty_layout.clone(),
            band_layout: main.band_layout.clone(),
            material_layout: main.material_layout.clone(),
            shader: main.shader.clone(),
            material: PhantomData,
        }
    }
}

fn material_shader<M: ChunkInstancedMaterial>(world: &World) -> Handle<Shader> {
    match M::shader() {
        bevy::shader::ShaderRef::Handle(handle) => handle,
        bevy::shader::ShaderRef::Path(path) => world.resource::<AssetServer>().load(path),
        bevy::shader::ShaderRef::Default => Handle::default(),
    }
}

/// The `@location(8..10)` instance stream of `chunk_instancing.wgsl`.
fn instance_buffer_layout() -> VertexBufferLayout {
    let rotation_offset = VertexFormat::Float32x4.size();
    let seed_offset = rotation_offset + VertexFormat::Float32.size();
    VertexBufferLayout {
        array_stride: size_of::<ChunkInstance>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: vec![
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: 0,
                shader_location: 8,
            },
            VertexAttribute {
                format: VertexFormat::Float32,
                offset: rotation_offset,
                shader_location: 9,
            },
            VertexAttribute {
                format: VertexFormat::Uint32,
                offset: seed_offset,
                shader_location: 10,
            },
        ],
    }
}
