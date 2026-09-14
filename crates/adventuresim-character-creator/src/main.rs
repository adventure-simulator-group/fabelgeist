mod studio_scene;
use studio_scene::{CreatorPanelRight, orbit_camera, setup};
mod cli;
use cli::Args;
mod catalog;
use catalog::{EquipmentCatalog, load_item_catalog, procedural_items};
mod fitted_existing;
use fitted_existing::{fitted_bracer, fitted_breastplate};
mod studio_generation;
use studio_generation::regenerate_mesh;
mod generation;
mod preview;
mod proportion_controls;
use generation::generate_character;
mod armor_controls;
mod breastplate_controls;
mod character_export;
mod character_morphs;
mod equipment_controls;
mod equipment_export;
mod fluting_controls;
mod parametric_equipment;
mod review_export;
mod underlayer_equipment;
mod underlayer_preview;
use character_export::export_character;
use equipment_export::generate_equipment_assets;
mod animation_preview;
mod armor_preview;
mod drape_preview;
mod drape_controls;
mod fabric_controls;
use adventuresim_character_creator::garment::{FabricPreset, GarmentPreset, GarmentSelection};
use animation_preview::WalkPreview;
use drape_preview::DrapeJob;

use adventuresim_core::character_morph::IDENTITY_MORPH_COUNT;

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
        GlbOutput, MHR_ANATOMICAL_UV_DOMAIN, RiggedMesh, RiggedMorphTarget, RiggedShell,
        RiggedSocket, SurfaceUvLayout, export_rigged_glb, fitted_equipment_socket_from_uv,
    },
    item_catalog_schema::{EquipmentLocation, ItemCatalogDocument, ItemDefinition},
};
use anyhow::{Context, Result};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, skinning::SkinnedMeshInverseBindposes},
    prelude::*,
    render::render_resource::PrimitiveTopology,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use burn::tensor::{Device, Tensor, TensorData};
use clap::Parser;
use fabelgeist_mhr::{Mhr, MhrConfig, NUM_FACE_EXPRESSION_BLEND_SHAPES};
use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Resource)]
struct BodyModel {
    mhr: Mhr,
    lod: u8,
    correctives: bool,
}

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
    armor_designs_path: String,
    bracer_design_path: String,
    breastplate_design_path: String,
    bracer_design: BracerDesign,
    breastplate_design: BreastplateDesign,
}

impl Studio {
    fn new(
        args: &Args,
        recipe: CharacterRecipe,
        bracer_design: BracerDesign,
        breastplate_design: BreastplateDesign,
    ) -> Self {
        Self {
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
            armor_designs_path: args.armor_designs.as_ref().map_or_else(
                || "target/armor-designs.json".into(),
                |path| path.display().to_string(),
            ),
            bracer_design,
            breastplate_design,
            bracer_design_path: args.bracer_design.as_ref().map_or_else(
                || "target/bracer-design.json".into(),
                |path| path.display().to_string(),
            ),
            breastplate_design_path: args.breastplate_design.as_ref().map_or_else(
                || "target/breastplate-design.json".into(),
                |path| path.display().to_string(),
            ),
        }
    }
}

#[derive(Component)]
struct CharacterMesh;

struct GeneratedCharacter {
    joint_proportions: Vec<adventuresim_core::character_proportions::JointProportionBasis>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    global_joint_states: Vec<[f32; 8]>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bracer_design = adventuresim_character_creator::design_input::load_bracer_design(
        args.bracer_design.as_deref(),
    )?;
    let breastplate_design = load_breastplate_design(args.breastplate_design.as_deref())?;
    let device = Device::default();
    let model = load_body_model(&args.assets, args.lod, false, &device)
        .with_context(|| format!("loading MHR assets from {}", args.assets.display()))?;
    let catalog = EquipmentCatalog(
        load_item_catalog(&args.catalog)?,
        adventuresim_character_creator::armor_design_input::load(args.armor_designs.as_deref())?,
    );
    if let Some(path) = &args.write_armor_designs {
        let designs = catalog
            .0
            .iter()
            .filter_map(|item| catalog.design(&item.id).map(|d| (item.id.clone(), d)))
            .collect::<adventuresim_character_creator::armor_design_input::ArmorDesigns>();
        std::fs::write(path, serde_json::to_vec_pretty(&designs)?)?;
        return Ok(());
    }

