use std::path::PathBuf;

use adventuresim_armor_model::{
    BracerDesign, BreastplateDesign, GeneratedArmor, generate_bracer, generate_breastplate,
};
use adventuresim_character_creator::{
    CharacterRecipe, ClothingSelection, EXPRESSION_COUNT, IDENTITY_COUNT, IdentityGroup,
    bracer::{ForearmMorphSample, ForearmSide, ForearmSurfaceInput, build_forearm_surface},
    breastplate::{TorsoSurfaceInput, build_front_torso_surface},
    clothing::{GarmentSpecification, generate_clothing_shells},
    export::{
        MHR_ANATOMICAL_UV_DOMAIN, RiggedMesh, RiggedMorphTarget, RiggedShell, RiggedSocket,
        SurfaceUvLayout, export_rigged_glb, fitted_equipment_socket_from_uv,
    },
    item_catalog_schema::{EquipmentLocation, ItemCatalogDocument, ItemDefinition},
};
use anyhow::{Context, Result};
use bevy::{
    asset::RenderAssetUsages,
    input::mouse::{MouseMotion, MouseWheel},
    mesh::Indices,
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use burn::tensor::{Device, Tensor, TensorData};
use clap::Parser;
use fabelgeist_mhr::{Mhr, MhrConfig, NUM_FACE_EXPRESSION_BLEND_SHAPES};
use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Parser, Resource, Clone)]
#[command(about = "Fabelgeist's MHR character design studio")]
struct Args {
    #[arg(
        long,
        env = "MHR_ASSETS",
        default_value = "target/mhr-assets/v1.0.1/assets"
    )]
    assets: PathBuf,
    // LOD 1 retains enough facial, ear, and finger topology for close creator
    // views while remaining inexpensive with pose correctives disabled.
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..=6))]
    lod: u8,
    #[arg(long, default_value = "assets_src/characters/john_fabelgeist.json")]
    recipe: PathBuf,
    #[arg(long, default_value = "assets_src/biped/unarmed/base.glb")]
    glb: PathBuf,
    #[arg(long, default_value = "content/items")]
    catalog: PathBuf,
    #[arg(long, default_value = "assets/equipment/procedural")]
    equipment_output: PathBuf,
    /// Export the selected recipe without opening the studio window.
    #[arg(long)]
    export_only: bool,
    /// Generate one procedural MHR asset for every armor/clothing placement.
    #[arg(long)]
    generate_equipment: bool,
}

#[derive(Resource)]
struct BodyModel {
    mhr: Mhr,
    lod: u8,
    correctives: bool,
}

#[derive(Resource)]
struct EquipmentCatalog(Vec<ItemDefinition>);

#[derive(Resource)]
struct Studio {
    recipe: CharacterRecipe,
    selected: IdentityGroup,
    show_expressions: bool,
    dirty: bool,
    status: String,
    recipe_path: String,
    glb_path: String,
    seed: u64,
    selected_lod: u8,
    selected_correctives: bool,
    bracer_design: BracerDesign,
    breastplate_design: BreastplateDesign,
}

#[derive(Component)]
struct CharacterMesh;

struct GeneratedCharacter {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    global_joint_states: Vec<[f32; 8]>,
}

#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    radius: f32,
    focus: Vec3,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let device = Device::default();
    let model = load_body_model(&args.assets, args.lod, false, &device)
        .with_context(|| format!("loading MHR assets from {}", args.assets.display()))?;
    let catalog = EquipmentCatalog(load_item_catalog(&args.catalog)?);

    let recipe = if args.recipe.is_file() {
        let parsed: CharacterRecipe = serde_json::from_slice(&std::fs::read(&args.recipe)?)?;
        parsed.validate().map_err(anyhow::Error::msg)?;
        parsed
    } else {
        CharacterRecipe {
            name: "John Fabelgeist".into(),
            ..CharacterRecipe::default()
        }
    };

    if args.generate_equipment {
        generate_equipment_assets(&args.equipment_output, &model, &recipe, &catalog)?;
        println!(
            "Generated equipment under {}",
            args.equipment_output.display()
        );
        return Ok(());
    }
    if args.export_only {
        export_character(
            &args.glb,
            &model,
            &recipe,
            &catalog,
            &BracerDesign::default(),
            &BreastplateDesign::default(),
        )?;
        println!("Exported {}", args.glb.display());
        return Ok(());
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.055)))
        .insert_resource(args.clone())
        .insert_resource(model)
        .insert_resource(catalog)
        .insert_resource(Studio {
            recipe,
            selected: IdentityGroup::Body,
            show_expressions: false,
            dirty: true,
            status: format!("MHR LOD {} ready", args.lod),
            recipe_path: args.recipe.display().to_string(),
            glb_path: args.glb.display().to_string(),
            seed: 1544,
            selected_lod: args.lod,
            selected_correctives: false,
            bracer_design: BracerDesign::default(),
            breastplate_design: BreastplateDesign::default(),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fabelgeist · Character Studio".into(),
                resolution: (1440, 900).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, studio_ui)
        .add_systems(
            Update,
            (
                reload_model.before(regenerate_mesh),
                regenerate_mesh,
                orbit_camera,
            ),
        )
        .run();
    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        // A restrained ambient term stands in for indirect room bounce. It
        // prevents fully black occlusion without flattening the spotlight's
        // form and floor shadow.
        AmbientLight {
            color: Color::srgb(0.78, 0.84, 0.94),
            brightness: 155.0,
            ..default()
        },
        Transform::default(),
        OrbitCamera {
            yaw: 0.1,
            pitch: -0.05,
            radius: 2.7,
            focus: Vec3::new(0.0, 1.0, 0.0),
        },
    ));

    let light_position = Vec3::new(-2.4, 4.2, 3.0);
    commands.spawn((
        SpotLight {
            color: Color::srgb(1.0, 0.90, 0.79),
            intensity: 1_050_000.0,
            range: 8.0,
            radius: 0.38,
            inner_angle: 0.30,
            outer_angle: 0.58,
            shadow_maps_enabled: true,
            shadow_depth_bias: 0.025,
            shadow_normal_bias: 1.9,
            shadow_map_near_z: 0.1,
            ..default()
        },
        Transform::from_translation(light_position).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(12.0, 12.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.17, 0.18),
            perceptual_roughness: 0.86,
            reflectance: 0.12,
            ..default()
        })),
    ));
}

