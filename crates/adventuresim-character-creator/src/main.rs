mod studio_scene;
use studio_scene::{CreatorPanelRight, orbit_camera, setup};
mod cli;
use cli::Args;
mod catalog;
use catalog::EquipmentCatalog;
mod fitted_existing;
use fitted_existing::{fitted_bracer, fitted_breastplate};
mod studio_generation;
use studio_generation::regenerate_mesh;
mod character_controls;
mod generation;
mod preview;
use generation::generate_character;
mod armor_controls;
mod breastplate_controls;
mod character_export;
mod character_morphs;
mod construction_controls;
mod decoration_controls;
mod device_equipment;
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
mod armory;
mod armory_ui;
mod bare_body;
mod drape_controls;
mod drape_preview;
mod drape_worker;
mod fabric_controls;
mod garment_controls;
mod inventory_ui;
mod metal_controls;
mod metal_preview;
mod ornament_controls;
mod outfit;
mod studio_overlay;
mod studio_theme;
mod studio_ui;
mod wardrobe_tab;
mod wardrobe_ui;
use adventuresim_character_creator::garment::{FabricPreset, GarmentForm, GarmentSelection};
use animation_preview::WalkPreview;
use drape_preview::DrapeJob;

use adventuresim_core::character_morph::IDENTITY_MORPH_COUNT;

use adventuresim_character_creator::{
    CharacterRecipe, IdentityGroup,
    bracer::{ForearmMorphSample, ForearmSide, ForearmSurfaceInput},
    clothing::{GarmentSpecification, generate_clothing_shells},
    device_torso::TorsoSurfaceInput,
    equipment_catalog::ItemCatalog,
    export::{
        GlbOutput, MHR_ANATOMICAL_UV_DOMAIN, RiggedMesh, RiggedMorphTarget, RiggedShell,
        RiggedSocket, SurfaceUvLayout, export_rigged_glb, fitted_equipment_socket_from_uv,
    },
    item_catalog_schema::{EquipmentLocation, ItemDefinition},
    item_design::CatalogDesigns,
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
use fabelgeist_armor::{BracerDesign, BreastplateDesign, GeneratedArmor};
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
    /// The open page of the character tab.
    page: character_controls::CharacterPage,
    dirty: bool,
    status: String,
    recipe_path: String,
    glb_path: String,
    seed: u64,
    selected_lod: u8,
    selected_correctives: bool,
    /// Where the catalog default designs are saved.
    design_paths: studio_ui::DesignPathInputs,
    /// Named engravings and trims, and where they are saved.
    decorations: decoration_controls::Decorations,
    /// Garments saved once settled, and where they are saved.
    wardrobe: wardrobe_tab::WardrobeLibrary,
    tab: studio_ui::StudioTab,
    inventory: inventory_ui::InventoryView,
}

impl Studio {
    fn new(args: &Args, recipe: CharacterRecipe) -> Self {
        let path = |path: &Option<std::path::PathBuf>, default: &str| {
            path.as_ref()
                .map_or_else(|| default.into(), |path| path.display().to_string())
        };
        Self {
            recipe,
            page: character_controls::CharacterPage::Build,
            dirty: true,
            status: format!("MHR LOD {} ready", args.lod),
            recipe_path: args.recipe.display().to_string(),
            glb_path: args.glb.display().to_string(),
            seed: 1544,
            selected_lod: args.lod,
            selected_correctives: false,
            design_paths: studio_ui::DesignPathInputs {
                catalog: path(&args.armor_designs, "target/armor-designs.json"),
                vambrace: path(&args.bracer_design, "target/bracer-design.json"),
                breastplate: path(&args.breastplate_design, "target/breastplate-design.json"),
            },
            decorations: decoration_controls::Decorations {
                library: default(),
                path: args.decorations.display().to_string(),
            },
            wardrobe: wardrobe_tab::WardrobeLibrary {
                library: default(),
                path: args.wardrobe.display().to_string(),
            },
            tab: studio_ui::StudioTab::Character,
            inventory: inventory_ui::InventoryView::default(),
        }
    }
}

#[derive(Component, Clone)]
struct CharacterMesh;

struct GeneratedCharacter {
    joint_proportions: Vec<adventuresim_core::character_proportions::JointProportionBasis>,
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    global_joint_states: Vec<[f32; 8]>,
    /// This body on the armor device, once a piece has uploaded it.
    device: adventuresim_character_creator::device_body::DeviceBody,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let device = Device::default();
    let model = load_body_model(&args.assets, args.lod, false, &device)
        .with_context(|| format!("loading MHR assets from {}", args.assets.display()))?;
    let designs = CatalogDesigns::load(
        args.armor_designs.as_deref(),
        args.bracer_design.as_deref(),
        args.breastplate_design.as_deref(),
    )?;
    let catalog = EquipmentCatalog(ItemCatalog::load(&args.catalog, designs)?);
    if let Some(path) = &args.write_armor_designs {
        let designs = catalog
            .items
            .iter()
            .filter_map(|item| {
                let design = catalog.design(&item.id)?;
                Some((item.id.clone(), design.recipe()?.clone()))
            })
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
        return review_export::export(output, &model, &recipe, &catalog);
    }

