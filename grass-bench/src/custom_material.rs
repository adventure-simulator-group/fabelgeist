//! The bench's "fixed" custom material: what the game's line-boil material
//! should become for batching and early-Z.
//!
//! - bevy's mesh and prepass vertex shaders (copied, plus a grass bend for
//!   objects flagged as grass), one small fragment shader.
//! - `discard` only under `MAY_DISCARD` (alpha-tested pipelines) so opaque
//!   surfaces keep early depth testing; `FORCE_DISCARD` puts it back into an
//!   opaque pipeline on purpose for the "Opaque + discard" foliage test.
//! - A prepass fragment shader that applies the same alpha test, so the depth
//!   pre-pass / TAA / occlusion culling see cut-out leaves, not full quads.
//! - Per-object colour and parameters live in a fixed uniform table indexed by
//!   `MeshTag`, so every object sharing a texture shares one material handle,
//!   hence one bind group, hence one batch. `params.w = 1` marks grass: the
//!   vertex shader then applies wind and the affector array from the globals
//!   uniform. One shader, behaviour switched by data, not by pipeline.

use bevy::asset::{load_internal_asset, uuid_handle};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
#[cfg(not(feature = "downlevel"))]
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::{
    AsBindGroup, Face, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
#[cfg(not(feature = "downlevel"))]
use bevy::render::render_resource::{Buffer, BufferDescriptor, BufferUsages};
#[cfg(not(feature = "downlevel"))]
use bevy::render::renderer::{RenderDevice, RenderQueue};

#[cfg(not(feature = "downlevel"))]
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::shader::ShaderRef;

use crate::affectors::Affectors;

pub const CUSTOM_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3c1f5f4e-0a6b-4a0e-9a5f-1d2b4c6e8f01");
pub const CUSTOM_PREPASS_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3c1f5f4e-0a6b-4a0e-9a5f-1d2b4c6e8f02");
pub const CUSTOM_BINDINGS_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3c1f5f4e-0a6b-4a0e-9a5f-1d2b4c6e8f03");

pub const CUSTOM_FLAG_LIT: u32 = 1;
pub const MAX_AFFECTORS: usize = 16;
pub const MAX_OBJECTS: usize = 128;

/// Shared by every custom material: the one sun, the sky strength, the wind.
#[derive(ShaderType, Debug, Clone, Copy)]
pub struct CustomGlobals {
    /// Unit vector toward the sun (w unused).
    pub sun_dir: Vec4,
    /// Sun colour pre-multiplied by intensity (w unused).
    pub sun_color: Vec4,
    pub sky_strength: f32,
    pub alpha_cutoff: f32,
    pub flags: u32,
    /// Grass blade width scale, applied in the vertex shader.
    pub blade_width: f32,
    /// Direction xy, tip displacement (m), time scale.
    pub wind: Vec4,
    /// Sprite cards: x world scale, y shape (0 triangle, 1 quad), zw the
    /// atlas's uniform cell in uv (curved cards; zero for the plant atlas).
    pub card: Vec4,
    /// Curved cards: x bend (fraction of the card's width at the tip),
    /// y flutter, zw unused.
    pub curve: Vec4,
    /// Displacement map: min corner (x, z), 1 / size, w: scorch (0..1, how
    /// far a remembered trail burns the grass colour; `trample_scorch`).
    pub map: Vec4,
}

impl Default for CustomGlobals {
    fn default() -> Self {
        Self {
            sun_dir: Vec4::new(0.0, 1.0, 0.0, 0.0),
            sun_color: Vec4::ZERO,
            sky_strength: 1.0,
            alpha_cutoff: 0.5,
            flags: CUSTOM_FLAG_LIT,
            blade_width: 1.0,
            wind: Vec4::new(0.8, 0.6, 0.18, 1.3),
            card: Vec4::new(1.0, 0.0, 0.0, 0.0),
            curve: Vec4::new(0.15, 0.35, 0.0, 0.0),
            map: Vec4::new(0.0, 0.0, 1.0, 0.0),
        }
    }
}

/// `params.w` values: what the vertex and fragment shaders do with an object.
pub const KIND_GRASS: f32 = 1.0;
pub const KIND_CARD: f32 = 2.0;
pub const KIND_GRASS_MAP: f32 = 3.0;
pub const KIND_CARD_CURVED: f32 = 4.0;

/// One packed foliage sprite (see `grass/cards.rs`): three (x, y, u, v)
/// corners of the fitted triangle, four of the tight quad, world height in
/// `meta.x`. Sprite space is x right, y up, origin at the root, one unit =
/// the sprite height.
#[derive(ShaderType, Debug, Clone, Copy, Default)]
pub struct Sprite {
    pub tri: [Vec4; 3],
    pub quad: [Vec4; 4],
    pub info: Vec4,
}

/// The sprite table every custom material binds (only cards read it).
/// Families are contiguous runs: grass, clover, dandelion.
#[derive(ShaderType, Debug, Clone, Default)]
pub struct SpriteTable {
    pub family_start: UVec4,
    pub family_count: UVec4,
    pub sprites: [Sprite; 32],
}

/// Shared sprite table; `grass/cards.rs` fills it.
#[derive(Resource)]
pub struct SpriteTableBuffer {
    pub data: SpriteTable,
}

/// The affector list every custom material binds raw: one persistent GPU
/// buffer (count + 16 x position/radius), written in place each frame by the
/// render world. Changing it never touches a material asset, so no bind
/// group is rebuilt; this is the "uniform array, not a pipeline split" model
/// at its cheapest.
#[cfg(not(feature = "downlevel"))]
#[derive(Resource, Clone, ExtractResource)]
pub struct AffectorBuffer(pub Buffer);

#[cfg(feature = "downlevel")]
#[derive(ShaderType, Debug, Clone, Default)]
pub struct AffectorUniform {
    pub count: UVec4,
    pub items: [Vec4; MAX_AFFECTORS],
}

#[cfg(feature = "downlevel")]
#[derive(Resource, Default)]
pub struct AffectorBuffer(pub AffectorUniform);

#[cfg(not(feature = "downlevel"))]
const AFFECTOR_BUFFER_BYTES: u64 = 16 + 16 * MAX_AFFECTORS as u64;

/// One entry per object, indexed by `MeshTag`.
#[derive(ShaderType, Debug, Clone, Copy, PartialEq)]
pub struct ObjectParams {
    pub base_color: Vec4,
    pub emissive: Vec4,
    /// x metallic, y roughness, z sheen, w = 1 marks grass (vertex bend).
    pub params: Vec4,
}

impl Default for ObjectParams {
    fn default() -> Self {
        Self {
            base_color: Vec4::ONE,
            emissive: Vec4::ZERO,
            params: Vec4::new(0.0, 1.0, 0.0, 0.0),
        }
    }
}

/// Pipeline-relevant bits; everything else is uniform or storage data.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct CustomKey {
    pub cull_mode: Option<Face>,
    pub force_discard: bool,
}

