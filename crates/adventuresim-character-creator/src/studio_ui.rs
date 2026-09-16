//! The studio side panel: the character's body, its inventory, and output.
use super::*;
use adventuresim_character_creator::armor_design_output::DesignPaths;

const PANEL_WIDTH: f32 = 380.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum StudioTab {
    Character,
    Inventory,
    Output,
}

impl StudioTab {
    const ALL: [Self; 3] = [Self::Character, Self::Inventory, Self::Output];

    fn label(self) -> &'static str {
        match self {
            Self::Character => "Character",
            Self::Inventory => "Inventory",
            Self::Output => "Output",
        }
    }
}

/// Editable paths for saving the catalog default designs.
pub(super) struct DesignPathInputs {
    pub catalog: String,
    pub vambrace: String,
    pub breastplate: String,
}

#[expect(
    deprecated,
    reason = "egui's replacement requires a parent Ui, but this is the top-level panel"
)]
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects the studio, drape, animation and catalog resources into this system"
)]
pub(super) fn show(
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
        .exact_width(PANEL_WIDTH)
        .show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut studio.recipe.name);
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                for tab in StudioTab::ALL {
                    ui.selectable_value(&mut studio.tab, tab, tab.label());
                }
            });
            ui.separator();
            egui::Panel::bottom("creator_status").show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.small(&studio.status);
                ui.small("Drag to orbit · wheel to zoom");
            });
            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt(("creator_controls", studio.tab))
                    .show(ui, |ui| match studio.tab {
                        StudioTab::Character => character(ui, &mut studio),
                        StudioTab::Inventory => {
                            inventory_ui::show(ui, &mut studio, &mut catalog, &mut drape_job)
                        }
                        StudioTab::Output => output(
                            ui,
                            &mut studio,
                            &catalog,
                            &model,
                            &drape_job,
                            &mut walk,
                            &mut animation_players,
                        ),
                    });
            });
        })
        .response
        .rect
        .right();
}

fn character(ui: &mut egui::Ui, studio: &mut Studio) {
    proportion_controls::show(ui, studio);
    proportion_controls::show_identity(ui, studio);
}

fn output(
    ui: &mut egui::Ui,
    studio: &mut Studio,
    catalog: &EquipmentCatalog,
    model: &BodyModel,
    drape_job: &DrapeJob,
    walk: &mut WalkPreview,
    animation_players: &mut Query<&mut AnimationPlayer>,
) {
    ui.heading("Body model");
    mesh(ui, studio);
    ui.separator();

    ui.heading("Recipe");
    ui.add(egui::TextEdit::singleline(&mut studio.recipe_path).hint_text("character.json"));
    ui.horizontal(|ui| {
        if ui.button("Save recipe").clicked() {
            studio.status =
                save_recipe(studio).unwrap_or_else(|error| format!("Save failed: {error:#}"));
        }
        if ui.button("Load recipe").clicked() {
            match load_recipe(&studio.recipe_path) {
                Ok(recipe) => {
                    studio.recipe = recipe;
                    studio.inventory = default();
                    studio.dirty = true;
                    studio.status = "Recipe loaded".into();
                }
                Err(error) => studio.status = format!("Load failed: {error:#}"),
            }
        }
    });
    ui.separator();

    // A drape with problems still finishes; only a running one holds these back.
    let generated = !studio.dirty && !drape_job.running();
    ui.heading("Animation");
    animation(ui, walk, animation_players, generated && walk.ready());
    ui.separator();

    ui.heading("Rigged export");
    ui.add(
        egui::TextEdit::singleline(&mut studio.glb_path)
            .hint_text("assets_src/biped/unarmed/base.glb"),
    );
    if ui
        .add_enabled(generated, egui::Button::new("Export rigged GLB"))
        .on_disabled_hover_text("Wait for generation and draping to finish.")
        .clicked()
    {
        studio.status = match export_character(
            std::path::Path::new(&studio.glb_path),
            model,
            &studio.recipe,
            catalog,
            drape_job.ready.as_deref(),
        ) {
            Ok(warnings) if warnings.is_empty() => format!("Exported {}", studio.glb_path),
            Ok(warnings) => format!(
                "Exported {} with problems: {}",
                studio.glb_path,
                warnings.join("; ")
            ),
            Err(error) => format!("Export failed: {error:#}"),
        };
    }
    ui.separator();

    ui.heading("Catalog designs");
    ui.small(
        "Defaults for newly acquired armor and for equipment asset generation. \
         Use “Make catalog default” on an inventory item to change them.",
    );
    let paths = &mut studio.design_paths;
    for (label, path) in [
        ("Catalog armor", &mut paths.catalog),
        ("Vambrace", &mut paths.vambrace),
        ("Breastplate", &mut paths.breastplate),
    ] {
        ui.horizontal(|ui| {
            ui.label(label);
            ui.text_edit_singleline(path);
        });
    }
    if ui.button("Save all catalog designs").clicked() {
        let paths = &studio.design_paths;
        studio.status = match catalog.designs.save(&DesignPaths {
            catalog: std::path::Path::new(&paths.catalog),
            bracer: std::path::Path::new(&paths.vambrace),
            breastplate: std::path::Path::new(&paths.breastplate),
        }) {
            Ok(()) => "Saved catalog, vambrace and breastplate designs".into(),
            Err(error) => format!("Could not save catalog designs: {error:#}"),
        };
    }
}