    if args.generate_equipment {
        generate_equipment_assets(
            &args.equipment_output,
            &model,
            &recipe,
            &catalog,
            &args.equipment_item,
        )?;
        println!(
            "Generated equipment under {}",
            args.equipment_output.display()
        );
        return Ok(());
    }
    if args.export_only {
        for warning in export_character(&args.glb, &model, &recipe, &catalog, None)? {
            eprintln!("warning: {warning}");
        }
        println!("Exported {}", args.glb.display());
        return Ok(());
    }

    let libraries = Libraries {
        decorations: adventuresim_character_creator::decoration::DecorationLibrary::load(
            &args.decorations,
        )?,
        wardrobe: adventuresim_character_creator::wardrobe::Wardrobe::load(&args.wardrobe)?,
    };
    run_studio(args, model, catalog, recipe, libraries);
    Ok(())
}

/// The studio's saved decorations and garments.
struct Libraries {
    decorations: adventuresim_character_creator::decoration::DecorationLibrary,
    wardrobe: adventuresim_character_creator::wardrobe::Wardrobe,
}

/// Open the interactive studio window on `recipe`.
fn run_studio(
    args: Args,
    model: BodyModel,
    catalog: EquipmentCatalog,
    recipe: CharacterRecipe,
    libraries: Libraries,
) {
    let mut studio = Studio::new(&args, recipe);
    studio.decorations.library = libraries.decorations;
    studio.wardrobe.library = libraries.wardrobe;
    App::new()
        .insert_resource(ClearColor(studio_scene::BACKDROP))
        .init_resource::<DrapeJob>()
        .init_resource::<drape_preview::MailMaterials>()
        .init_resource::<WalkPreview>()
        .insert_resource(args.clone())
        .init_resource::<underlayer_preview::EquipmentMaps>()
        .insert_resource(model)
        .insert_resource(catalog)
        .insert_resource(studio)
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
        .init_resource::<studio_scene::CharacterBounds>()
        .init_resource::<studio_scene::ShotRequest>()
        .init_resource::<armory::Armory>()
        .init_resource::<wardrobe_tab::WardrobeTab>()
        .add_systems(Startup, (setup, armory::setup, animation_preview::request))
        .add_systems(
            EguiPrimaryContextPass,
            (studio_theme::install, studio_ui::show, armory_ui::labels).chain(),
        )
        .add_systems(
            Update,
            (
                reload_model.before(regenerate_mesh),
                animation_preview::prepare,
                regenerate_mesh,
                drape_preview::poll.after(regenerate_mesh),
                drape_preview::refresh_mail.after(drape_preview::poll),
                (
                    armory::enter_or_leave,
                    armory::refit,
                    armory::display,
                    armory::frame_camera,
                )
                    .chain()
                    .after(reload_model)
                    .before(regenerate_mesh),
                (wardrobe_tab::update, wardrobe_tab::display)
                    .chain()
                    .after(reload_model)
                    .before(regenerate_mesh),
                studio_scene::frame_shot.after(armory::frame_camera),
                studio_scene::center_beside_panel,
                orbit_camera.after(studio_scene::frame_shot),
            ),
        )
        .add_systems(
            PostUpdate,
            animation_preview::deform_cloth.after(bevy::transform::TransformSystems::Propagate),
        )
        .run();
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
    use adventuresim_character_creator::{
        garment::{DrapeSettings, DrapeStage, drape},
        inventory::Article,
    };
    #[test]
    #[ignore = "requires MHR_ASSETS and a compute-capable GPU"]
    fn measured_mhr_garment_drapes_and_exports() -> Result<()> {
        let assets = std::env::var_os("MHR_ASSETS").context("set MHR_ASSETS")?;
        let model = load_body_model(std::path::Path::new(&assets), 1, false, &Device::default())?;
        let mut recipe = CharacterRecipe {
            inventory: Default::default(),
            ..CharacterRecipe::default()
        };
        let catalog = EquipmentCatalog(ItemCatalog::new(vec![], CatalogDesigns::authored())?);
        let generated = generate_character(&model, &recipe)?;
        // Every named shape, in mail unless GARMENT_TEST_FABRIC names another
        // fabric, and the fitted coif.
        let fabric = match std::env::var("GARMENT_TEST_FABRIC") {
            Ok(name) => *FabricPreset::ALL
                .iter()
                .find(|preset| preset.label() == name)
                .context("unknown GARMENT_TEST_FABRIC")?,
            Err(_) => FabricPreset::Chainmail,
        };
        let garments: Vec<_> = adventuresim_character_creator::garment::pattern::shapes::SHAPES
            .iter()
            .map(|shape| GarmentSelection {
                fabric,
                drape: DrapeSettings::for_fabric(fabric.fabric()),
                ..GarmentSelection::from_shape(shape)
            })
            .chain([GarmentSelection {
                name: "Coif".into(),
                ..GarmentSelection::chainmail_coif()
            }])
            .collect();
        if let Ok(name) = std::env::var("GARMENT_TEST_SHAPE") {
            assert!(
                garments.iter().any(|garment| garment.name == name),
                "unknown GARMENT_TEST_SHAPE"
            );
        }
        for selection in garments {
            if std::env::var("GARMENT_TEST_SHAPE").is_ok_and(|name| name != selection.name) {
                continue;
            }
            let label = selection.name.clone();
            recipe.inventory = Default::default();
            let id = recipe.inventory.add(Article::Draped(selection.clone()));
            recipe
                .inventory
                .wear(id, &catalog)
                .map_err(|conflict| anyhow::anyhow!("{conflict:?}"))?;
            println!("checking {label}");
            let input = drape_preview::input(&model, &generated, selection.clone());
            let fitted = drape(
                input,
                None,
                &std::sync::atomic::AtomicBool::new(false),
                |snapshot| {
                    let steps = selection.drape.settling.steps;
                    let write = |suffix: &str| {
                        let diagnostic = serde_json::json!({
                            "body": generated.positions,
                            "body_faces": model.mhr.character.mesh.faces,
                            "garment": snapshot.positions,
                            "garment_faces": snapshot.faces,
                            "uv": snapshot.texcoords,
                            "normals": snapshot.normals,
                        });
                        std::fs::write(
                            std::env::temp_dir()
                                .join(format!("fabelgeist-drape-{label}{suffix}.json")),
                            serde_json::to_vec(&diagnostic).unwrap(),
                        )
                        .unwrap();
                    };
                    if snapshot.stage
                        == (DrapeStage::Settling {
                            step: steps,
                            of: steps,
                        })
                    {
                        write("");
                    }
                    // Every milestone, to see where a drape goes wrong.
                    if std::env::var_os("GARMENT_TEST_SNAPSHOTS").is_some() {
                        match snapshot.stage {
                            DrapeStage::Placed => write("-placed"),
                            DrapeStage::Sewing { step, of } if step == of => write("-sewn"),
                            DrapeStage::Sewing { step, .. }
                                if [5, 10, 20, 30, 45].contains(&step) =>
                            {
                                write(&format!("-sewing{step}"))
                            }
                            DrapeStage::Settling { step, .. } if step % 20 == 0 => {
                                write(&format!("-settling{step}"))
                            }
                            _ => {}
                        }
                    }
                    let milestone = match snapshot.stage {
                        DrapeStage::Placed | DrapeStage::Worn => true,
                        DrapeStage::Sewing { .. } => false,
                        DrapeStage::Settling { step, .. } => step % 20 == 0,
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
            let steps = selection.drape.settling.steps;
            assert_eq!(
                fitted.stage,
                DrapeStage::Settling {
                    step: steps,
                    of: steps
                }
            );
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
                std::env::temp_dir().join(format!("fabelgeist-drape-{label}.json")),
                serde_json::to_vec(
                    &serde_json::json!({"body":generated.positions,"body_faces":model.mhr.character.mesh.faces,"garment":fitted.positions,"garment_faces":fitted.faces,"uv":fitted.texcoords,"normals":fitted.normals}),
                )?,
            )?;
            let maximum_distance = match selection.form() {
                GarmentForm::Upper | GarmentForm::Legged | GarmentForm::Fitted => 0.08,
                GarmentForm::Skirted => 0.25,
            };
            assert!(
                mean < maximum_distance,
                "{label} did not remain fitted to the body: {mean}",
            );
            let top = fitted
                .positions
                .iter()
                .map(|p| p[1])
                .fold(f32::NEG_INFINITY, f32::max);
            assert!(top > 0.8, "garment fell below the waist");
            let directory =
                std::env::temp_dir().join(format!("fabelgeist-drape-{}", std::process::id()));
            let path = directory.join(format!("{label}.glb"));
            for warning in export_character(
                &path,
                &model,
                &recipe,
                &catalog,
                Some(std::slice::from_ref(&fitted)),
            )? {
                println!("export warning: {warning}");
            }
            let bytes = std::fs::read(&path)?;
            let parsed = gltf::Gltf::from_slice(&bytes)?;
            assert_eq!(parsed.meshes().count(), 2);
            println!("verified draped character: {}", path.display());
        }
        Ok(())
    }
}

#[cfg(test)]
mod chainmail_export_test;

#[cfg(test)]
mod settled_garment_test;