impl From<&CustomMaterial> for CustomKey {
    fn from(material: &CustomMaterial) -> Self {
        Self {
            cull_mode: material.cull_mode,
            force_discard: material.force_discard,
        }
    }
}

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
#[bind_group_data(CustomKey)]
pub struct CustomMaterial {
    #[uniform(0)]
    pub globals: CustomGlobals,
    #[texture(1)]
    #[sampler(2)]
    pub base_texture: Option<Handle<Image>>,
    #[texture(3, dimension = "cube")]
    #[sampler(4)]
    pub sky_cube: Option<Handle<Image>>,
    #[uniform(5)]
    pub objects: [ObjectParams; MAX_OBJECTS],
    #[cfg(not(feature = "downlevel"))]
    #[storage(6, read_only, buffer)]
    pub affectors: Buffer,
    #[cfg(feature = "downlevel")]
    #[uniform(6)]
    pub affectors: AffectorUniform,
    // Share binding 5 to stay within WebGL2's 11 uniform buffers per stage.
    #[uniform(5)]
    pub sprites: SpriteTable,
    /// The displacement map (`displacement.rs`); only map-mode grass reads it.
    #[texture(8)]
    #[sampler(9)]
    pub displacement: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    pub cull_mode: Option<Face>,
    /// `discard` inside an opaque pipeline: the early-Z killer under test.
    pub force_discard: bool,
}