fn mesh(ui: &mut egui::Ui, studio: &mut Studio) {
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
    ui.small("Correctives improve posed deformation but require substantially more memory.");
}

fn animation(
    ui: &mut egui::Ui,
    walk: &mut WalkPreview,
    animation_players: &mut Query<&mut AnimationPlayer>,
    ready: bool,
) {
    ui.add_enabled_ui(ready, |ui| {
        ui.checkbox(&mut walk.physics, "Keep simulating cloth during animation")
            .on_hover_text(
                "Uses particles, stretch constraints, gravity, and collision with the animated body.",
            );
    });
    ui.add_enabled_ui(ready && walk.physics, |ui| {
        ui.checkbox(&mut walk.ignore_cloth_weights, "Ignore cloth weights")
            .on_hover_text(
                "Cloth moves through gravity and body contact only. Unsupported garments can fall off.",
            );
    });
    ui.collapsing("Cloth simulation parameters", |ui| {
        let ignore_weights = walk.ignore_cloth_weights;
        let settings = &mut walk.simulation;
        ui.small("Changes apply live during physics animation.");
        ui.checkbox(
            &mut settings.self_collision,
            "Cloth self-collision and layers",
        );
        ui.add(
            egui::Slider::new(&mut settings.cloth_thickness, 0.001..=0.02)
                .text("Cloth contact thickness (m)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.contact_iterations, 1..=8).text("Contact iterations"),
        );
        ui.add(egui::Slider::new(&mut settings.gravity, 0.0..=30.0).text("Gravity (m/s²)"));
        ui.add(egui::Slider::new(&mut settings.damping, 0.0..=1.0).text("Velocity damping"));
        ui.add(
            egui::Slider::new(&mut settings.stretch_stiffness, 0.0..=1.0).text("Stretch stiffness"),
        );
        ui.add_enabled(
            !ignore_weights,
            egui::Slider::new(&mut settings.follow_strength, 0.0..=4.0)
                .text("Weight following multiplier"),
        );
        ui.add(
            egui::Slider::new(&mut settings.collision_margin, 0.001..=0.03)
                .text("Body clearance (m)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.collision_distance, 0.03..=0.3)
                .text("Collision search distance (m)"),
        );
        ui.add(egui::Slider::new(&mut settings.substeps, 1..=8).text("Substeps"));
        ui.add(egui::Slider::new(&mut settings.iterations, 1..=12).text("Constraint iterations"));
        ui.small("More substeps and iterations increase simulation cost.");
        if ui.button("Reset simulation parameters").clicked() {
            *settings = default();
        }
    });
    let label = if walk.playing {
        "Pause animation"
    } else {
        "Play animation"
    };
    if ui.add_enabled(ready, egui::Button::new(label)).clicked()
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
}