#[expect(
    deprecated,
    reason = "egui's replacement requires a parent Ui, but this is the top-level panel"
)]
fn studio_ui(
    mut contexts: EguiContexts,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::SidePanel::left("creator")
        .exact_width(360.0)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.label("Name");
            ui.text_edit_singleline(&mut studio.recipe.name);
            let lod_changed = ui
                .add(
                    egui::Slider::new(&mut studio.selected_lod, 0..=6)
                        .text("Mesh LOD")
                        .custom_formatter(|value, _| {
                            let lod = value.round() as usize;
                            let vertices = [73_639, 18_439, 10_661, 4_899, 2_461, 971, 595][lod];
                            format!("{lod} · {vertices} vertices")
                        }),
                )
                .changed();
            if lod_changed {
                studio.status = format!("Loading MHR LOD {}…", studio.selected_lod);
            }
            ui.small("LOD 0 is highest fidelity; LOD 6 is lowest.");
            if ui
                .checkbox(&mut studio.selected_correctives, "Pose-corrective model")
                .changed()
            {
                studio.status = format!(
                    "Loading MHR LOD {} with correctives {}…",
                    studio.selected_lod,
                    if studio.selected_correctives {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );
            }
            ui.small(
                "Correctives improve posed deformation but require substantially more memory.",
            );
            ui.separator();

            ui.collapsing("Catalog clothing and armor", |ui| {
                for item in procedural_items(&catalog) {
                    let equipment = item.equipment.as_ref().expect("filtered equipment");
                    for placement in &equipment.placements {
                        let selection = ClothingSelection {
                            item_id: item.id.clone(),
                            placement_id: placement.id.clone(),
                        };
                        let mut enabled = studio.recipe.clothing.contains(&selection);
                        let label = if equipment.placements.len() == 1 {
                            item.display_name.clone()
                        } else {
                            format!("{} · {}", item.display_name, placement.id)
                        };
                        if ui.checkbox(&mut enabled, label).changed() {
                            if enabled {
                                studio.recipe.clothing.push(selection.clone());
                            } else {
                                studio
                                    .recipe
                                    .clothing
                                    .retain(|selected| selected != &selection);
                            }
                            studio.dirty = true;
                        }
                    }
                }
            });
            ui.small("Bone-weight shells follow the generated body and share its MHR skin.");
            ui.collapsing("Parametric bracers", |ui| {
                let mut changed = false;
                changed |= ui
                    .add(
                        egui::Slider::new(&mut studio.bracer_design.coverage.0, 50..=1_000)
                            .text("Forearm coverage")
                            .suffix(" ‰"),
                    )
                    .changed();
                let maximum_offset = 1_000_u16 - studio.bracer_design.coverage.0;
                if studio.bracer_design.wrist_offset.0 > maximum_offset {
                    studio.bracer_design.wrist_offset.0 = maximum_offset;
                    changed = true;
                }
                changed |= ui
                    .add(
                        egui::Slider::new(
                            &mut studio.bracer_design.wrist_offset.0,
                            0..=maximum_offset,
                        )
                        .text("Wrist offset")
                        .suffix(" ‰"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut studio.bracer_design.wall_thickness.0, 1..=20)
                            .text("Wall thickness")
                            .suffix(" mm"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut studio.bracer_design.clearance.0, 1..=30)
                            .text("Body clearance")
                            .suffix(" mm"),
                    )
                    .changed();
                ui.horizontal(|ui| {
                    if ui.button("Bracelet").clicked() {
                        studio.bracer_design = BracerDesign::bracelet();
                        changed = true;
                    }
                    if ui.button("Vambrace").clicked() {
                        studio.bracer_design = BracerDesign::default();
                        changed = true;
                    }
                    if ui.button("Full forearm").clicked() {
                        studio.bracer_design = BracerDesign::full_forearm();
                        changed = true;
                    }
                });
                ui.small("Enable either Vambrace catalog placement above to preview it.");
                studio.dirty |= changed;
            });
            ui.collapsing("Parametric front breastplate", |ui| {
                let design = &mut studio.breastplate_design;
                let mut changed = false;
                changed |= ui
                    .add(egui::Slider::new(&mut design.neck_width.0, 200..=700).text("Neck width"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut design.neck_depth.0, 0..=500).text("Neck depth"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.arm_opening_depth.0, 100..=600)
                            .text("Arm opening depth"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.waist_width.0, 550..=1_000)
                            .text("Waist width"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.stomach_height.0, 0..=350)
                            .text("Stomach edge height"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.rigidity.0, 0..=1_000).text("Plate rigidity"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut design.wrap.0, 0..=1_000).text("Torso wrap"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.crown.0, 0..=80)
                            .text("Rounded crown")
                            .suffix(" mm"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.skirt_length.0, 40..=250)
                            .text("Skirt length")
                            .suffix(" ‰"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.skirt_flare.0, 0..=120)
                            .text("Skirt flare")
                            .suffix(" mm"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.wall_thickness.0, 1..=20)
                            .text("Wall thickness")
                            .suffix(" mm"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut design.clearance.0, 1..=30)
                            .text("Body clearance")
                            .suffix(" mm"),
                    )
                    .changed();
                if ui.button("Reset breastplate").clicked() {
                    *design = BreastplateDesign::default();
                    changed = true;
                }
                ui.small("Enable Breastplate · worn above to preview it.");
                studio.dirty |= changed;
            });
            ui.separator();

            ui.horizontal(|ui| {
                for group in IdentityGroup::ALL {
                    ui.selectable_value(&mut studio.selected, group, group.label());
                }
            });
            let selected = studio.selected;
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for index in selected.range() {
                        let response = ui.add(
                            egui::Slider::new(&mut studio.recipe.identity[index], -3.0..=3.0)
                                .text(format!(
                                    "{} {:02}",
                                    selected.label(),
                                    index - selected.range().start + 1
                                ))
                                .fixed_decimals(2),
                        );
                        studio.dirty |= response.changed();
                    }
                });

            ui.collapsing("Expression laboratory", |ui| {
                ui.checkbox(
                    &mut studio.show_expressions,
                    "Show all 72 expression channels",
                );
                if studio.show_expressions {
                    egui::ScrollArea::vertical()
                        .max_height(180.0)
                        .show(ui, |ui| {
                            let mut changed = false;
                            for (index, value) in studio.recipe.expression.iter_mut().enumerate() {
                                changed |= ui
                                    .add(
                                        egui::Slider::new(value, -1.0..=1.0)
                                            .text(format!("Expression {:02}", index + 1)),
                                    )
                                    .changed();
                            }
                            studio.dirty |= changed;
                        });
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Randomize body").clicked() {
                    studio.seed = studio.seed.wrapping_add(1);
                    let mut rng = StdRng::seed_from_u64(studio.seed);
                    for value in &mut studio.recipe.identity {
                        *value = rng.random_range(-1.35..=1.35);
                    }
                    studio.dirty = true;
                }
                if ui.button("Neutral").clicked() {
                    studio.recipe.reset_body();
                    studio.recipe.reset_face();
                    studio.dirty = true;
                }
            });
            ui.add(egui::TextEdit::singleline(&mut studio.recipe_path).hint_text("character.json"));
            ui.horizontal(|ui| {
                if ui.button("Save recipe").clicked() {
                    studio.status = save_recipe(&studio)
                        .unwrap_or_else(|error| format!("Save failed: {error:#}"));
                }
                if ui.button("Load recipe").clicked() {
                    match load_recipe(&studio.recipe_path) {
                        Ok(recipe) => {
                            studio.recipe = recipe;
                            studio.dirty = true;
                            studio.status = "Recipe loaded".into();
                        }
                        Err(error) => studio.status = format!("Load failed: {error:#}"),
                    }
                }
            });
            ui.add(
                egui::TextEdit::singleline(&mut studio.glb_path)
                    .hint_text("assets_src/biped/unarmed/base.glb"),
            );
            if ui.button("Export rigged GLB").clicked() {
                studio.status = export_character(
                    std::path::Path::new(&studio.glb_path),
                    &model,
                    &studio.recipe,
                    &catalog,
                    &studio.bracer_design,
                    &studio.breastplate_design,
                )
                .map(|()| format!("Exported {}", studio.glb_path))
                .unwrap_or_else(|error| format!("Export failed: {error:#}"));
            }
            ui.add_space(6.0);
            ui.small(&studio.status);
            ui.small("Drag to orbit · wheel to zoom");
        });
}