impl Material for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(CUSTOM_SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(CUSTOM_SHADER_HANDLE)
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Handle(CUSTOM_PREPASS_SHADER_HANDLE)
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(CUSTOM_PREPASS_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        #[cfg(feature = "downlevel")]
        {
            descriptor.vertex.shader_defs.push("DOWNLEVEL".into());
            if let Some(fragment) = descriptor.fragment.as_mut() {
                fragment.shader_defs.push("DOWNLEVEL".into());
            }
        }
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;
        if key.bind_group_data.force_discard
            && let Some(fragment) = descriptor.fragment.as_mut()
        {
            fragment.shader_defs.push("FORCE_DISCARD".into());
        }
        Ok(())
    }
}

/// The per-object parameter table every custom material binds.
#[derive(Resource)]
pub struct ObjectParamsBuffer {
    pub data: [ObjectParams; MAX_OBJECTS],
    len: usize,
    dirty: bool,
}

impl ObjectParamsBuffer {
    /// Reuses identical parameters across meshes and returns their `MeshTag`.
    pub fn push(&mut self, params: ObjectParams) -> u32 {
        if let Some(index) = self.data[..self.len].iter().position(|value| *value == params) {
            return index as u32;
        }
        assert!(self.len < MAX_OBJECTS, "object parameter table full");
        self.data[self.len] = params;
        self.len += 1;
        self.dirty = true;
        (self.len - 1) as u32
    }
}

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            CUSTOM_BINDINGS_SHADER_HANDLE,
            "shaders/custom_bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CUSTOM_SHADER_HANDLE,
            "shaders/custom.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            CUSTOM_PREPASS_SHADER_HANDLE,
            "shaders/custom_prepass.wgsl",
            Shader::from_wgsl
        );
        app.add_plugins(MaterialPlugin::<CustomMaterial>::default())
            .add_systems(PostUpdate, (upload_object_params, apply_grass_knobs));
        #[cfg(not(feature = "downlevel"))]
        {
            app.add_plugins((
                ExtractResourcePlugin::<AffectorBuffer>::default(),
                ExtractResourcePlugin::<Affectors>::default(),
            ))
            .add_systems(PreStartup, create_affector_buffer);
            app.sub_app_mut(RenderApp).add_systems(
                Render,
                write_affectors.in_set(RenderSystems::PrepareResources),
            );
        }
        #[cfg(feature = "downlevel")]
        app.init_resource::<AffectorBuffer>()
            .add_systems(PostUpdate, update_affector_uniforms);
        app.insert_resource(ObjectParamsBuffer {
            data: [ObjectParams::default(); MAX_OBJECTS],
            len: 1,
            dirty: false,
        });
        app.insert_resource(SpriteTableBuffer {
            data: SpriteTable::default(),
        });
    }
}

/// Copies the shared uniform table to materials when objects are added.
fn upload_object_params(
    mut table: ResMut<ObjectParamsBuffer>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    if !table.dirty {
        return;
    }
    table.dirty = false;
    for (_, material) in materials.iter_mut() {
        material.objects = table.data;
    }
}

/// The render device only exists once the renderer is up, so the shared
/// affector buffer is made at startup, before any material is created.
#[cfg(not(feature = "downlevel"))]
fn create_affector_buffer(mut commands: Commands, device: Res<RenderDevice>) {
    commands.insert_resource(AffectorBuffer(device.create_buffer(&BufferDescriptor {
        label: Some("custom_affectors"),
        size: AFFECTOR_BUFFER_BYTES,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })));
}

