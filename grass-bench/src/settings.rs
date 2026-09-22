//! The knobs: one resource, an always-on egui panel, and the systems that push the
//! camera-side knobs (anti-aliasing, prepasses, occlusion culling, shadows,
//! vsync) onto the live entities. Scene-side knobs (shading, foliage alpha,
//! instancing, trees, LOD) are consumed by `shading.rs` / `scene.rs`.

use bevy::anti_alias::{
    fxaa::Fxaa,
    smaa::{Smaa, SmaaPreset},
    taa::TemporalAntiAliasing,
};
use bevy::core_pipeline::prepass::DepthPrepass;
use bevy::prelude::*;
use bevy::render::occlusion_culling::OcclusionCulling;
use bevy::window::{PresentMode, PrimaryWindow};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AaMode {
    Off,
    Msaa2,
    Msaa4,
    Fxaa,
    Smaa,
    Taa,
}

impl AaMode {
    pub const ALL: [AaMode; 6] = [
        AaMode::Off,
        AaMode::Msaa2,
        AaMode::Msaa4,
        AaMode::Fxaa,
        AaMode::Smaa,
        AaMode::Taa,
    ];
    pub fn label(self) -> &'static str {
        match self {
            AaMode::Off => "Off",
            AaMode::Msaa2 => "MSAA 2x",
            AaMode::Msaa4 => "MSAA 4x",
            AaMode::Fxaa => "FXAA",
            AaMode::Smaa => "SMAA",
            AaMode::Taa => "TAA",
        }
    }
    pub fn msaa(self) -> Msaa {
        match self {
            AaMode::Msaa2 => Msaa::Sample2,
            AaMode::Msaa4 => Msaa::Sample4,
            _ => Msaa::Off,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ShadingMode {
    /// bevy_pbr StandardMaterial straight from the glTF loader.
    Standard,
    /// The game's line-boil material as vendored (unconditional discard, one
    /// bind group per material).
    LineBoil,
    /// The bench's tightened custom material: one handle per texture, per-object
    /// parameters in a storage buffer, discard only in alpha-tested pipelines.
    Custom,
}

impl ShadingMode {
    pub const ALL: [ShadingMode; 3] =
        [ShadingMode::Standard, ShadingMode::LineBoil, ShadingMode::Custom];
    pub fn label(self) -> &'static str {
        match self {
            ShadingMode::Standard => "StandardMaterial",
            ShadingMode::LineBoil => "LineBoil (as-is)",
            ShadingMode::Custom => "Custom (fixed)",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum FoliageAlpha {
    AlphaToCoverage,
    Blend,
    Mask,
    /// Opaque pipeline (no MAY_DISCARD shader def) with a `discard` forced in
    /// the fragment shader anyway. Custom shading only.
    OpaqueDiscard,
}

impl FoliageAlpha {
    pub const ALL: [FoliageAlpha; 4] = [
        FoliageAlpha::AlphaToCoverage,
        FoliageAlpha::Blend,
        FoliageAlpha::Mask,
        FoliageAlpha::OpaqueDiscard,
    ];
    pub fn label(self) -> &'static str {
        match self {
            FoliageAlpha::AlphaToCoverage => "Alpha to coverage",
            FoliageAlpha::Blend => "Blend",
            FoliageAlpha::Mask => "Mask",
            FoliageAlpha::OpaqueDiscard => "Opaque + discard",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum InstancingMode {
    None,
    Simple,
    /// Persistent per-tier instance buffer, per-instance CPU culling, one
    /// draw per tier (the particles_and_trails buffer discipline on instances).
    SimpleCulled,
    Eidolon,
    /// No instancing: one baked mesh per chunk through the custom material,
    /// wind and affectors in its vertex shader behind a per-object flag.
    MeshChunks,
    /// Mesh chunks, but wind and affectors come from the displacement map
    /// (`displacement.rs`): one texture fetch per vertex, a small top-down
    /// render pass per frame, no affector loop in the grass shader.
    MeshChunksMap,
    /// Mesh chunks of textured sprite cards: one triangle (or quad) per plant,
    /// the vertex shader picks family, sprite and size from hashes of the
    /// root and looks the atlas corners up in a table.
    Cards,
    /// The same cards, the same meshes, a different atlas (the tinted Kenney
    /// sprites) and a fragment shader that bends the sprite inside its card
    /// with the wind: the cost of curving blades without curved geometry.
    CardsCurved,
}

impl InstancingMode {
    pub const ALL: [InstancingMode; 8] = [
        InstancingMode::None,
        InstancingMode::Simple,
        InstancingMode::SimpleCulled,
        InstancingMode::Eidolon,
        InstancingMode::MeshChunks,
        InstancingMode::MeshChunksMap,
        InstancingMode::Cards,
        InstancingMode::CardsCurved,
    ];
    pub fn label(self) -> &'static str {
        match self {
            InstancingMode::None => "No grass",
            InstancingMode::Simple => "Simple (chunked)",
            InstancingMode::SimpleCulled => "Simple, CPU-culled",
            InstancingMode::Eidolon => "bevy_eidolon",
            InstancingMode::MeshChunks => "Mesh chunks (no instancing)",
            InstancingMode::MeshChunksMap => "Mesh chunks + displacement map",
            InstancingMode::Cards => "Sprite cards (no instancing)",
            InstancingMode::CardsCurved => "Sprite cards, curved in the fragment",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum CardShape {
    /// The fitted apex-down triangle: 3 vertices, 1 triangle, no indices.
    Triangle,
    /// The tight rectangle: 4 vertices, 2 triangles.
    Quad,
}

impl CardShape {
    pub const ALL: [CardShape; 2] = [CardShape::Triangle, CardShape::Quad];
    pub fn label(self) -> &'static str {
        match self {
            CardShape::Triangle => "Triangle",
            CardShape::Quad => "Quad",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum AtlasMips {
    /// Mip 0 only.
    Off,
    /// Plain 2x2 box filter: alpha thins out with distance.
    Plain,
    /// Box filter plus per-sprite alpha scaling so the covered fraction at
    /// the cutoff matches mip 0 on every level.
    CoveragePreserving,
}

impl AtlasMips {
    pub const ALL: [AtlasMips; 3] = [AtlasMips::Off, AtlasMips::Plain, AtlasMips::CoveragePreserving];
    pub fn label(self) -> &'static str {
        match self {
            AtlasMips::Off => "Off",
            AtlasMips::Plain => "Plain box",
            AtlasMips::CoveragePreserving => "Coverage preserving",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LodMode {
    Lod0,
    Lod1,
    Distance,
}

impl LodMode {
    pub const ALL: [LodMode; 3] = [LodMode::Lod0, LodMode::Lod1, LodMode::Distance];
    pub fn label(self) -> &'static str {
        match self {
            LodMode::Lod0 => "LOD0 only",
            LodMode::Lod1 => "LOD1 only",
            LodMode::Distance => "Distance based",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum TreeModel {
    Placeholder,
    Oak,
}

impl TreeModel {
    pub const ALL: [TreeModel; 2] = [TreeModel::Placeholder, TreeModel::Oak];
    pub fn label(self) -> &'static str {
        match self {
            TreeModel::Placeholder => "Placeholder",
            TreeModel::Oak => "Fabelgeist oak",
        }
    }
}

#[derive(Resource, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BenchSettings {
    pub aa: AaMode,
    pub depth_prepass: bool,
    pub occlusion_culling: bool,
    pub shadows: bool,
    pub shading: ShadingMode,
    pub foliage_alpha: FoliageAlpha,
    pub instancing: InstancingMode,
    /// 0..1 scale on the grass tuft counts.
    pub grass_density: f32,
    /// Geometric grass ends here (m); tiers beyond are dropped, the last one
    /// fades out at this distance.
    pub grass_range: f32,
    pub grass_shadows: bool,
    /// Mesh-chunk blade width scale (vertex shader, free).
    pub grass_width: f32,
    /// Sprite cards: geometry per plant.
    pub card_shape: CardShape,
    /// Sprite cards: world scale on the sprite heights.
    pub card_scale: f32,
    /// Curved cards: how far the tip leans at full wind, as a fraction of
    /// the card's width. Past the sprite's margin in its cell (0.25) the
    /// lean walks off the sprite and the tip is clipped, not bent.
    pub card_curve: f32,
    /// Curved cards: the tip's flutter on top of the lean.
    pub card_flutter: f32,
    /// Sprite cards: atlas mip chain.
    pub atlas_mips: AtlasMips,
    /// Sprite cards: sampler anisotropy clamp (1 = off).
    pub atlas_anisotropy: u16,
    /// Displacement map resolution (texels per side over the patch).
    pub displacement_res: u32,
    /// Displacement map: accumulate a trail map so affectors leave trails of
    /// flattened grass.
    pub trample: bool,
    /// How hard a trail flattens and crushes the grass (1 = the live push).
    pub trample_strength: f32,
    /// Scorch (0..1): how much a trail kills the grass rather than bending
    /// it. It widens the flattened core of the path, presses those blades
    /// down and dries their colour out.
    pub trample_scorch: f32,
    /// How long a trail takes to grow back (s); 0 never does. The trail map
    /// relaxes toward standing grass at this rate, so a fresh path stays
    /// dark and old ones come back on their own.
    pub trample_recover: f32,
    /// Backlit foliage: how much sun scatters through a blade toward the
    /// eye. 0 is the wrapped-Lambert-only look every path had before.
    pub transmit: f32,
    /// How tight that lobe is: high is a narrow flare when you look straight
    /// at the sun through the grass, low is a broad glow.
    pub transmit_power: f32,
    /// Sun elevation and compass angle (degrees). Backlighting needs the sun
    /// low and behind the grass, which the fixed sun could never be.
    pub sun_elevation: f32,
    pub sun_azimuth: f32,
    /// Moving things that push grass aside (mesh-chunk mode: the affector
    /// uniform array; instanced modes: the first one drives the game's single
    /// interaction slot).
    pub affector_count: u32,
    pub tree_model: TreeModel,
    pub tree_count: u32,
    pub character_count: u32,
    pub lod: LodMode,
    pub orbit: bool,
    pub vsync: bool,
}

impl Default for BenchSettings {
    fn default() -> Self {
        Self {
            aa: AaMode::Msaa4,
            depth_prepass: false,
            occlusion_culling: false,
            shadows: true,
            shading: ShadingMode::Standard,
            foliage_alpha: FoliageAlpha::AlphaToCoverage,
            instancing: InstancingMode::None,
            grass_density: 0.35,
            grass_range: 30.0,
            grass_shadows: false,
            grass_width: 1.6,
            card_shape: CardShape::Triangle,
            card_scale: 1.0,
            card_curve: 0.15,
            card_flutter: 0.35,
            atlas_mips: AtlasMips::CoveragePreserving,
            atlas_anisotropy: 1,
            displacement_res: 512,
            trample: true,
            trample_strength: 3.0,
            trample_scorch: 0.5,
            trample_recover: 30.0,
            transmit: 0.8,
            transmit_power: 4.0,
            sun_elevation: 52.0,
            sun_azimuth: 58.0,
            affector_count: 2,
            tree_model: TreeModel::Placeholder,
            tree_count: 20,
            character_count: 0,
            lod: LodMode::Distance,
            orbit: true,
            vsync: false,
        }
    }
}

impl BenchSettings {
    /// TAA needs the depth prepass whatever the knob says.
    pub fn effective_depth_prepass(&self) -> bool {
        self.depth_prepass || self.aa == AaMode::Taa
    }
    /// Occlusion culling is a depth-prepass feature.
    pub fn effective_occlusion_culling(&self) -> bool {
        self.occlusion_culling && self.effective_depth_prepass()
    }
    /// Alpha-to-coverage needs MSAA; bevy quietly runs it as Mask otherwise.
    pub fn a2c_available(&self) -> bool {
        matches!(self.aa, AaMode::Msaa2 | AaMode::Msaa4)
    }

    /// One line for the CSV and the overlay.
    pub fn summary(&self) -> String {
        format!(
            "aa={} prepass={} occl={} shadows={} shading={} foliage={} grass={} density={:.2} range={:.0} affectors={} grass_shadows={} trees={}x{} chars={} lod={}",
            self.aa.label(),
            self.effective_depth_prepass(),
            self.effective_occlusion_culling(),
            self.shadows,
            self.shading.label(),
            self.foliage_alpha.label(),
            self.instancing.label(),
            self.grass_density,
            self.grass_range,
            self.affector_count,
            self.grass_shadows,
            self.tree_count,
            self.tree_model.label(),
            self.character_count,
            self.lod.label(),
        )
    }

    /// Native: `BENCH_SETTINGS` env (JSON, partial is fine) over `bench.json`
    /// in the working directory. Web: the `s` URL query parameter (JSON).
    pub fn load() -> Self {
        #[cfg(not(target_family = "wasm"))]
        {
            if let Ok(json) = std::env::var("BENCH_SETTINGS") {
                match serde_json::from_str(&json) {
                    Ok(settings) => return settings,
                    Err(err) => eprintln!("BENCH_SETTINGS ignored: {err}"),
                }
            }
            if let Ok(json) = std::fs::read_to_string("bench.json") {
                match serde_json::from_str(&json) {
                    Ok(settings) => return settings,
                    Err(err) => eprintln!("bench.json ignored: {err}"),
                }
            }
        }
        #[cfg(target_family = "wasm")]
        {
            if let Some(json) = web_sys::window()
                .and_then(|w| w.location().search().ok())
                .and_then(|search| {
                    search
                        .trim_start_matches('?')
                        .split('&')
                        .find_map(|pair| pair.strip_prefix("s=").map(str::to_string))
                })
                .and_then(|encoded| js_sys::decode_uri_component(&encoded).ok())
                .map(|decoded| String::from(decoded))
            {
                match serde_json::from_str(&json) {
                    Ok(settings) => return settings,
                    Err(err) => warn!("?s= settings ignored: {err}"),
                }
            }
        }
        Self::default()
    }

    fn save(&self) {
        #[cfg(not(target_family = "wasm"))]
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("bench.json", json);
        }
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, settings_panel)
            .add_systems(Update, apply_camera_settings);
    }
}

fn combo<T: Copy + PartialEq>(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut T,
    all: &[T],
    name: impl Fn(T) -> &'static str,
) -> bool {
    let mut changed = false;
    egui::ComboBox::from_label(label)
        .selected_text(name(*value))
        .show_ui(ui, |ui| {
            for option in all {
                changed |= ui
                    .selectable_value(value, *option, name(*option))
                    .changed();
            }
        });
    changed
}

/// Always on: the panel is the bench's control surface, never hidden.
fn settings_panel(mut contexts: EguiContexts, mut settings: ResMut<BenchSettings>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let s = settings.bypass_change_detection();
    let mut changed = false;
    egui::Window::new("GPU bench")
        .default_pos((10.0, 200.0))
        .vscroll(true)
        .show(ctx, |ui| {
            ui.heading("Camera");
            changed |= combo(ui, "Anti-aliasing", &mut s.aa, &AaMode::ALL, AaMode::label);
            if s.aa == AaMode::Taa {
                ui.add_enabled(false, egui::Checkbox::new(&mut true, "Depth pre-pass"))
                    .on_disabled_hover_text("TAA requires depth prepass");
                ui.label(egui::RichText::new("TAA requires depth prepass").weak());
            } else {
                changed |= ui.checkbox(&mut s.depth_prepass, "Depth pre-pass").changed();
            }
            ui.add_enabled_ui(s.effective_depth_prepass(), |ui| {
                changed |= ui
                    .checkbox(&mut s.occlusion_culling, "GPU occlusion culling")
                    .on_disabled_hover_text("Needs the depth pre-pass")
                    .changed();
            });
            changed |= ui.checkbox(&mut s.shadows, "Sun shadows").changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.sun_elevation, 2.0..=85.0).text("Sun elevation"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.sun_azimuth, 0.0..=360.0).text("Sun compass"))
                .changed();
            #[cfg(not(target_family = "wasm"))]
            {
                changed |= ui.checkbox(&mut s.vsync, "VSync").changed();
            }
            changed |= ui.checkbox(&mut s.orbit, "Orbit camera").changed();
            ui.label(egui::RichText::new("Orbit off: right-drag looks, WASD/QE moves").weak());

            ui.separator();
            ui.heading("Materials");
            changed |= combo(ui, "Shading", &mut s.shading, &ShadingMode::ALL, ShadingMode::label);
            changed |= combo(
                ui,
                "Foliage alpha",
                &mut s.foliage_alpha,
                &FoliageAlpha::ALL,
                FoliageAlpha::label,
            );
            if s.foliage_alpha == FoliageAlpha::AlphaToCoverage && !s.a2c_available() {
                ui.label(egui::RichText::new("A2C needs MSAA; bevy runs it as Mask").weak());
            }
            if s.foliage_alpha == FoliageAlpha::OpaqueDiscard && s.shading != ShadingMode::Custom {
                ui.label(egui::RichText::new("Opaque + discard only exists in the custom shader").weak());
            }

            ui.separator();
            ui.heading("Grass");
            changed |= combo(
                ui,
                "Instancing",
                &mut s.instancing,
                &InstancingMode::ALL,
                InstancingMode::label,
            );
            changed |= ui
                .add(egui::Slider::new(&mut s.grass_density, 0.05..=1.0).text("Density"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.grass_range, 8.0..=72.0).text("Range (m)"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.affector_count, 0..=16).text("Affectors"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.grass_width, 0.5..=4.0).text("Blade width (mesh chunks)"))
                .changed();
            changed |= ui.checkbox(&mut s.grass_shadows, "Grass casts shadows").changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.transmit, 0.0..=2.0).text("Backlit glow"))
                .changed();
            changed |= ui
                .add(egui::Slider::new(&mut s.transmit_power, 1.0..=16.0).text("Glow sharpness"))
                .changed();
            if s.instancing == InstancingMode::MeshChunks {
                ui.label(egui::RichText::new("Mesh chunks always use the custom material").weak());
            }
            if s.instancing == InstancingMode::MeshChunksMap {
                ui.label(egui::RichText::new("Wind + affectors rendered top-down once per frame, one fetch per vertex").weak());
                changed |= combo(
                    ui,
                    "Map resolution",
                    &mut s.displacement_res,
                    &[128, 256, 512, 1024],
                    |r| match r {
                        128 => "128",
                        256 => "256",
                        512 => "512",
                        _ => "1024",
                    },
                );
                changed |= ui.checkbox(&mut s.trample, "Trails (accumulated trail map)").changed();
                changed |= ui
                    .add(egui::Slider::new(&mut s.trample_strength, 0.5..=6.0).text("Trail strength"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut s.trample_scorch, 0.0..=1.0).text("Scorch"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut s.trample_recover, 0.0..=180.0)
                            .text("Regrow (s, 0 = never)"),
                    )
                    .changed();
            }
            if matches!(s.instancing, InstancingMode::Cards | InstancingMode::CardsCurved) {
                ui.label(egui::RichText::new("Sprite cards: custom material, foliage alpha knob applies").weak());
                if s.instancing == InstancingMode::CardsCurved {
                    ui.label(
                        egui::RichText::new(
                            "Curved: same meshes, the fragment bends the sprite inside its card",
                        )
                        .weak(),
                    );
                    changed |= ui
                        .add(egui::Slider::new(&mut s.card_curve, 0.0..=0.3).text("Curve (tip lean)"))
                        .changed();
                    changed |= ui
                        .add(egui::Slider::new(&mut s.card_flutter, 0.0..=1.0).text("Flutter"))
                        .changed();
                } else {
                    changed |= combo(ui, "Card shape", &mut s.card_shape, &CardShape::ALL, CardShape::label);
                }
                changed |= ui
                    .add(egui::Slider::new(&mut s.card_scale, 0.5..=2.0).text("Card scale"))
                    .changed();
                changed |= combo(ui, "Atlas mips", &mut s.atlas_mips, &AtlasMips::ALL, AtlasMips::label);
                let mut aniso = s.atlas_anisotropy.max(1).ilog2();
                if ui
                    .add(egui::Slider::new(&mut aniso, 0..=4).text("Anisotropy (log2)"))
                    .changed()
                {
                    s.atlas_anisotropy = 1 << aniso;
                    changed = true;
                }
                ui.label(egui::RichText::new(format!("anisotropy {}x", s.atlas_anisotropy)).weak());
            }

            ui.separator();
            ui.heading("Trees");
            changed |= combo(ui, "Model", &mut s.tree_model, &TreeModel::ALL, TreeModel::label);
            changed |= ui
                .add(egui::Slider::new(&mut s.tree_count, 0..=200).text("Count"))
                .changed();
            changed |= combo(ui, "LOD", &mut s.lod, &LodMode::ALL, LodMode::label);

            ui.separator();
            ui.heading("Characters");
            changed |= ui
                .add(egui::Slider::new(&mut s.character_count, 0..=64).text("Count"))
                .changed();

            ui.separator();
            if ui.button("Reset to defaults").clicked() {
                *s = BenchSettings::default();
                changed = true;
            }
            if ui.button("Log settings JSON").clicked() {
                info!("{}", serde_json::to_string(&*s).unwrap_or_default());
            }
        });
    if changed {
        settings.save();
        settings.set_changed();
    }
}

fn apply_camera_settings(
    mut commands: Commands,
    settings: Res<BenchSettings>,
    cameras: Query<(Entity, &Transform, &crate::scene::BenchCamera), With<Camera3d>>,
    mut suns: Query<&mut DirectionalLight>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut applied: Local<Option<(AaMode, bool, bool)>>,
) {
    if !settings.is_changed() {
        return;
    }
    let camera_key = (
        settings.aa,
        settings.effective_depth_prepass(),
        settings.effective_occlusion_culling(),
    );
    let camera_changed = *applied != Some(camera_key);
    *applied = Some(camera_key);
    for (entity, transform, state) in &cameras {
        if !camera_changed {
            continue;
        }
        // A fresh entity, not a mutation: see `scene::camera_bundle`.
        commands.entity(entity).despawn();
        let mut cam = commands.spawn(crate::scene::camera_bundle(state.clone(), *transform));
        cam.insert(settings.aa.msaa());
        match settings.aa {
            AaMode::Fxaa => {
                cam.insert(Fxaa::default());
            }
            AaMode::Smaa => {
                cam.insert(Smaa {
                    preset: SmaaPreset::High,
                });
            }
            AaMode::Taa => {
                cam.insert(TemporalAntiAliasing::default());
            }
            _ => {}
        }
        if settings.effective_depth_prepass() {
            cam.insert(DepthPrepass);
        }
        if settings.effective_occlusion_culling() {
            cam.insert(OcclusionCulling);
        }
    }
    for mut sun in &mut suns {
        if sun.shadow_maps_enabled != settings.shadows {
            sun.shadow_maps_enabled = settings.shadows;
        }
    }
    for mut window in &mut windows {
        let wanted = if settings.vsync {
            PresentMode::AutoVsync
        } else {
            PresentMode::AutoNoVsync
        };
        if window.present_mode != wanted {
            window.present_mode = wanted;
        }
    }
}