fn save_recipe(studio: &Studio) -> Result<String> {
    studio.recipe.validate().map_err(anyhow::Error::msg)?;
    let path = std::path::Path::new(&studio.recipe_path);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_vec_pretty(&studio.recipe)?)?;
    Ok(format!("Saved {}", studio.recipe_path))
}

fn load_recipe(path: &str) -> Result<CharacterRecipe> {
    let recipe: CharacterRecipe = serde_json::from_slice(&std::fs::read(path)?)?;
    recipe.validate().map_err(anyhow::Error::msg)?;
    Ok(recipe)
}

fn load_item_catalog(directory: &std::path::Path) -> Result<Vec<ItemDefinition>> {
    let mut files = std::fs::read_dir(directory)
        .with_context(|| format!("reading item catalog directory {}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    files.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "yaml")
    });
    files.sort();
    let mut items = Vec::new();
    for path in files {
        let document: ItemCatalogDocument = serde_json::from_slice(&std::fs::read(&path)?)
            .with_context(|| format!("parsing item catalog {}", path.display()))?;
        items.extend(document.items);
    }
    if items.is_empty() {
        anyhow::bail!("item catalog contains no definitions");
    }
    Ok(items)
}

fn procedural_items(catalog: &EquipmentCatalog) -> impl Iterator<Item = &ItemDefinition> {
    catalog.0.iter().filter(|item| {
        item.equipment.as_ref().is_some_and(|equipment| {
            equipment.material.is_some()
                && equipment
                    .placements
                    .iter()
                    .any(|placement| !placement.surface.is_empty())
        })
    })
}

fn selected_garments(
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
) -> Result<Vec<GarmentSpecification>, String> {
    recipe
        .clothing
        .iter()
        .filter(|selection| !matches!(selection.item_id.as_str(), "vambrace" | "breastplate"))
        .map(|selection| {
            let item = procedural_items(catalog)
                .find(|item| item.id == selection.item_id)
                .ok_or_else(|| format!("unknown procedural item {}", selection.item_id))?;
            let placement = item
                .equipment
                .as_ref()
                .and_then(|equipment| {
                    equipment
                        .placements
                        .iter()
                        .find(|placement| placement.id == selection.placement_id)
                })
                .ok_or_else(|| {
                    format!(
                        "item {} has no placement {}",
                        selection.item_id, selection.placement_id
                    )
                })?;
            Ok(GarmentSpecification::from_catalog(
                format!("{} · {}", item.display_name, placement.id),
                placement,
                item.equipment
                    .as_ref()
                    .and_then(|equipment| equipment.material)
                    .ok_or_else(|| format!("item {} has no procedural material", item.id))?,
            ))
        })
        .collect()
}

fn selected_vambrace_sides(recipe: &CharacterRecipe) -> Vec<ForearmSide> {
    recipe
        .clothing
        .iter()
        .filter(|selection| selection.item_id == "vambrace")
        .filter_map(|selection| match selection.placement_id.as_str() {
            "left" => Some(ForearmSide::Left),
            "right" => Some(ForearmSide::Right),
            _ => None,
        })
        .collect()
}

fn breastplate_selected(recipe: &CharacterRecipe) -> bool {
    recipe
        .clothing
        .iter()
        .any(|selection| selection.item_id == "breastplate" && selection.placement_id == "worn")
}

fn fitted_bracer(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &BracerDesign,
    side: ForearmSide,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let surface = build_forearm_surface(ForearmSurfaceInput {
        domain: MHR_ANATOMICAL_UV_DOMAIN,
        side,
        positions: &generated.positions,
        normals: &generated.normals,
        faces: &character.mesh.faces,
        texcoords: &character.mesh.texcoords,
        texcoord_faces: &character.mesh.texcoord_faces,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
        global_joint_states: &generated.global_joint_states,
        morphs,
    })
    .map_err(anyhow::Error::msg)?;
    generate_bracer(design, &surface).map_err(anyhow::Error::new)
}