/// Render world: packs the extracted affector list and writes it into the
/// persistent buffer in place.
#[cfg(not(feature = "downlevel"))]
fn write_affectors(
    affectors: Option<Res<Affectors>>,
    buffer: Option<Res<AffectorBuffer>>,
    queue: Res<RenderQueue>,
    mut was_empty: Local<bool>,
) {
    let (Some(affectors), Some(buffer)) = (affectors, buffer) else {
        return;
    };
    if affectors.list.is_empty() && *was_empty {
        return;
    }
    *was_empty = affectors.list.is_empty();
    let count = affectors.list.len().min(MAX_AFFECTORS);
    let mut bytes = Vec::with_capacity(AFFECTOR_BUFFER_BYTES as usize);
    bytes.extend_from_slice(&(count as u32).to_le_bytes());
    bytes.extend_from_slice(&[0u8; 12]);
    for slot in 0..MAX_AFFECTORS {
        let value = affectors
            .list
            .get(slot)
            // y carries the heading (radians) so the trample pass can flatten
            // along the direction of travel; the grass only reads xz and w.
            .map(|a| {
                Vec4::new(
                    a.position.x,
                    a.velocity.z.atan2(a.velocity.x),
                    a.position.z,
                    a.radius,
                )
            })
            .unwrap_or(Vec4::ZERO);
        for component in value.to_array() {
            bytes.extend_from_slice(&component.to_le_bytes());
        }
    }
    queue.write_buffer(&buffer.0, 0, &bytes);
}

/// Pushes the blade width and scorch knobs into every custom material when
/// they change (a slider move, not per frame). Materials created later pick
/// the scorch up from the knob themselves (`grass/mesh_chunks.rs`).
fn apply_grass_knobs(
    settings: Res<crate::settings::BenchSettings>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut last: Local<Option<(f32, f32)>>,
) {
    let knobs = (settings.grass_width, settings.trample_scorch);
    if *last == Some(knobs) {
        return;
    }
    *last = Some(knobs);
    for (_, material) in materials.iter_mut() {
        material.globals.blade_width = settings.grass_width;
        material.globals.map.w = settings.trample_scorch;
    }
}

#[cfg(feature = "downlevel")]
fn update_affector_uniforms(
    affectors: Res<Affectors>,
    mut buffer: ResMut<AffectorBuffer>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut maps: ResMut<Assets<crate::displacement::DisplacementMaterial>>,
    mut trails: ResMut<Assets<crate::displacement::TrampleMaterial>>,
) {
    if !affectors.is_changed() {
        return;
    }
    buffer.0.count = UVec4::new(affectors.list.len().min(MAX_AFFECTORS) as u32, 0, 0, 0);
    for (slot, value) in buffer.0.items.iter_mut().enumerate() {
        *value = affectors
            .list
            .get(slot)
            .map(|a| {
                Vec4::new(
                    a.position.x,
                    a.velocity.z.atan2(a.velocity.x),
                    a.position.z,
                    a.radius,
                )
            })
            .unwrap_or(Vec4::ZERO);
    }
    for (_, m) in materials.iter_mut() {
        m.affectors = buffer.0.clone();
    }
    for (_, m) in maps.iter_mut() {
        m.affectors = buffer.0.clone();
    }
    for (_, m) in trails.iter_mut() {
        m.affectors = buffer.0.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_scene_spawns_reuse_object_parameters() {
        let mut table = ObjectParamsBuffer {
            data: [ObjectParams::default(); MAX_OBJECTS],
            len: 1,
            dirty: false,
        };
        let grass = ObjectParams {
            params: Vec4::new(0.0, 1.0, 0.0, KIND_GRASS),
            ..default()
        };
        let tag = table.push(grass);
        assert_ne!(tag, table.push(ObjectParams::default()));
        // Repeated tree/character spawns must not exhaust the fixed uniform.
        for _ in 0..MAX_OBJECTS * 4 {
            assert_eq!(table.push(grass), tag);
            assert_eq!(table.push(ObjectParams::default()), 0);
        }
    }
}