    let recipe: CharacterRecipe = serde_json::from_slice(
        &std::fs::read(&args.recipe)
            .with_context(|| format!("reading character recipe {}", args.recipe.display()))?,
    )?;
    recipe.validate().map_err(anyhow::Error::msg)?;

    if let Some(output) = &args.armor_review_dir {
        return review_export::export(
            output,
            &model,
            &recipe,
            &catalog,
            &bracer_design,
            &breastplate_design,
        );
    }

    if args.generate_equipment {
        generate_equipment_assets(
            &args.equipment_output,
            &model,
            &recipe,
            &catalog,
            &bracer_design,
            &breastplate_design,
            &args.equipment_item,
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
            &bracer_design,
            &breastplate_design,
            None,
        )?;
        println!("Exported {}", args.glb.display());
        return Ok(());
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.035, 0.045, 0.055)))
        .init_resource::<DrapeJob>()
        .init_resource::<drape_preview::MailMaterials>()
        .init_resource::<WalkPreview>()
        .insert_resource(args.clone())
        .init_resource::<underlayer_preview::MailMaps>()
        .insert_resource(model)
        .insert_resource(catalog)
        .insert_resource(Studio::new(
            &args,
            recipe,
            bracer_design,
            breastplate_design,
        ))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: "../../assets".into(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fabelgeist · Character Studio".into(),
                        resolution: (1440, 900).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin::default())
        .init_resource::<CreatorPanelRight>()
        .add_systems(Startup, (setup, animation_preview::request))
        .add_systems(EguiPrimaryContextPass, studio_ui)
        .add_systems(
            Update,
            (
                reload_model.before(regenerate_mesh),
                animation_preview::prepare,
                regenerate_mesh,
                drape_preview::poll.after(regenerate_mesh),
                drape_preview::refresh_mail.after(drape_preview::poll),
                orbit_camera,
            ),
        )
        .add_systems(
            PostUpdate,
            animation_preview::deform_cloth.after(bevy::transform::TransformSystems::Propagate),
        )
        .run();
    Ok(())
}