fn fitted_breastplate(
    model: &BodyModel,
    generated: &GeneratedCharacter,
    design: &BreastplateDesign,
    morphs: &[ForearmMorphSample],
) -> Result<GeneratedArmor> {
    let character = &model.mhr.character;
    let surface = build_front_torso_surface(TorsoSurfaceInput {
        domain: MHR_ANATOMICAL_UV_DOMAIN,
        positions: &generated.positions,
        normals: &generated.normals,
        faces: &character.mesh.faces,
        texcoords: &character.mesh.texcoords,
        texcoord_faces: &character.mesh.texcoord_faces,
        joint_indices: &character.skin_weights.index,
        joint_weights: &character.skin_weights.weight,
        joint_names: &character.skeleton.names,
        global_joint_states: &generated.global_joint_states,
        morphs,
    })
    .map_err(anyhow::Error::msg)?;
    generate_breastplate(design, &surface).map_err(anyhow::Error::new)
}

fn placement_coverage(
    placement: &adventuresim_character_creator::item_catalog_schema::EquipmentPlacement,
) -> f32 {
    let region_count = placement
        .surface
        .iter()
        .map(|span| span.regions.len())
        .sum::<usize>();
    placement
        .surface
        .iter()
        .map(|span| span.coverage * span.regions.len() as f32)
        .sum::<f32>()
        / region_count as f32
}

fn belt_mount_outward(location: EquipmentLocation) -> Option<[f64; 3]> {
    Some(match location {
        // MHR uses positive model-space X for the character's anatomical left.
        EquipmentLocation::LeftBelt => [1.0, 0.0, 0.0],
        EquipmentLocation::RightBelt => [-1.0, 0.0, 0.0],
        EquipmentLocation::FrontBelt => [0.0, 0.0, -1.0],
        EquipmentLocation::BackBelt => [0.0, 0.0, 1.0],
        _ => return None,
    })
}

fn signed_shape_zero_endpoint_recipe(source: &CharacterRecipe, sign: f32) -> CharacterRecipe {
    CharacterRecipe {
        version: source.version,
        name: source.name.clone(),
        identity: {
            let mut identity = vec![0.0; IDENTITY_COUNT];
            identity[0] = sign;
            identity
        },
        expression: vec![0.0; EXPRESSION_COUNT],
        clothing: Vec::new(),
    }
}
fn generate_equipment_assets(
    output: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
) -> Result<()> {
    std::fs::create_dir_all(output)
        .with_context(|| format!("creating equipment output {}", output.display()))?;
    let generated = generate_character(model, recipe)?;
    let character = &model.mhr.character;
    let bracer_morphs = generate_morph_samples(model, recipe)?;
    let mut assets = Vec::new();
    let mut generated_files = std::collections::BTreeSet::new();
    for item in procedural_items(catalog) {
        let equipment = item.equipment.as_ref().expect("filtered equipment");
        for placement in &equipment.placements {
            if placement.surface.is_empty() {
                continue;
            }
            if matches!(item.id.as_str(), "vambrace" | "breastplate") {
                let (armor, parametric_coverage) = if item.id == "vambrace" {
                    let side = match placement.id.as_str() {
                        "left" => ForearmSide::Left,
                        "right" => ForearmSide::Right,
                        _ => {
                            anyhow::bail!("vambrace placement {} has no forearm side", placement.id)
                        }
                    };
                    let design = BracerDesign::default();
                    (
                        fitted_bracer(model, &generated, &design, side, &bracer_morphs)?,
                        design.coverage.unit(),
                    )
                } else {
                    let design = BreastplateDesign::default();
                    (
                        fitted_breastplate(model, &generated, &design, &bracer_morphs)?,
                        placement_coverage(placement),
                    )
                };
                let armor = center_signed_armor_morphs(armor)?;
                let faces = armor.indices.as_chunks::<3>().0.to_vec();
                let morph_targets = armor
                    .morphs
                    .iter()
                    .map(|target| RiggedMorphTarget {
                        name: &target.name,
                        position_deltas: &target.position_deltas,
                        normal_deltas: &target.normal_deltas,
                    })
                    .collect::<Vec<_>>();
                let file_name = format!("{}--{}.glb", item.id, placement.id);
                let path = output.join(&file_name);
                let rigged_shell = RiggedShell {
                    name: &item.display_name,
                    positions: &armor.positions,
                    normals: &armor.normals,
                    faces: &faces,
                    joint_indices: Some(&armor.joint_indices),
                    joint_weights: Some(&armor.joint_weights),
                    morph_targets: &morph_targets,
                    base_color: [0.769, 0.776, 0.776, 1.0],
                    metallic: 1.0,
                    roughness: 0.20,
                };
                export_rigged_glb(
                    &path,
                    &item.id,
                    recipe.version,
                    model.lod,
                    &RiggedMesh {
                        positions: &generated.positions,
                        normals: &generated.normals,
                        faces: &character.mesh.faces,
                        export_body: false,
                        joint_indices: &character.skin_weights.index,
                        joint_weights: &character.skin_weights.weight,
                        joint_names: &character.skeleton.names,
                        joint_parents: &character.skeleton.parents,
                        global_joint_states: &generated.global_joint_states,
                    },
                    &[rigged_shell],
                    &[],
                )?;
                generated_files.insert(file_name.clone());
                assets.push(serde_json::json!({
                    "item_id": item.id,
                    "placement_id": placement.id,
                    "file": file_name,
                    "coverage": parametric_coverage,
                    "material": equipment.material,
                    "triangles": faces.len(),
                    "armor_generator_version": adventuresim_armor_model::GENERATOR_VERSION,
                    "armor_design_hash": armor.design_hash.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                    "morph_targets": armor.morphs.len(),
                    "surface_uv_domain": armor.surface_domain,
                }));
                continue;
            }
            let specification = GarmentSpecification::from_catalog(
                format!("{} · {}", item.display_name, placement.id),
                placement,
                equipment.material.ok_or_else(|| {
                    anyhow::anyhow!("item {} has no procedural material", item.id)
                })?,
            );
            let clothed = generate_clothing_shells(
                &[specification],
                &generated.positions,
                &generated.normals,
                &character.mesh.faces,
                &character.skin_weights.index,
                &character.skin_weights.weight,
                &character.skeleton.names,
                &generated.global_joint_states,
            )
            .map_err(anyhow::Error::msg)?;
            let shell = &clothed.shells[0];
            let file_name = format!("{}--{}.glb", item.id, placement.id);
            let path = output.join(&file_name);
            let rigged_shell = RiggedShell {
                name: &shell.specification.name,
                positions: &shell.positions,
                normals: &shell.normals,
                faces: &shell.faces,
                joint_indices: None,
                joint_weights: None,
                morph_targets: &[],
                base_color: shell.specification.base_color,
                metallic: shell.specification.metallic,
                roughness: shell.specification.roughness,
            };
            let rigged_mesh = RiggedMesh {
                positions: &generated.positions,
                normals: &generated.normals,
                faces: &character.mesh.faces,
                export_body: false,
                joint_indices: &character.skin_weights.index,
                joint_weights: &character.skin_weights.weight,
                joint_names: &character.skeleton.names,
                joint_parents: &character.skeleton.parents,
                global_joint_states: &generated.global_joint_states,
            };
            let surface_uv_layout = SurfaceUvLayout {
                domain: MHR_ANATOMICAL_UV_DOMAIN,
                texcoords: &character.mesh.texcoords,
                texcoord_faces: &character.mesh.texcoord_faces,
            };
            let sockets = equipment
                .attachment_points
                .iter()
                .filter(|point| point.tangent_direction.is_some())
                .map(|point| {
                    let tangent = point.tangent_direction.expect("filtered tangent");
                    let outward = point
                        .locations
                        .iter()
                        .copied()
                        .find_map(belt_mount_outward)
                        .with_context(|| {
                            format!(
                                "item {} attachment point {} has a tangent but no belt location",
                                item.id, point.id
                            )
                        })?;
                    let surface = point.surface_uv.as_ref().with_context(|| {
                        format!(
                            "item {} attachment point {} has a tangent but no anatomical surface UV",
                            item.id, point.id
                        )
                    })?;
                    if surface.domain != surface_uv_layout.domain {
                        anyhow::bail!(
                            "item {} attachment point {} uses unsupported anatomical UV domain {}",
                            item.id,
                            point.id,
                            surface.domain
                        );
                    }
                    fitted_equipment_socket_from_uv(
                        &rigged_mesh,
                        &rigged_shell,
                        &surface_uv_layout,
                        surface.uv,
                        outward,
                        tangent,
                    )
                        .with_context(|| {
                            format!(
                                "could not fit item {} attachment point {}",
                                item.id, point.id
                            )
                        })
                        .map(|transform| RiggedSocket {
                            attachment_point_id: &point.id,
                            surface_uv_domain: &surface.domain,
                            surface_uv: surface.uv,
                            transform,
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            export_rigged_glb(
                &path,
                &item.id,
                recipe.version,
                model.lod,
                &rigged_mesh,
                &[rigged_shell],
                &sockets,
            )?;
            generated_files.insert(file_name.clone());
            assets.push(serde_json::json!({
                "item_id": item.id,
                "placement_id": placement.id,
                "file": file_name,
                "coverage": placement_coverage(placement),
                "material": equipment.material,
                "triangles": shell.faces.len(),
            }));
        }
    }
    for entry in std::fs::read_dir(output)? {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "glb")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !generated_files.contains(name))
        {
            std::fs::remove_file(&path)
                .with_context(|| format!("removing stale generated asset {}", path.display()))?;
        }
    }
    let manifest = serde_json::json!({
        "schema_version": 1,
        "mhr_release": "v1.0.1",
        "lod": model.lod,
        "assets": assets,
    });
    std::fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&manifest)?,
    )?;
    Ok(())
}

