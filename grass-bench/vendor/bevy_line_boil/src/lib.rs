//! # Bevy Line Boil
//!
//! The game's single mesh material: classic cartoon "line boil" turbulent
//! vertex displacement, a MeshTag-driven spawn-in bayer dither, and a
//! deliberately tiny shading model (unlit, or one sun + one diffuse sky
//! cubemap) instead of PBR.
//!
//! Why not `ExtendedMaterial<StandardMaterial, _>`: the PBR fragment is huge,
//! and on WebGL2 every pipeline variant compiles synchronously on the main
//! thread (~3.5s each, ~150 variants = an unplayable boot). One small material
//! keeps the whole game inside a handful of cheap pipelines.

use bevy::{
    asset::{load_internal_asset, uuid_handle},
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    prelude::*,
    render::render_resource::{
        AsBindGroup, Face, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

/// Shader handle for the line boil shader
pub const LINE_BOIL_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("89237458-9234-4589-a3ab-cdef12345678");

/// Lighting on: `base * (sun N·L + sky cubemap) + emissive`. Off: `base` as-is.
pub const BOIL_FLAG_LIT: u32 = 1;
/// Alpha-mask mode: discard fragments under `alpha_cutoff` (pair with
/// [`LineBoilMaterial::alpha_mode`] = `AlphaMode::Mask`).
pub const BOIL_FLAG_ALPHA_MASK: u32 = 2;
/// Retro ground dither: a bayer-thresholded two-tone pattern driven by world
/// height + value noise (see the `dither_*` fields on [`BoilShading`]).
pub const BOIL_FLAG_GROUND_DITHER: u32 = 4;

pub struct LineBoilPlugin;

impl Plugin for LineBoilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<LineBoilMaterial>::default());

        load_internal_asset!(
            app,
            LINE_BOIL_SHADER_HANDLE,
            "line_boil.wgsl",
            Shader::from_wgsl
        );
    }
}