#[expect(
    deprecated,
    reason = "egui's replacement requires a parent Ui, but this is the top-level panel"
)]
fn studio_ui(
    mut drape_job: ResMut<DrapeJob>,
    mut walk: ResMut<WalkPreview>,
    mut animation_players: Query<&mut AnimationPlayer>,
    mut contexts: EguiContexts,
    model: Res<BodyModel>,
    mut catalog: ResMut<EquipmentCatalog>,
    mut studio: ResMut<Studio>,
    mut panel_right: ResMut<CreatorPanelRight>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    // egui points equal logical window pixels at bevy_egui's default scale.
    panel_right.0 = egui::SidePanel::left("creator")
        .exact_width(360.0)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("creator_controls")
                .show(ui, |ui| {
                    ui.add_space(8.0);
                    ui.label("Name");
                    ui.text_edit_singleline(&mut studio.recipe.name);
                    let lod_changed = ui
                        .add(
                            egui::Slider::new(&mut studio.selected_lod, 0..=6)
                                .text("Mesh LOD")
                                .custom_formatter(|value, _| {
                                    let lod = value.round() as usize;
                                    let vertices =
                                        [73_639, 18_439, 10_661, 4_899, 2_461, 971, 595][lod];
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

                    if armor_preview::editor(ui, &mut studio.recipe.armor) {
                        studio.dirty = true;
                    }
                    ui.collapsing("Draped clothing", |ui| {
                        let before = studio.recipe.garments.clone();
                        let mut remove = None;
                        let mut move_up = None;
                        ui.small("Garments are draped from first to last, inside to outside.");
                        for (index, selection) in studio.recipe.garments.iter_mut().enumerate() {
                            ui.push_id(index, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Garment {}", index + 1));
                                    if index > 0 && ui.button("Move inward").clicked() {
                                        move_up = Some(index);
                                    }
                                    if ui.button("Remove").clicked() {
                                        remove = Some(index);
                                    }
                                });
                                egui::ComboBox::from_id_salt(("garment_preset", index))
                                    .selected_text(selection.preset.label())
                                    .show_ui(ui, |ui| {
                                        for preset in GarmentPreset::ALL {
                                            ui.selectable_value(
                                                &mut selection.preset,
                                                preset,
                                                preset.label(),
                                            );
                                        }
                                    });
                                fabric_controls::show(ui, index, selection);
                                drape_controls::show(ui, selection);
                            });
                            ui.separator();
                        }
                        if let Some(index) = remove {
                            studio.recipe.garments.remove(index);
                        } else if let Some(index) = move_up {
                            studio.recipe.garments.swap(index, index - 1);
                        }
                        if ui.button("Add garment").clicked() {
                            studio.recipe.garments.push(GarmentSelection::default());
                        }
                        let add_chainmail = ui.button("Add chainmail shirt").clicked();
                        if add_chainmail {
                            studio.recipe.garments.push(GarmentSelection::chainmail());
                        }
                        // Appearance edits refresh the shown garments without draping.
                        studio.dirty |= before.len() != studio.recipe.garments.len()
                            || before
                                .iter()
                                .zip(&studio.recipe.garments)
                                .any(|(old, new)| !old.same_simulation(new));
                        if ui.button("Drape again").clicked() {
                            drape_job.restart_from_placement();
                            studio.dirty = true;
                        }
                    });
                    ui.separator();
                    ui.collapsing("Catalog clothing and armor", |ui| {
                        if ui
                            .add_enabled(
                                !studio.recipe.clothing.is_empty(),
                                egui::Button::new("Uncheck all"),
                            )
                            .clicked()
                        {
                            studio.recipe.clothing.clear();
                            studio.dirty = true;
                        }
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
                                let replaced = drape_replaces(&studio.recipe, placement);
                                if ui
                                    .add_enabled(
                                        !replaced,
                                        egui::Checkbox::new(&mut enabled, label),
                                    )
                                    .on_hover_text(if replaced {
                                        "Replaced by the draped garment"
                                    } else {
                                        ""
                                    })
                                    .changed()
                                {
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
                    ui.small("Armor recipes follow body proportions and share its MHR skin.");
                    equipment_controls::show(ui, &mut catalog, &mut studio);
                    ui.separator();

                    proportion_controls::show(ui, &mut studio);
                    proportion_controls::show_identity(ui, &mut studio);
                    ui.add(
                        egui::TextEdit::singleline(&mut studio.recipe_path)
                            .hint_text("character.json"),
                    );
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
                    let animation_ready = !studio.dirty
                        && (studio.recipe.garments.is_empty() || drape_job.ready.is_some())
                        && walk.ready();
                    let animation_label = if walk.playing {
                        "Pause animation"
                    } else {
                        "Play animation"
                    };
                    ui.add_enabled_ui(animation_ready, |ui| {
                        ui.checkbox(
                            &mut walk.physics,
                            "Keep simulating cloth during animation",
                        )
                        .on_hover_text(
                            "Uses particles, stretch constraints, gravity, and collision with the animated body.",
                        );
                    });
                    ui.add_enabled_ui(animation_ready && walk.physics, |ui| {
                        ui.checkbox(&mut walk.ignore_cloth_weights, "Ignore cloth weights")
                            .on_hover_text("Cloth moves through gravity and body contact only. Unsupported garments can fall off.");
                    });
                    ui.collapsing("Cloth simulation parameters", |ui| {
                        let ignore_weights = walk.ignore_cloth_weights;
                        let settings = &mut walk.simulation;
                        ui.small("Changes apply live during physics animation.");
                        ui.checkbox(&mut settings.self_collision, "Cloth self-collision and layers");
                        ui.add(egui::Slider::new(&mut settings.cloth_thickness, 0.001..=0.02).text("Cloth contact thickness (m)"));
                        ui.add(egui::Slider::new(&mut settings.contact_iterations, 1..=8).text("Contact iterations"));
                        ui.add(egui::Slider::new(&mut settings.gravity, 0.0..=30.0).text("Gravity (m/s²)"));
                        ui.add(egui::Slider::new(&mut settings.damping, 0.0..=1.0).text("Velocity damping"));
                        ui.add(egui::Slider::new(&mut settings.stretch_stiffness, 0.0..=1.0).text("Stretch stiffness"));
                        ui.add_enabled(
                            !ignore_weights,
                            egui::Slider::new(&mut settings.follow_strength, 0.0..=4.0).text("Weight following multiplier"),
                        );
                        ui.add(egui::Slider::new(&mut settings.collision_margin, 0.001..=0.03).text("Body clearance (m)"));
                        ui.add(egui::Slider::new(&mut settings.collision_distance, 0.03..=0.3).text("Collision search distance (m)"));
                        ui.add(egui::Slider::new(&mut settings.substeps, 1..=8).text("Substeps"));
                        ui.add(egui::Slider::new(&mut settings.iterations, 1..=12).text("Constraint iterations"));
                        ui.small("More substeps and iterations increase simulation cost.");
                        if ui.button("Reset simulation parameters").clicked() {
                            *settings = default();
                        }
                    });
                    if ui
                        .add_enabled(animation_ready, egui::Button::new(animation_label))
                        .clicked()
                        && let Some(player) = walk.player
                        && let Ok(mut player) = animation_players.get_mut(player)
                    {
                        if walk.playing {
                            player.pause_all();
                        } else {
                            walk.start_or_resume(&mut player);
                        }
                        walk.playing = !walk.playing;
                    }
                    if ui
                        .add_enabled(
                            !studio.dirty
                                && (studio.recipe.garments.is_empty() || drape_job.ready.is_some()),
                            egui::Button::new("Export rigged GLB"),
                        )
                        .clicked()
                    {
                        studio.status = export_character(
                            std::path::Path::new(&studio.glb_path),
                            &model,
                            &studio.recipe,
                            &catalog,
                            &studio.bracer_design,
                            &studio.breastplate_design,
                            drape_job.ready.as_deref(),
                        )
                        .map(|()| format!("Exported {}", studio.glb_path))
                        .unwrap_or_else(|error| format!("Export failed: {error:#}"));
                    }
                    ui.add_space(6.0);
                    ui.small(&studio.status);
                    ui.small("Drag to orbit · wheel to zoom");
                });
        })
        .response
        .rect
        .right();
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

fn drape_replaces(
    recipe: &CharacterRecipe,
    placement: &adventuresim_character_creator::item_catalog_schema::EquipmentPlacement,
) -> bool {
    use adventuresim_character_creator::item_catalog_schema::EquipmentChannel;
    if recipe.garments.is_empty() {
        return false;
    }
    placement.occupancy.iter().any(|slot| {
        slot.channel == EquipmentChannel::BaseClothing
            && match slot.location {
                EquipmentLocation::Chest
                | EquipmentLocation::Stomach
                | EquipmentLocation::Back
                | EquipmentLocation::LeftArm
                | EquipmentLocation::RightArm
                | EquipmentLocation::LeftShoulder
                | EquipmentLocation::RightShoulder => recipe.garments.iter().any(|garment| {
                    matches!(
                        garment.preset,
                        GarmentPreset::Shirt | GarmentPreset::FittedShirt | GarmentPreset::Dress
                    )
                }),
                EquipmentLocation::LeftLeg | EquipmentLocation::RightLeg => {
                    recipe.garments.iter().any(|garment| {
                        !matches!(
                            garment.preset,
                            GarmentPreset::Shirt | GarmentPreset::FittedShirt
                        )
                    })
                }
                _ => false,
            }
    })
}
fn selected_garments(
    recipe: &CharacterRecipe,
    catalog: &EquipmentCatalog,
) -> Result<Vec<GarmentSpecification>, String> {
    recipe
        .clothing
        .iter()
        .filter(|selection| {
            !adventuresim_character_creator::armor_recipes::is_parametric(&selection.item_id)
        })
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
        .collect::<Result<Vec<_>, String>>()
        .map(|specifications| {
            specifications
                .into_iter()
                .filter(|spec| !drape_replaces(recipe, &spec.placement))
                .collect()
        })
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

#[cfg(test)]
mod garment_integration_tests {
    use super::*;
    use adventuresim_character_creator::garment::{DrapeStage, drape};
    #[test]
    #[ignore = "requires MHR_ASSETS and a compute-capable GPU"]
    fn measured_mhr_garment_drapes_and_exports() -> Result<()> {
        let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
        let model = load_body_model(std::path::Path::new(&assets), 1, false, &Device::default())?;
        let mut recipe = CharacterRecipe::default();
        recipe.clothing.clear();
        if std::env::var_os("GARMENT_TEST_ARMOR").is_some() {
            recipe.armor = Some(fabelgeist_armor::Armor::default());
        }
        let generated = generate_character(&model, &recipe)?;
        if let Ok(name) = std::env::var("GARMENT_TEST_PRESET") {
            assert!(
                GarmentPreset::ALL
                    .iter()
                    .any(|preset| preset.label() == name),
                "unknown GARMENT_TEST_PRESET"
            );
        }
        for preset in GarmentPreset::ALL {
            if std::env::var("GARMENT_TEST_PRESET").is_ok_and(|name| name != preset.label()) {
                continue;
            }
            recipe.garments = vec![GarmentSelection {
                preset,
                ..GarmentSelection::chainmail()
            }];
            println!("checking {}", preset.label());
            let mut input = drape_preview::input(&model, &generated, recipe.garments[0].clone());
            input.armor = recipe.armor.clone();
            let fitted = drape(
                input,
                None,
                &std::sync::atomic::AtomicBool::new(false),
                |snapshot| {
                    let steps = recipe.garments[0].drape.settling.steps;
                    if snapshot.stage == (DrapeStage::Settling { step: steps, of: steps }) {
                        let diagnostic = serde_json::json!({
                            "body": generated.positions,
                            "body_faces": model.mhr.character.mesh.faces,
                            "garment": snapshot.positions,
                            "garment_faces": snapshot.faces,
                            "uv": snapshot.texcoords,
                            "normals": snapshot.normals,
                        });
                        std::fs::write(
                            std::env::temp_dir().join(format!("fabelgeist-drape-{preset:?}.json")),
                            serde_json::to_vec(&diagnostic).unwrap(),
                        )
                        .unwrap();
                    }
                    let milestone = match snapshot.stage {
                        DrapeStage::Placed => true,
                        DrapeStage::Sewing { .. } => false,
                        DrapeStage::Settling { step, .. } => step % 60 == 0,
                    };
                    if milestone {
                        println!(
                            "drape {}: {} vertices",
                            snapshot.stage,
                            snapshot.positions.len()
                        );
                    }
                },
            )
            .result?;
            assert_eq!(fitted.stage, DrapeStage::Settling { step: 180, of: 180 });
            assert_eq!(fitted.indices.len(), fitted.positions.len());
            assert_ne!(fitted.positions.len(), generated.positions.len());
            let body = fabelgeist_bvh::TriangleBvh::new(
                generated
                    .positions
                    .iter()
                    .map(|p| fabelgeist_math::Vec3::from_array(*p))
                    .collect(),
                model.mhr.character.mesh.faces.clone(),
            );
            let distances: Vec<_> = fitted
                .positions
                .iter()
                .map(|p| {
                    body.closest_point(fabelgeist_math::Vec3::from_array(*p), f32::MAX)
                        .unwrap()
                        .2
                })
                .collect();
            let mean = distances.iter().sum::<f32>() / distances.len() as f32;
            println!("mean garment distance from body: {mean:.4} m");
            std::fs::write(
                std::env::temp_dir().join(format!("fabelgeist-drape-{preset:?}.json")),
                serde_json::to_vec(
                    &serde_json::json!({"body":generated.positions,"body_faces":model.mhr.character.mesh.faces,"garment":fitted.positions,"garment_faces":fitted.faces,"uv":fitted.texcoords,"normals":fitted.normals}),
                )?,
            )?;
            let maximum_distance = match preset {
                GarmentPreset::Shirt | GarmentPreset::FittedShirt | GarmentPreset::Trousers => 0.08,
                GarmentPreset::Skirt | GarmentPreset::Dress => 0.25,
            };
            assert!(
                mean < maximum_distance,
                "{} did not remain fitted to the body: {mean}",
                preset.label()
            );
            let top = fitted
                .positions
                .iter()
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(top > 0.8, "garment fell below the waist");
            let directory =
                std::env::temp_dir().join(format!("fabelgeist-drape-{}", std::process::id()));
            let path = directory.join(format!("{preset:?}.glb"));
            export_character(
                &path,
                &model,
                &recipe,
                &EquipmentCatalog(vec![], Default::default()),
                &BracerDesign::default(),
                &BreastplateDesign::default(),
                Some(std::slice::from_ref(&fitted)),
            )?;
            let bytes = std::fs::read(&path)?;
            let parsed = gltf::Gltf::from_slice(&bytes)?;
            let armor_parts = recipe
                .armor
                .as_ref()
                .map(fabelgeist_armor::build)
                .transpose()
                .map_err(anyhow::Error::msg)?
                .unwrap_or_default();
            assert_eq!(parsed.meshes().count(), 2 + armor_parts.len());
            println!("verified draped character: {}", path.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod chainmail_export_test;