fn load_body_model(
    assets: &std::path::Path,
    lod: u8,
    correctives: bool,
    device: &Device,
) -> Result<BodyModel> {
    let mhr = Mhr::from_files(
        assets,
        MhrConfig {
            lod,
            pose_correctives: correctives,
        },
        device,
    )?;
    Ok(BodyModel {
        mhr,
        lod,
        correctives,
    })
}

fn reload_model(args: Res<Args>, mut model: ResMut<BodyModel>, mut studio: ResMut<Studio>) {
    if studio.selected_lod == model.lod && studio.selected_correctives == model.correctives {
        return;
    }
    let requested = studio.selected_lod;
    let requested_correctives = studio.selected_correctives;
    let device = Device::default();
    match load_body_model(&args.assets, requested, requested_correctives, &device) {
        Ok(loaded) => {
            *model = loaded;
            studio.dirty = true;
            studio.status = format!(
                "MHR LOD {requested} ready · correctives {}",
                if requested_correctives { "on" } else { "off" }
            );
        }
        Err(error) => {
            studio.selected_lod = model.lod;
            studio.selected_correctives = model.correctives;
            studio.status = format!(
                "Could not load LOD {requested} with correctives {}: {error:#}",
                if requested_correctives { "on" } else { "off" }
            );
        }
    }
}

