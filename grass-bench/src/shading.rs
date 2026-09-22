//! The shading switch. Every mesh that arrives with a `StandardMaterial`
//! (glTF, the ground, the placeholder tree) gets a [`MaterialSet`]: the
//! standard handle plus an equivalent line-boil material and an equivalent
//! bench custom material derived from it. The active `MeshMaterial3d<_>`
//! component follows the `shading` knob; the `foliage_alpha` knob rewrites
//! the alpha mode of every [`Foliage`] material in all three sets.

use std::collections::{HashMap, HashSet};

use bevy::asset::{AssetId, UntypedAssetId};
use bevy::mesh::MeshTag;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy_line_boil::{BOIL_FLAG_ALPHA_MASK, BOIL_FLAG_LIT, BoilShading, LineBoilMaterial, LineBoilSettings};

use crate::custom_material::{AffectorBuffer, CUSTOM_FLAG_LIT, CustomGlobals, CustomMaterial, ObjectParams, ObjectParamsBuffer, SpriteTableBuffer};
use crate::settings::{BenchSettings, FoliageAlpha, ShadingMode};

/// The three renderings of one surface.
#[derive(Component, Clone)]
pub struct MaterialSet {
    pub standard: Handle<StandardMaterial>,
    pub boil: Handle<LineBoilMaterial>,
    pub custom: Handle<CustomMaterial>,
}

/// Alpha-textured surface: the foliage alpha knob applies.
#[derive(Component)]
pub struct Foliage;

/// The one sun and sky the non-PBR materials shade with, in the same
/// exposure-scaled units the PBR path lands in after `view.exposure`.
#[derive(Resource, Clone)]
pub struct SunSky {
    pub sun_dir: Vec3,
    pub sun_color: Vec4,
    pub sky_strength: f32,
    pub sky_cube: Handle<Image>,
}

#[derive(Hash, PartialEq, Eq, Clone)]
struct CustomCacheKey {
    texture: Option<AssetId<Image>>,
    /// `AlphaMode` is not `Hash`; the pipeline-relevant kind is enough.
    alpha_kind: u8,
    cull_mode: Option<Face>,
    lit: bool,
}

fn alpha_kind(mode: AlphaMode) -> u8 {
    match mode {
        AlphaMode::Opaque => 0,
        AlphaMode::Mask(_) => 1,
        AlphaMode::Blend => 2,
        AlphaMode::AlphaToCoverage => 3,
        AlphaMode::Premultiplied => 4,
        AlphaMode::Add => 5,
        AlphaMode::Multiply => 6,
    }
}

#[derive(Resource, Default)]
struct Caches {
    boil: HashMap<AssetId<StandardMaterial>, Handle<LineBoilMaterial>>,
    custom: HashMap<CustomCacheKey, Handle<CustomMaterial>>,
}

pub struct ShadingPlugin;

impl Plugin for ShadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Caches>().add_systems(
            Update,
            (convert_new_materials, apply_shading, apply_foliage_alpha).chain(),
        );
    }
}

fn is_foliage(material: &StandardMaterial, name: Option<&Name>) -> bool {
    if !matches!(material.alpha_mode, AlphaMode::Opaque) {
        return true;
    }
    name.map(|n| {
        let n = n.as_str().to_ascii_lowercase();
        ["leaf", "leaves", "foliage", "crown", "canopy"]
            .iter()
            .any(|k| n.contains(k))
    })
    .unwrap_or(false)
}

fn boil_from_standard(material: &StandardMaterial, sun_sky: &SunSky) -> LineBoilMaterial {
    let mut flags = 0;
    if !material.unlit {
        flags |= BOIL_FLAG_LIT;
    }
    let mut alpha_cutoff = 0.5;
    match material.alpha_mode {
        AlphaMode::Mask(cutoff) => {
            flags |= BOIL_FLAG_ALPHA_MASK;
            alpha_cutoff = cutoff;
        }
        AlphaMode::AlphaToCoverage => flags |= BOIL_FLAG_ALPHA_MASK,
        _ => {}
    }
    LineBoilMaterial {
        settings: LineBoilSettings {
            intensity: 0.0,
            ..default()
        },
        shading: BoilShading {
            base_color: Vec4::from_array(material.base_color.to_linear().to_f32_array()),
            emissive: Vec4::from_array(material.emissive.to_f32_array()),
            metallic: material.metallic,
            roughness: material.perceptual_roughness,
            sun_dir: sun_sky.sun_dir.extend(0.0),
            sun_color: sun_sky.sun_color,
            sky_strength: sun_sky.sky_strength,
            sheen: 0.3,
            flags,
            alpha_cutoff,
            ..default()
        },
        base_texture: material.base_color_texture.clone(),
        sky_cube: Some(sun_sky.sky_cube.clone()),
        alpha_mode: material.alpha_mode,
        cull_mode: material.cull_mode,
        depth_bias: 0.0,
    }
}

fn object_params(material: &StandardMaterial) -> ObjectParams {
    ObjectParams {
        base_color: Vec4::from_array(material.base_color.to_linear().to_f32_array()),
        emissive: Vec4::from_array(material.emissive.to_f32_array()),
        params: Vec4::new(material.metallic, material.perceptual_roughness, 0.3, 0.0),
    }
}

