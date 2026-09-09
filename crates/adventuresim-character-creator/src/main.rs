mod generation;
mod preview;
mod proportion_controls;
use generation::generate_character;
mod character_export;
mod character_morphs;
mod equipment_controls;
mod equipment_export;
use character_export::export_character;
use equipment_export::generate_equipment_assets;

use adventuresim_core::character_morph::IDENTITY_MORPH_COUNT;
use std::path::PathBuf;

use adventuresim_armor_model::{
    BracerDesign, BreastplateDesign, GeneratedArmor, generate_bracer, generate_breastplate,
};
use adventuresim_character_creator::{
    CharacterRecipe, ClothingSelection, IdentityGroup,
    bracer::{ForearmMorphSample, ForearmSide, ForearmSurfaceInput, build_forearm_surface},
    breastplate::{TorsoSurfaceInput, build_front_torso_surface},
    clothing::{GarmentSpecification, generate_clothing_shells},
    design_input::load_breastplate_design,
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
    #[arg(long, default_value = "assets_src/characters/mhr_base.json")]
    recipe: PathBuf,
    #[arg(long, default_value = "assets_src/biped/unarmed/base.glb")]
    glb: PathBuf,
    #[arg(long, default_value = "content/items")]
    catalog: PathBuf,
    #[arg(long, default_value = "assets/equipment/procedural")]
    equipment_output: PathBuf,
    /// BreastplateDesign JSON for studio, character, and equipment exports.
    #[arg(long)]
    breastplate_design: Option<PathBuf>,
    /// Export the selected recipe without opening the studio window.
    #[arg(long)]
    export_only: bool,
    /// Generate one procedural MHR asset for every armor/clothing placement.
    #[arg(long)]
    generate_equipment: bool,
    /// Restrict procedural equipment generation to one catalog item.
    #[arg(long, requires = "generate_equipment")]
    equipment_item: Option<String>,
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
    joint_proportions: Vec<adventuresim_core::character_proportions::JointProportionBasis>,
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
    let breastplate_design = load_breastplate_design(args.breastplate_design.as_deref())?;
    let device = Device::default();
    let model = load_body_model(&args.assets, args.lod, false, &device)
        .with_context(|| format!("loading MHR assets from {}", args.assets.display()))?;
    let catalog = EquipmentCatalog(load_item_catalog(&args.catalog)?);

    let recipe: CharacterRecipe = serde_json::from_slice(
        &std::fs::read(&args.recipe)
            .with_context(|| format!("reading character recipe {}", args.recipe.display()))?,
    )?;
    recipe.validate().map_err(anyhow::Error::msg)?;

    if args.generate_equipment {
        generate_equipment_assets(
            &args.equipment_output,
            &model,
            &recipe,
            &catalog,
            &breastplate_design,
            args.equipment_item.as_deref(),
        )?;
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
            &breastplate_design,
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
            breastplate_design,
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
            equipment_controls::bracer(ui, &mut studio);
            equipment_controls::breastplate(ui, &mut studio);
            ui.separator();

            proportion_controls::show(ui, &mut studio);
            proportion_controls::show_identity(ui, &mut studio);
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
    let armor = generate_bracer(design, &surface).map_err(anyhow::Error::new)?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
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
    let armor = generate_breastplate(design, &surface).map_err(anyhow::Error::new)?;
    Ok(character_morphs::correct_armor_fit(
        armor, generated, morphs,
    ))
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
    preview::spawn_body(&mut commands, &mut meshes, &mut materials, mesh);
    preview::spawn_clothing(&mut commands, &mut meshes, &mut materials, clothed.shells);
    for (index, bracer) in bracers.iter().enumerate() {
        preview::spawn_armor(
            &mut commands,
            &mut meshes,
            &mut materials,
            bracer,
            format!("Parametric bracer {}", index + 1),
        );
    }
    if let Some(breastplate) = &breastplate {
        preview::spawn_armor(
            &mut commands,
            &mut meshes,
            &mut materials,
            breastplate,
            "Parametric front breastplate".into(),
        );
    }
    studio.status = format!(
        "Generated {} body vertices · {} clothing shells · {} bracers · {} breastplate",
        model.mhr.num_vertices(),
        clothing_shell_count,
        bracers.len(),
        usize::from(breastplate.is_some()),
    );
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