fn regenerate_mesh(
    mut commands: Commands,
    model: Res<BodyModel>,
    catalog: Res<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
    old: Query<Entity, With<CharacterMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !studio.dirty {
        return;
    }
    studio.dirty = false;
    let generated = match generate_character(&model, &studio.recipe) {
        Ok(generated) => generated,
        Err(error) => {
            studio.status = format!("Generation failed: {error:#}");
            return;
        }
    };
    let faces = &model.mhr.character.mesh.faces;
    let specifications = match selected_garments(&studio.recipe, &catalog) {
        Ok(specifications) => specifications,
        Err(error) => {
            studio.status = format!("Clothing selection failed: {error}");
            return;
        }
    };
    let clothed = match generate_clothing_shells(
        &specifications,
        &generated.positions,
        &generated.normals,
        faces,
        &model.mhr.character.skin_weights.index,
        &model.mhr.character.skin_weights.weight,
        &model.mhr.character.skeleton.names,
        &generated.global_joint_states,
    ) {
        Ok(clothed) => clothed,
        Err(error) => {
            studio.status = format!("Clothing generation failed: {error}");
            return;
        }
    };
    let bracers = match selected_vambrace_sides(&studio.recipe)
        .into_iter()
        .map(|side| fitted_bracer(&model, &generated, &studio.bracer_design, side, &[]))
        .collect::<Result<Vec<_>>>()
    {
        Ok(bracers) => bracers,
        Err(error) => {
            studio.status = format!("Parametric bracer generation failed: {error:#}");
            return;
        }
    };
    let breastplate = if breastplate_selected(&studio.recipe) {
        match fitted_breastplate(&model, &generated, &studio.breastplate_design, &[]) {
            Ok(breastplate) => Some(breastplate),
            Err(error) => {
                studio.status = format!("Parametric breastplate generation failed: {error:#}");
                return;
            }
        }
    } else {
        None
    };
    let indices = clothed
        .visible_body_faces
        .iter()
        .flat_map(|face| face.iter().copied())
        .collect::<Vec<_>>();
    let clothing_shell_count = clothed.shells.len();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, generated.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, generated.normals.clone())
    .with_inserted_indices(Indices::U32(indices));
    for entity in &old {
        commands.entity(entity).despawn();
    }
    commands.spawn((
        CharacterMesh,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            // Skin is a rough dielectric with a small amount of diffuse
            // transmission. This keeps thin features such as the nose, ears,
            // and fingers warm instead of crushing them to black.
            base_color: Color::srgb(0.64, 0.39, 0.30),
            metallic: 0.0,
            perceptual_roughness: 0.52,
            reflectance: 0.46,
            specular_tint: Color::srgb(1.0, 0.93, 0.89),
            // A small back-diffuse lobe is Bevy's inexpensive approximation
            // of the short scattering distance seen in skin. Kept subtle so
            // the body remains opaque and shadowed rather than wax-like.
            diffuse_transmission: 0.045,
            ..default()
        })),
    ));
    for shell in clothed.shells {
        let specification = shell.specification;
        let indices = shell
            .faces
            .iter()
            .flat_map(|face| face.iter().copied())
            .collect::<Vec<_>>();
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, shell.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, shell.normals)
        .with_inserted_indices(Indices::U32(indices));
        let [red, green, blue, alpha] = specification.base_color;
        commands.spawn((
            CharacterMesh,
            Name::new(specification.name.clone()),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(red, green, blue, alpha),
                metallic: specification.metallic,
                perceptual_roughness: specification.roughness,
                ..default()
            })),
        ));
    }
    for (index, bracer) in bracers.iter().enumerate() {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, bracer.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, bracer.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, bracer.texcoords.clone())
        .with_inserted_indices(Indices::U32(bracer.indices.clone()));
        commands.spawn((
            CharacterMesh,
            Name::new(format!("Parametric bracer {}", index + 1)),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.769, 0.776, 0.776),
                metallic: 1.0,
                perceptual_roughness: 0.20,
                ..default()
            })),
        ));
    }
    if let Some(breastplate) = &breastplate {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, breastplate.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, breastplate.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, breastplate.texcoords.clone())
        .with_inserted_indices(Indices::U32(breastplate.indices.clone()));
        commands.spawn((
            CharacterMesh,
            Name::new("Parametric front breastplate"),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.769, 0.776, 0.776),
                metallic: 1.0,
                perceptual_roughness: 0.20,
                ..default()
            })),
        ));
    }
    studio.status = format!(
        "Generated {} body vertices · {} clothing shells · {} bracers · {} breastplate",
        model.mhr.num_vertices(),
        clothing_shell_count,
        bracers.len(),
        usize::from(breastplate.is_some()),
    );
}

fn generate_character(model: &BodyModel, recipe: &CharacterRecipe) -> Result<GeneratedCharacter> {
    recipe.validate().map_err(anyhow::Error::msg)?;
    let device = Device::default();
    let identity = Tensor::from_data(TensorData::new(recipe.identity.clone(), [1, 45]), &device);
    let expression = Tensor::from_data(
        TensorData::new(
            recipe.expression.clone(),
            [1, NUM_FACE_EXPRESSION_BLEND_SHAPES],
        ),
        &device,
    );
    let pose = model.mhr.zero_parameters(1);
    let output = model.mhr.forward(identity, pose, Some(expression))?;
    let vertex_values = output
        .vertices
        .into_data()
        .into_vec::<f32>()
        .map_err(|error| anyhow::anyhow!("GPU vertex readback failed: {error:?}"))?;
    let (vertex_chunks, vertex_remainder) = vertex_values.as_chunks::<3>();
    if !vertex_remainder.is_empty() {
        return Err(anyhow::anyhow!(
            "GPU vertex readback did not contain complete three-axis positions"
        ));
    }
    let positions: Vec<[f32; 3]> = vertex_chunks
        .iter()
        .map(|v| [v[0] / 100.0, v[1] / 100.0, v[2] / 100.0])
        .collect();

    let normal_values = output
        .normals
        .into_data()
        .into_vec::<f32>()
        .map_err(|error| anyhow::anyhow!("GPU normal readback failed: {error:?}"))?;
    let (normal_chunks, normal_remainder) = normal_values.as_chunks::<3>();
    if !normal_remainder.is_empty() {
        return Err(anyhow::anyhow!(
            "GPU normal readback did not contain complete three-axis normals"
        ));
    }
    let normals: Vec<[f32; 3]> = normal_chunks.iter().map(|n| [n[0], n[1], n[2]]).collect();

    let skeleton_values = output
        .skeleton_state
        .into_data()
        .into_vec::<f32>()
        .map_err(|error| anyhow::anyhow!("GPU skeleton readback failed: {error:?}"))?;
    let (skeleton_chunks, skeleton_remainder) = skeleton_values.as_chunks::<8>();
    if !skeleton_remainder.is_empty() {
        return Err(anyhow::anyhow!(
            "GPU skeleton readback did not contain complete joint states"
        ));
    }
    let global_joint_states = skeleton_chunks
        .iter()
        .map(|joint| {
            let mut state = *joint;
            state[0] /= 100.0;
            state[1] /= 100.0;
            state[2] /= 100.0;
            state
        })
        .collect();
    Ok(GeneratedCharacter {
        positions,
        normals,
        global_joint_states,
    })
}