/// Settings for the line boil vertex displacement effect.
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct LineBoilSettings {
    /// Displacement intensity (how far vertices move).
    /// Higher values = more aggressive jitter.
    /// Recommended: 0.008 for subtle, 0.04 for aggressive.
    pub intensity: f32,

    /// Frames per second for time quantization.
    /// Lower values = more "held" frames (classic animation look).
    /// Classic animation: 4-8 fps. Smooth: 12-24 fps.
    pub frame_rate: f32,

    /// Noise frequency - controls turbulence scale.
    /// Higher values = more chaotic per-vertex variation.
    pub noise_frequency: f32,

    /// Seed offset for noise variation between entities.
    pub seed: f32,

    /// Spawn-fade flash color (the "white" phase). Over-bright values pop
    /// through tonemapping / bloom.
    pub fade_color: Vec3,

    /// Spawn fade phase 1: transparent -> flash color dither length (secs).
    pub fade_in_secs: f32,

    /// Spawn fade phase 2: flash color -> real color dither length (secs).
    pub fade_to_color_secs: f32,

    /// Down-to-up spawn sweep: fade start delay per meter above the entity
    /// origin.
    pub sweep_secs_per_meter: f32,

    // Padding to a 16-byte multiple for WebGPU compatibility.
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for LineBoilSettings {
    fn default() -> Self {
        Self {
            intensity: 0.02,
            frame_rate: 6.0,
            noise_frequency: 8.0,
            seed: 0.0,
            fade_color: Vec3::splat(2.0),
            fade_in_secs: 0.3,
            fade_to_color_secs: 0.3,
            sweep_secs_per_meter: 0.25,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

/// The whole shading model. All colors are linear.
///
/// `sun_*`/`sky_*` are stamped globally by the game (every material gets the
/// same sun) — see skatepark's `apply_dither_settings`.
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct BoilShading {
    pub base_color: Vec4,
    /// Added after lighting; only applies to lit materials (unlit ignores it,
    /// matching StandardMaterial).
    pub emissive: Vec4,
    /// Unit vector pointing from the surface TOWARD the sun (w unused).
    pub sun_dir: Vec4,
    /// Sun color pre-multiplied by intensity (w unused).
    pub sun_color: Vec4,
    /// Multiplier on the diffuse sky cubemap sample.
    pub sky_strength: f32,
    pub alpha_cutoff: f32,
    /// [`BOIL_FLAG_LIT`] | [`BOIL_FLAG_ALPHA_MASK`]
    pub flags: u32,
    /// NDC depth offset, reverse-Z: positive pulls toward the camera. Replaces
    /// bevy's `Material::depth_bias`, which only nudges phase sort order.
    pub depth_bias: f32,
    /// Viewmodel FOV reprojection: `1/tan(viewmodel_fov/2)`; 0 = off.
    pub fov_inv_tan: f32,
    /// Viewmodel screen sprite-sheet frame count; 0 = not a screen. Layout
    /// (8 cols, 60fps) lives in line_boil.wgsl.
    pub screen_frames: f32,
    /// glTF metallic factor. Lit metals swap the sky-diffuse term for a
    /// view-dependent reflection sample of the same cubemap — soft toon
    /// metal, one extra sample, no extra textures.
    pub metallic: f32,
    /// glTF perceptual roughness. Steers the metal sample direction between
    /// the mirror reflection (0) and the surface normal (1); the cube is
    /// diffuse-convolved, so sharp metals aren't representable anyway.
    pub roughness: f32,
    /// Additive sky sheen on lit surfaces, independent of albedo — PBR's
    /// "~4% dielectric reflectance" in one number. This is what keeps blacks
    /// from going dead black: they lift toward the sky color instead.
    pub sheen: f32,
    /// [`BOIL_FLAG_GROUND_DITHER`] bayer cell size in world meters over XZ —
    /// anchored to the floor, so walking streams the pattern past.
    pub dither_cell: f32,
    /// Ground dither value-noise frequency over world position (1/m).
    pub dither_freq: f32,
    /// Ground dither noise amplitude on the bayer threshold (0 = height only).
    pub dither_noise: f32,
    /// Ground dither threshold shift per meter of world height. Positive
    /// clears the pattern off peaks and pools it in valleys.
    pub dither_height: f32,
    /// Brightness multiplier of the dithered tone (1 = effect invisible).
    pub dither_shade: f32,
    pub _padding1: f32,
    pub _padding2: f32,
}

impl Default for BoilShading {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            emissive: Vec4::ZERO,
            sun_dir: Vec4::new(0.0, 1.0, 0.0, 0.0),
            sun_color: Vec4::ZERO,
            sky_strength: 0.0,
            alpha_cutoff: 0.5,
            flags: 0,
            depth_bias: 0.0,
            fov_inv_tan: 0.0,
            screen_frames: 0.0,
            metallic: 0.0,
            roughness: 1.0,
            sheen: 0.0,
            dither_cell: 0.25,
            dither_freq: 0.15,
            dither_noise: 0.35,
            dither_height: 0.02,
            dither_shade: 1.0,
            _padding1: 0.0,
            _padding2: 0.0,
        }
    }
}

/// Pipeline-relevant bits: everything else is uniform data.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BoilKey {
    pub cull_mode: Option<Face>,
}

impl From<&LineBoilMaterial> for BoilKey {
    fn from(material: &LineBoilMaterial) -> Self {
        Self {
            cull_mode: material.cull_mode,
        }
    }
}

/// The game's mesh material. See module docs.
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
#[bind_group_data(BoilKey)]
pub struct LineBoilMaterial {
    #[uniform(0)]
    pub settings: LineBoilSettings,
    #[uniform(1)]
    pub shading: BoilShading,
    /// None binds the 1x1 white fallback, so "no texture" needs no flag.
    #[texture(2)]
    #[sampler(3)]
    pub base_texture: Option<Handle<Image>>,
    /// Diffuse-convolved sky cubemap (the game's baked gradient sky). None
    /// binds a fallback; harmless because sky_strength defaults to 0.
    #[texture(4, dimension = "cube")]
    #[sampler(5)]
    pub sky_cube: Option<Handle<Image>>,

    // CPU-side pipeline state.
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
    pub depth_bias: f32,
}

impl Default for LineBoilMaterial {
    fn default() -> Self {
        Self {
            settings: LineBoilSettings::default(),
            shading: BoilShading::default(),
            base_texture: None,
            sky_cube: None,
            alpha_mode: AlphaMode::Opaque,
            cull_mode: Some(Face::Back),
            depth_bias: 0.0,
        }
    }
}

impl Material for LineBoilMaterial {
    /// The vertex fn is a faithful bevy 0.19 mesh.wgsl port + the boil
    /// displacement. It MUST keep mesh.wgsl's `vertex_no_morph.instance_index`
    /// naga-dx12 workaround (gfx-rs/naga#2416): the old pasted vertex read
    /// `instance_index` through the `var` copy, which Firefox WebGPU
    /// (naga + dx12) miscompiled into the wrong instance index — meshes wore
    /// other entities' transforms. When porting to a new bevy, re-diff against
    /// mesh.wgsl rather than editing in place.
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(LINE_BOIL_SHADER_HANDLE)
    }

    /// Flat or one-sun shading plus a spawn-in bayer dither driven by
    /// `MeshTag` (spawn time in ms of `globals.time`). Untagged meshes (tag 0)
    /// render normally.
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(LINE_BOIL_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        Ok(())
    }
}

/// Marker for a scene root whose meshes boil. The material is built at glTF
/// load (see skatepark's gltf_materials.rs), so the settings here no longer
/// reach the shader — this remains as the toon-root marker only.
#[derive(Component, Default, Clone)]
pub struct LineBoil {
    pub settings: LineBoilSettings,
}

impl LineBoil {
    /// Create a new LineBoil with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the displacement intensity.
    pub fn with_intensity(mut self, intensity: f32) -> Self {
        self.settings.intensity = intensity;
        self
    }

    /// Set the frame rate for time quantization.
    pub fn with_frame_rate(mut self, fps: f32) -> Self {
        self.settings.frame_rate = fps;
        self
    }

    /// Set the noise frequency.
    pub fn with_noise_frequency(mut self, freq: f32) -> Self {
        self.settings.noise_frequency = freq;
        self
    }

    /// Set the noise seed for variation.
    pub fn with_seed(mut self, seed: f32) -> Self {
        self.settings.seed = seed;
        self
    }

    /// Create with aggressive jitter preset.
    ///
    /// Settings: intensity=0.04, frame_rate=4.0, noise_frequency=12.0
    pub fn aggressive() -> Self {
        Self {
            settings: LineBoilSettings {
                intensity: 0.04,
                frame_rate: 4.0,
                noise_frequency: 12.0,
                ..Default::default()
            },
        }
    }

    /// Create with subtle boil preset.
    ///
    /// Settings: intensity=0.008, frame_rate=8.0, noise_frequency=6.0
    pub fn subtle() -> Self {
        Self {
            settings: LineBoilSettings {
                intensity: 0.008,
                frame_rate: 8.0,
                noise_frequency: 6.0,
                ..Default::default()
            },
        }
    }
}