/// Builds the boil and custom twins of every newly seen StandardMaterial
/// mesh, once the standard asset has loaded.
fn convert_new_materials(
    mut commands: Commands,
    sun_sky: Option<Res<SunSky>>,
    standards: Res<Assets<StandardMaterial>>,
    mut boils: ResMut<Assets<LineBoilMaterial>>,
    mut customs: ResMut<Assets<CustomMaterial>>,
    mut caches: ResMut<Caches>,
    mut objects: ResMut<ObjectParamsBuffer>,
    affector_buffer: Res<AffectorBuffer>,
    sprite_table: Res<SpriteTableBuffer>,
    pending: Query<(Entity, &MeshMaterial3d<StandardMaterial>, Option<&Name>), Without<MaterialSet>>,
) {
    let Some(sun_sky) = sun_sky else {
        return;
    };
    for (entity, handle, name) in &pending {
        let Some(material) = standards.get(&handle.0) else {
            continue;
        };
        let boil = caches
            .boil
            .entry(handle.0.id())
            .or_insert_with(|| boils.add(boil_from_standard(material, &sun_sky)))
            .clone();
        let key = CustomCacheKey {
            texture: material.base_color_texture.as_ref().map(|t| t.id()),
            alpha_kind: alpha_kind(material.alpha_mode),
            cull_mode: material.cull_mode,
            lit: !material.unlit,
        };
        let objects_handle = objects.handle.clone();
        let custom = caches
            .custom
            .entry(key)
            .or_insert_with(|| {
                customs.add(CustomMaterial {
                    globals: CustomGlobals {
                        sun_dir: sun_sky.sun_dir.extend(0.0),
                        sun_color: sun_sky.sun_color,
                        sky_strength: sun_sky.sky_strength,
                        alpha_cutoff: match material.alpha_mode {
                            AlphaMode::Mask(c) => c,
                            _ => 0.5,
                        },
                        flags: if material.unlit { 0 } else { CUSTOM_FLAG_LIT },
                        ..default()
                    },
                    base_texture: material.base_color_texture.clone(),
                    sky_cube: Some(sun_sky.sky_cube.clone()),
                    objects: objects_handle,
                    affectors: affector_buffer.0.clone(),
                    sprites: sprite_table.handle.clone(),
                    displacement: None,
                    alpha_mode: material.alpha_mode,
                    cull_mode: material.cull_mode,
                    force_discard: false,
                })
            })
            .clone();
        let tag = objects.push(object_params(material));
        let mut e = commands.entity(entity);
        e.try_insert((
            MaterialSet {
                standard: handle.0.clone(),
                boil,
                custom,
            },
            MeshTag(tag),
        ));
        if is_foliage(material, name) {
            e.try_insert(Foliage);
        }
    }
}

/// Puts the selected material component on every converted mesh.
fn apply_shading(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    all: Query<(Entity, &MaterialSet)>,
    added: Query<(Entity, &MaterialSet), Added<MaterialSet>>,
) {
    let mut apply = |entity: Entity, set: &MaterialSet| {
        let mut e = commands.entity(entity);
        e.try_remove::<(
            MeshMaterial3d<StandardMaterial>,
            MeshMaterial3d<LineBoilMaterial>,
            MeshMaterial3d<CustomMaterial>,
        )>();
        match settings.shading {
            ShadingMode::Standard => {
                e.try_insert(MeshMaterial3d(set.standard.clone()));
            }
            ShadingMode::LineBoil => {
                e.try_insert(MeshMaterial3d(set.boil.clone()));
            }
            ShadingMode::Custom => {
                e.try_insert(MeshMaterial3d(set.custom.clone()));
            }
        }
    };
    if settings.is_changed() {
        for (entity, set) in &all {
            apply(entity, set);
        }
    } else {
        for (entity, set) in &added {
            apply(entity, set);
        }
    }
}

/// Rewrites the alpha mode of every foliage material to the knob.
fn apply_foliage_alpha(
    settings: Res<BenchSettings>,
    mut standards: ResMut<Assets<StandardMaterial>>,
    mut boils: ResMut<Assets<LineBoilMaterial>>,
    mut customs: ResMut<Assets<CustomMaterial>>,
    all: Query<&MaterialSet, With<Foliage>>,
    added: Query<&MaterialSet, Added<Foliage>>,
    mut last: Local<Option<FoliageAlpha>>,
) {
    let changed = *last != Some(settings.foliage_alpha);
    *last = Some(settings.foliage_alpha);
    let sets: Vec<MaterialSet> = if changed {
        all.iter().cloned().collect()
    } else {
        added.iter().cloned().collect()
    };
    if sets.is_empty() {
        return;
    }
    let (alpha_mode, force_discard) = match settings.foliage_alpha {
        FoliageAlpha::AlphaToCoverage => (AlphaMode::AlphaToCoverage, false),
        FoliageAlpha::Blend => (AlphaMode::Blend, false),
        FoliageAlpha::Mask => (AlphaMode::Mask(0.5), false),
        FoliageAlpha::OpaqueDiscard => (AlphaMode::Opaque, true),
    };
    // Standard and boil cannot discard from an opaque pipeline: they get Mask.
    let pbr_alpha_mode = if force_discard { AlphaMode::Mask(0.5) } else { alpha_mode };
    let mut seen: HashSet<UntypedAssetId> = HashSet::new();
    for set in sets {
        if seen.insert(set.standard.id().untyped())
            && let Some(mut m) = standards.get_mut(&set.standard)
        {
            m.alpha_mode = pbr_alpha_mode;
            m.double_sided = true;
            m.cull_mode = None;
        }
        if seen.insert(set.boil.id().untyped())
            && let Some(mut m) = boils.get_mut(&set.boil)
        {
            m.alpha_mode = pbr_alpha_mode;
            m.shading.flags |= BOIL_FLAG_ALPHA_MASK;
            m.cull_mode = None;
        }
        if seen.insert(set.custom.id().untyped())
            && let Some(mut m) = customs.get_mut(&set.custom)
        {
            m.alpha_mode = alpha_mode;
            m.force_discard = force_discard;
            m.cull_mode = None;
        }
    }
}