fn generate_morph_samples(
    model: &BodyModel,
    base: &CharacterRecipe,
) -> Result<Vec<ForearmMorphSample>> {
    let names = &model.mhr.character.blend_shapes.names;
    if names.len() != IDENTITY_COUNT + EXPRESSION_COUNT {
        anyhow::bail!(
            "MHR exposes {} morph targets, expected {} identity plus {} expression targets",
            names.len(),
            IDENTITY_COUNT,
            EXPRESSION_COUNT,
        );
    }
    let mut samples = Vec::with_capacity(names.len() + 1);
    for (index, name) in names.iter().enumerate() {
        let signs: &[(f32, &str)] = if index == 0 {
            &[(1.0, "positive"), (-1.0, "negative")]
        } else {
            &[(1.0, "positive")]
        };
        for &(sign, suffix) in signs {
            let mut positive = if index == 0 {
                // `shape_0` is the one signed runtime target. Its endpoints
                // are absolute Body01 reference wearers, not one-unit offsets
                // from the nonzero saved John recipe used by the zero-weight
                // production mesh.
                signed_shape_zero_endpoint_recipe(base, sign)
            } else {
                base.clone()
            };
            if index != 0 {
                if index < IDENTITY_COUNT {
                    positive.identity[index] += sign;
                } else {
                    positive.expression[index - IDENTITY_COUNT] += sign;
                }
            }
            let positive = generate_character(model, &positive)
                .with_context(|| format!("generating {suffix} armor morph target {name}"))?;
            samples.push(ForearmMorphSample {
                name: format!("{name}::{suffix}"),
                positions: positive.positions,
                normals: positive.normals,
                global_joint_states: positive.global_joint_states,
            });
        }
    }
    Ok(samples)
}

fn center_signed_armor_morphs(mut armor: GeneratedArmor) -> Result<GeneratedArmor> {
    if armor.morphs.is_empty() {
        return Ok(armor);
    }
    let mut source = armor.morphs.into_iter();
    let first_positive = source
        .next()
        .ok_or_else(|| anyhow::anyhow!("shape_0 positive armor morph is missing"))?;
    let first_name = first_positive
        .name
        .strip_suffix("::positive")
        .ok_or_else(|| anyhow::anyhow!("shape_0 positive suffix is missing"))?;
    let mut first_negative = source
        .next()
        .ok_or_else(|| anyhow::anyhow!("shape_0 negative armor morph is missing"))?;
    if first_negative.name.strip_suffix("::negative") != Some(first_name) {
        anyhow::bail!("shape_0 signed armor pair names differ");
    }
    first_negative.name = first_name.to_owned();
    // GLTF weight -1 must land on the explicitly fitted negative endpoint:
    // base + (-1) * (base - negative) == negative.
    for delta in &mut first_negative.position_deltas {
        *delta = delta.map(|value| -value);
    }
    for delta in &mut first_negative.normal_deltas {
        *delta = delta.map(|value| -value);
    }
    let mut centered = vec![first_negative];
    for mut positive in source {
        positive.name = positive
            .name
            .strip_suffix("::positive")
            .ok_or_else(|| anyhow::anyhow!("positive armor morph suffix is missing"))?
            .to_owned();
        centered.push(positive);
    }
    for (index, morph) in centered.iter().enumerate() {
        let weight = if index == 0 { -1.0 } else { 1.0 };
        if morph.direct_positions.len() != armor.positions.len()
            || morph.position_deltas.len() != armor.positions.len()
        {
            anyhow::bail!(
                "armor morph {} has inconsistent endpoint topology",
                morph.name
            );
        }
        let maximum_error = armor
            .positions
            .iter()
            .zip(&morph.position_deltas)
            .zip(&morph.direct_positions)
            .map(|((base, delta), direct)| {
                (0..3)
                    .map(|axis| (base[axis] + weight * delta[axis] - direct[axis]).abs())
                    .fold(0.0_f32, f32::max)
            })
            .fold(0.0_f32, f32::max);
        if maximum_error > 2.0e-6 {
            anyhow::bail!(
                "armor morph {} endpoint encoding error {maximum_error}m exceeds 0.000002m",
                morph.name
            );
        }
    }
    armor.morphs = centered;
    Ok(armor)
}

#[cfg(test)]
mod equipment_morph_tests {
    use super::*;
    use adventuresim_armor_model::ArmorMorph;

    #[test]
    fn signed_shape_zero_endpoint_is_the_absolute_body01_wearer() {
        let mut wearer = CharacterRecipe::default();
        wearer.identity.fill(0.75);
        wearer.expression.fill(-0.25);
        let endpoint = signed_shape_zero_endpoint_recipe(&wearer, -1.0);
        assert_eq!(endpoint.version, wearer.version);
        assert_eq!(endpoint.identity[0], -1.0);
        assert!(endpoint.identity[1..].iter().all(|value| *value == 0.0));
        assert!(endpoint.expression.iter().all(|value| *value == 0.0));
        assert!(endpoint.clothing.is_empty());
    }

