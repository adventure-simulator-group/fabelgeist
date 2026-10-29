//! Shared production bark and double-sided leaf bindings for the game and Texture Studio.
use bevy::{
    asset::embedded_asset,
    pbr::{ExtendedMaterial, Material, MaterialExtension},
    prelude::*,
    render::render_resource::{
        AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};
const TREE_BARK_SHADER: &str =
    "embedded://adventuresim_procedural_materials/shaders/tactical_tree_bark.wgsl";
const TREE_LEAF_CARD_SHADER: &str =
    "embedded://adventuresim_procedural_materials/shaders/tactical_tree_leaf_card.wgsl";

pub struct ProceduralMaterialsPlugin;
impl Plugin for ProceduralMaterialsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/tactical_tree_bark.wgsl");
        embedded_asset!(app, "shaders/tactical_tree_leaf_card.wgsl");
        app.add_plugins((
            MaterialPlugin::<TacticalTreeBarkMaterial>::default(),
            MaterialPlugin::<TacticalTreeLeafCardMaterial>::default(),
        ));
    }
}
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TacticalTreeLeafCardMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub opacity: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub front_albedo: Handle<Image>,
    #[texture(4)]
    #[sampler(5)]
    pub back_albedo: Handle<Image>,
    #[texture(6)]
    #[sampler(7)]
    pub front_normal: Handle<Image>,
    #[texture(8)]
    #[sampler(9)]
    pub back_normal: Handle<Image>,
    #[texture(10)]
    #[sampler(11)]
    pub arm: Handle<Image>,
    /// Wind direction XZ, strength, and CPU-synchronized phase time.
    #[uniform(12)]
    pub parameters: Vec4,
    /// Opacity cutoff, tangent-space normal strength, canopy AO strength, and
    /// diffuse transmission for the species' leaf thickness.
    #[uniform(12)]
    pub surface_parameters: Vec4,
    /// Perceptual roughness, physical thickness in metres, ground-litter
    /// vertex-pigment strength, and reserved.
    #[uniform(12)]
    pub physical_parameters: Vec4,
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone)]
pub struct TacticalTreeBarkExtension {
    /// Periodic texture origin, independent of geometry and world position.
    #[uniform(102)]
    pub uv_offset: Vec4,
    /// Canonical scalar relief: normalized height in R and horizon AO in G.
    #[texture(100)]
    #[sampler(101)]
    pub height_ao: Handle<Image>,
    /// Tiles/metre, physical height range, normal strength, and AO strength.
    #[uniform(102)]
    pub relief: Vec4,
    /// Triplanar exponent, branch alignment, parallax fraction, fade distance.
    #[uniform(102)]
    pub projection: Vec4,
    /// Direction toward dominant light and normalized directional strength.
    #[uniform(102)]
    pub lighting: Vec4,
    /// Linear bark pigment and perceptual roughness.
    #[uniform(102)]
    pub surface: Vec4,
    /// Linear soil pigment and perceptual roughness. Soil remains a single
    /// molded albedo; only the binary coverage mask varies spatially.
    #[uniform(102)]
    pub soil_surface: Vec4,
    /// Solid soil height, maximum speck height, cell size, minimum radius.
    #[uniform(102)]
    pub deposition: Vec4,
    /// Playable half extents and encoded minimum/maximum terrain heights.
    #[uniform(102)]
    pub terrain_surface: Vec4,
    /// Soil tiles/metre, physical height range, normal strength, AO strength.
    #[uniform(102)]
    pub soil_response: Vec4,
    /// Soil dielectric reflectance; remaining components are reserved.
    #[uniform(102)]
    pub soil_optics: Vec4,
    /// Row-major playable terrain heightfield encoded into two channels.
    #[texture(103)]
    pub terrain_heightmap: Handle<Image>,
    /// The same packed height/AO surface sampled by tactical terrain.
    #[texture(104)]
    #[sampler(105)]
    pub soil_height_ao: Handle<Image>,
}

impl MaterialExtension for TacticalTreeBarkExtension {
    fn fragment_shader() -> ShaderRef {
        TREE_BARK_SHADER.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        TREE_BARK_SHADER.into()
    }
}

pub type TacticalTreeBarkMaterial = ExtendedMaterial<StandardMaterial, TacticalTreeBarkExtension>;

impl Material for TacticalTreeLeafCardMaterial {
    fn vertex_shader() -> ShaderRef {
        TREE_LEAF_CARD_SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        TREE_LEAF_CARD_SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        // Preserve the procedural cutout, then let 4x MSAA turn its remaining
        // fractional opacity into sample coverage instead of a jagged binary
        // silhouette. This is hardware multisampling and works on WebGPU.
        AlphaMode::AlphaToCoverage
    }

    fn enable_prepass() -> bool {
        true
    }

    fn enable_shadows() -> bool {
        true
    }

    fn prepass_vertex_shader() -> ShaderRef {
        TREE_LEAF_CARD_SHADER.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        TREE_LEAF_CARD_SHADER.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        if let Some(fragment) = descriptor.fragment.as_mut() {
            enable_leaf_transmission_shader_defs(&mut fragment.shader_defs);
        }
        Ok(())
    }
}

pub fn enable_leaf_transmission_shader_defs(shader_defs: &mut Vec<bevy::shader::ShaderDefVal>) {
    for name in [
        "STANDARD_MATERIAL_DIFFUSE_TRANSMISSION",
        "STANDARD_MATERIAL_DIFFUSE_OR_SPECULAR_TRANSMISSION",
    ] {
        let shader_def = bevy::shader::ShaderDefVal::from(name);
        if !shader_defs.contains(&shader_def) {
            shader_defs.push(shader_def);
        }
    }
}
