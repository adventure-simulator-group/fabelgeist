//! The material hair is drawn with: the standard material, shaded as
//! strands and, for cards, its atlas coverage turned into a share of each
//! pixel's samples, by `strand_shading.wgsl`. The strand normal it and the
//! scalp's material share lives in `strand.wgsl`.
//!
//! Hair is built on the GPU it is drawn on and drawn from where it was
//! built: the material binds the buffers its strips were built into, and
//! its vertex shaders (`hair_vertex.wgsl` for the main pass,
//! `hair_prepass.wgsl` for the prepass and shadows) pull each vertex from
//! them (`hair_vertices.wgsl`). Its mesh carries only how each vertex is
//! skinned, so no vertex comes back to the CPU.
use super::*;
use adventuresim_character_creator::hair::GpuGeometry;
use bevy::asset::embedded_asset;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::render::render_resource::{
    AsBindGroup, Buffer, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::shader::{ShaderDefVal, ShaderRef, load_shader_library};

const SHADER: &str = "embedded://adventuresim_character_creator/hair_preview/strand_shading.wgsl";
const VERTEX: &str = "embedded://adventuresim_character_creator/hair_preview/hair_vertex.wgsl";
const PREPASS_VERTEX: &str =
    "embedded://adventuresim_character_creator/hair_preview/hair_prepass.wgsl";
/// Bevy's shader definition for the prepass and shadow pipelines.
const PREPASS: &str = "PREPASS_PIPELINE";
/// Bevy's shader definition for a prepass writing normals.
const PREPASS_NORMALS: &str = "NORMAL_PREPASS_OR_DEFERRED_PREPASS";

/// Hair's material, for strands and cards alike.
pub(crate) type HairShading = ExtendedMaterial<StandardMaterial, StrandShading>;

/// The standard material's extension for hair: the buffers its strips were
/// built into, four floats a vertex, which its vertex shaders draw from.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
#[reflect(opaque)]
#[reflect(Clone, Debug)]
pub(crate) struct StrandShading {
    /// Position, and baked sky visibility.
    #[storage(102, read_only, buffer)]
    positions: Buffer,
    /// Normal, and the strip's own random value.
    #[storage(103, read_only, buffer)]
    normals: Buffer,
    /// Along the strands, root to tip.
    #[storage(104, read_only, buffer)]
    tangents: Buffer,
    /// Texture coordinates.
    #[storage(105, read_only, buffer)]
    texcoords: Buffer,
}

impl StrandShading {
    /// Drawing `geometry` from where it was built.
    pub(super) fn new(geometry: &GpuGeometry) -> Self {
        let bind =
            |buffer: &fabelgeist_gpu::prelude::Buffer| Buffer::from((*buffer.buffer).clone());
        Self {
            positions: bind(geometry.positions()),
            normals: bind(geometry.normals()),
            tangents: bind(geometry.tangents()),
            texcoords: bind(geometry.texcoords()),
        }
    }
}

impl MaterialExtension for StrandShading {
    fn vertex_shader() -> ShaderRef {
        VERTEX.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        PREPASS_VERTEX.into()
    }

    /// The vertex attributes the hair's buffers hold, as Bevy defines them
    /// for a mesh carrying them: the mesh itself carries only its skin.
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let defined = |name: &str| {
            descriptor
                .vertex
                .shader_defs
                .contains(&ShaderDefVal::from(name))
        };
        let mut attributes = vec![
            "VERTEX_UVS",
            "VERTEX_UVS_A",
            "VERTEX_UVS_B",
            "VERTEX_COLORS",
        ];
        if !defined(PREPASS) || defined(PREPASS_NORMALS) {
            attributes.extend(["VERTEX_NORMALS", "VERTEX_TANGENTS"]);
        }
        let defs = attributes.into_iter().map(ShaderDefVal::from);
        descriptor.vertex.shader_defs.extend(defs.clone());
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs.extend(defs);
        }
        Ok(())
    }
}

/// Draws hair with [`HairShading`] and the scalp under it with
/// [`scalp_shade::ScalpShading`].
pub(crate) struct HairShadingPlugin;

impl Plugin for HairShadingPlugin {
    fn build(&self, app: &mut App) {
        load_shader_library!(app, "strand.wgsl");
        load_shader_library!(app, "hair_vertices.wgsl");
        embedded_asset!(app, "strand_shading.wgsl");
        embedded_asset!(app, "hair_vertex.wgsl");
        embedded_asset!(app, "hair_prepass.wgsl");
        embedded_asset!(app, "scalp_shading.wgsl");
        app.add_plugins((
            MaterialPlugin::<HairShading>::default(),
            MaterialPlugin::<scalp_shade::ScalpShading>::default(),
            scalp_upload::ScalpUploadPlugin,
        ));
    }
}