    #[test]
    fn runtime_negative_weight_reconstructs_the_independent_endpoint_vertices() {
        let base = vec![[0.2, 1.1, -0.3], [-0.4, 0.7, 0.5]];
        let negative_endpoint = [[0.1, 1.05, -0.28], [-0.36, 0.68, 0.44]];
        let endpoint_deltas = negative_endpoint
            .iter()
            .zip(&base)
            .map(|(endpoint, origin)| std::array::from_fn(|axis| endpoint[axis] - origin[axis]))
            .collect::<Vec<_>>();
        let zero_normals = vec![[0.0; 3]; base.len()];
        let armor = GeneratedArmor {
            design_hash: [0; 32],
            surface_domain: "test".into(),
            positions: base.clone(),
            normals: zero_normals.clone(),
            texcoords: vec![[0.0; 2]; base.len()],
            joint_indices: vec![[0; 8]; base.len()],
            joint_weights: vec![[0.0; 8]; base.len()],
            indices: vec![0, 1, 0],
            morphs: vec![
                ArmorMorph {
                    name: "shape_0::positive".into(),
                    direct_positions: base.clone(),
                    position_deltas: vec![[0.0; 3]; base.len()],
                    normal_deltas: zero_normals.clone(),
                },
                ArmorMorph {
                    name: "shape_0::negative".into(),
                    direct_positions: negative_endpoint.to_vec(),
                    position_deltas: endpoint_deltas,
                    normal_deltas: zero_normals,
                },
            ],
        };
        let centered = center_signed_armor_morphs(armor).unwrap();
        let applied = base
            .iter()
            .zip(&centered.morphs[0].position_deltas)
            .map(|(origin, delta)| std::array::from_fn(|axis| origin[axis] - delta[axis]))
            .collect::<Vec<[f32; 3]>>();
        for (actual, expected) in applied.iter().zip(negative_endpoint) {
            for axis in 0..3 {
                assert!((actual[axis] - expected[axis]).abs() <= 1.0e-7);
            }
        }
    }
}

fn export_character(
    path: &std::path::Path,
    model: &BodyModel,
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
    bracer_design: &BracerDesign,
    breastplate_design: &BreastplateDesign,
) -> Result<()> {
    let generated = generate_character(model, recipe)?;
    let character = &model.mhr.character;
    let specifications = selected_garments(recipe, catalog).map_err(anyhow::Error::msg)?;
    let clothed = generate_clothing_shells(
        &specifications,
        &generated.positions,
        &generated.normals,
        &character.mesh.faces,
        &character.skin_weights.index,
        &character.skin_weights.weight,
        &character.skeleton.names,
        &generated.global_joint_states,
    )
    .map_err(anyhow::Error::msg)?;
    let bracers = selected_vambrace_sides(recipe)
        .into_iter()
        .map(|side| fitted_bracer(model, &generated, bracer_design, side, &[]))
        .collect::<Result<Vec<_>>>()?;
    let bracer_faces = bracers
        .iter()
        .map(|bracer| bracer.indices.as_chunks::<3>().0.to_vec())
        .collect::<Vec<_>>();
    let breastplate = breastplate_selected(recipe)
        .then(|| fitted_breastplate(model, &generated, breastplate_design, &[]))
        .transpose()?;
    let breastplate_faces = breastplate
        .as_ref()
        .map(|armor| armor.indices.as_chunks::<3>().0.to_vec());
    let mut shells = clothed
        .shells
        .iter()
        .map(|shell| {
            let specification = &shell.specification;
            RiggedShell {
                name: &specification.name,
                positions: &shell.positions,
                normals: &shell.normals,
                faces: &shell.faces,
                joint_indices: None,
                joint_weights: None,
                morph_targets: &[],
                base_color: specification.base_color,
                metallic: specification.metallic,
                roughness: specification.roughness,
            }
        })
        .collect::<Vec<_>>();
    for (index, (bracer, faces)) in bracers.iter().zip(&bracer_faces).enumerate() {
        shells.push(RiggedShell {
            name: if index == 0 {
                "Parametric vambrace"
            } else {
                "Parametric vambrace pair"
            },
            positions: &bracer.positions,
            normals: &bracer.normals,
            faces,
            joint_indices: Some(&bracer.joint_indices),
            joint_weights: Some(&bracer.joint_weights),
            morph_targets: &[],
            base_color: [0.769, 0.776, 0.776, 1.0],
            metallic: 1.0,
            roughness: 0.20,
        });
    }
    if let (Some(breastplate), Some(faces)) = (&breastplate, &breastplate_faces) {
        shells.push(RiggedShell {
            name: "Parametric front breastplate",
            positions: &breastplate.positions,
            normals: &breastplate.normals,
            faces,
            joint_indices: Some(&breastplate.joint_indices),
            joint_weights: Some(&breastplate.joint_weights),
            morph_targets: &[],
            base_color: [0.769, 0.776, 0.776, 1.0],
            metallic: 1.0,
            roughness: 0.20,
        });
    }
    export_rigged_glb(
        path,
        &recipe.name,
        recipe.version,
        model.lod,
        &RiggedMesh {
            positions: &generated.positions,
            normals: &generated.normals,
            faces: &clothed.visible_body_faces,
            export_body: true,
            joint_indices: &character.skin_weights.index,
            joint_weights: &character.skin_weights.weight,
            joint_names: &character.skeleton.names,
            joint_parents: &character.skeleton.parents,
            global_joint_states: &generated.global_joint_states,
        },
        &shells,
        &[],
    )
}

fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    mut contexts: EguiContexts,
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mut camera: Query<(&mut Transform, &mut OrbitCamera)>,
) {
    let Ok((mut transform, mut orbit)) = camera.single_mut() else {
        return;
    };
    let pointer_owned_by_ui = contexts
        .ctx_mut()
        .is_ok_and(|context| context.egui_wants_pointer_input());
    if buttons.pressed(MouseButton::Left) && !pointer_owned_by_ui {
        for event in motion.read() {
            orbit.yaw -= event.delta.x * 0.007;
            orbit.pitch = (orbit.pitch - event.delta.y * 0.007).clamp(-1.2, 1.2);
        }
    } else {
        motion.clear();
    }
    if pointer_owned_by_ui {
        wheel.clear();
    } else {
        for event in wheel.read() {
            orbit.radius = (orbit.radius * (-event.y * 0.1).exp()).clamp(1.2, 6.0);
        }
    }
    let rotation = Quat::from_euler(EulerRot::YXZ, orbit.yaw, orbit.pitch, 0.0);
    transform.translation = orbit.focus + rotation * Vec3::new(0.0, 0.0, orbit.radius);
    transform.look_at(orbit.focus, Vec3::Y);
}

#[cfg(test)]
mod belt_mount_tests {
    use super::*;

    #[test]
    fn belt_sides_follow_mhr_anatomical_x() {
        assert_eq!(
            belt_mount_outward(EquipmentLocation::LeftBelt),
            Some([1.0, 0.0, 0.0])
        );
        assert_eq!(
            belt_mount_outward(EquipmentLocation::RightBelt),
            Some([-1.0, 0.0, 0.0])
        );
    }
}
