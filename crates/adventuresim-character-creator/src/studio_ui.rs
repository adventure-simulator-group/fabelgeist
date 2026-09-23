//! The studio side panel: tabs for the character's body, its inventory,
//! the armory, the wardrobe and output.
use super::*;
use adventuresim_character_creator::{
    armor_design_output::DesignPaths, decoration::DecorationLibrary, wardrobe::Wardrobe,
};

const PANEL_WIDTH: f32 = 392.0;
/// Height of a tab tile: its icon above its name.
const TAB_HEIGHT: f32 = 64.0;
/// Edge length of a tab's icon.
const TAB_ICON: f32 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum StudioTab {
    Character,
    Inventory,
    Armory,
    Wardrobe,
    Output,
}

impl StudioTab {
    const ALL: [Self; 5] = [
        Self::Character,
        Self::Inventory,
        Self::Armory,
        Self::Wardrobe,
        Self::Output,
    ];

    /// Whether the dressed character is shown; the armory and the wardrobe
    /// show their own work instead.
    pub(super) fn shows_character(self) -> bool {
        !matches!(self, Self::Armory | Self::Wardrobe)
    }

    fn label(self) -> &'static str {
        match self {
            Self::Character => "Character",
            Self::Inventory => "Inventory",
            Self::Armory => "Armory",
            Self::Wardrobe => "Wardrobe",
            Self::Output => "Output",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Character => "Shape the body, face and expression.",
            Self::Inventory => "Wear, carry and acquire equipment.",
            Self::Armory => "Browse and reshape the catalog's armor.",
            Self::Wardrobe => "Drape clothes and save them for any body.",
            Self::Output => "Save, animate and export the character.",
        }
    }

    fn icon(self) -> studio_theme::Icon {
        match self {
            Self::Character => studio_theme::Icon::Person,
            Self::Inventory => studio_theme::Icon::Knapsack,
            Self::Armory => studio_theme::Icon::Anvil,
            Self::Wardrobe => studio_theme::Icon::Clothes,
            Self::Output => studio_theme::Icon::OpenBook,
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
    reason = "Bevy injects the studio, drape, animation, camera and catalog resources into this system"
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
    mut armory: ResMut<armory::Armory>,
    mut wardrobe: ResMut<wardrobe_tab::WardrobeTab>,
    mut shot: ResMut<studio_scene::ShotRequest>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    // egui points equal logical window pixels at bevy_egui's default scale.
    panel_right.0 = egui::SidePanel::left("creator")
        .exact_width(PANEL_WIDTH)
        .resizable(false)
        .frame(
            egui::Frame::new()
                .fill(studio_theme::PANEL)
                .inner_margin(egui::Margin::symmetric(16, 14)),
        )
        .show(ctx, |ui| {
            tabs(ui, &mut studio.tab);
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(studio.tab.label())
                    .heading()
                    .color(studio_theme::PARCHMENT),
            );
            ui.label(egui::RichText::new(studio.tab.description()).color(studio_theme::MUTED));
            ui.add_space(4.0);
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt(("creator_controls", studio.tab))
                .auto_shrink(false)
                .show(ui, |ui| match studio.tab {
                    StudioTab::Character => character_controls::show(ui, &mut studio, &mut shot.0),
                    StudioTab::Inventory => {
                        inventory_ui::show(ui, &mut studio, &mut catalog, &mut drape_job)
                    }
                    StudioTab::Armory => {
                        armory_ui::show(ui, &mut studio, &mut catalog, &mut armory)
                    }
                    StudioTab::Wardrobe => {
                        wardrobe_ui::show(ui, &mut studio, &catalog, &model, &mut wardrobe)
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
        })
        .response
        .rect
        .right();
    let screen = ctx.content_rect();
    let viewport = egui::Rect::from_min_max(egui::pos2(panel_right.0, screen.top()), screen.max);
    studio_overlay::show(ctx, &mut studio, &mut shot.0, viewport);
}

/// A row of icon tiles, one per tab; the open tab is lit in gold.
fn tabs(ui: &mut egui::Ui, open: &mut StudioTab) {
    let spacing = ui.spacing().item_spacing.x;
    let count = StudioTab::ALL.len() as f32;
    let width = (ui.available_width() - spacing * (count - 1.0)) / count;
    ui.horizontal(|ui| {
        for tab in StudioTab::ALL {
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(width, TAB_HEIGHT), egui::Sense::click());
            let selected = *open == tab;
            let hovered = response.hovered();
            let painter = ui.painter();
            if selected || hovered {
                painter.rect_filled(rect, 6, studio_theme::SURFACE);
            }
            if selected {
                let underline = egui::Rect::from_min_max(
                    egui::pos2(rect.left() + 10.0, rect.bottom() - 2.0),
                    egui::pos2(rect.right() - 10.0, rect.bottom()),
                );
                painter.rect_filled(underline, 1, studio_theme::GOLD);
            }
            let tint = if selected {
                studio_theme::GOLD_BRIGHT
            } else if hovered {
                studio_theme::PARCHMENT
            } else {
                studio_theme::MUTED
            };
            let icon = egui::Rect::from_center_size(
                egui::pos2(rect.center().x, rect.top() + 8.0 + TAB_ICON * 0.5),
                egui::Vec2::splat(TAB_ICON),
            );
            tab.icon().image().tint(tint).paint_at(ui, icon);
            ui.painter().text(
                egui::pos2(rect.center().x, rect.bottom() - 14.0),
                egui::Align2::CENTER_CENTER,
                tab.label(),
                egui::FontId::proportional(12.5),
                tint,
            );
            if response.on_hover_text(tab.description()).clicked() {
                *open = tab;
            }
        }
    });
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
    studio_theme::card(ui, "Recipe", |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut studio.recipe_path)
                .hint_text("character.json")
                .desired_width(f32::INFINITY),
        );
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
    });

    // A drape with problems still finishes; only a running one holds these back.
    let generated = !studio.dirty && !drape_job.running();
    studio_theme::card(ui, "Animation", |ui| {
        animation(ui, walk, animation_players, generated && walk.ready());
    });

    studio_theme::card(ui, "Rigged export", |ui| {
        ui.add(
            egui::TextEdit::singleline(&mut studio.glb_path)
                .hint_text("assets_src/biped/unarmed/base.glb")
                .desired_width(f32::INFINITY),
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
    });

    studio_theme::card(ui, "Catalog designs", |ui| {
        ui.small(
            "Defaults for newly acquired armor and for equipment asset generation. \
             Use “Make catalog default” on an inventory item to change them.",
        );
        let paths = &mut studio.design_paths;
        egui::Grid::new("design_paths")
            .num_columns(2)
            .show(ui, |ui| {
                for (label, path) in [
                    ("Catalog armor", &mut paths.catalog),
                    ("Vambrace", &mut paths.vambrace),
                    ("Breastplate", &mut paths.breastplate),
                ] {
                    ui.label(label);
                    ui.add(egui::TextEdit::singleline(path).desired_width(f32::INFINITY));
                    ui.end_row();
                }
            });
        if ui.button("Save all catalog designs").clicked() {
            save_designs(studio, catalog);
        }
    });

    studio_theme::card(ui, "Decoration library", |ui| {
        decoration_library(ui, studio)
    });

    studio_theme::card(ui, "Wardrobe", |ui| wardrobe_library(ui, studio));

    studio_theme::card(ui, "Body model", |ui| mesh(ui, studio));
}

/// Where the armory saves decorations, and reloading them from there.
fn decoration_library(ui: &mut egui::Ui, studio: &mut Studio) {
    ui.small(
        "Engravings and trims saved from the armory, which inventory \
         articles choose from.",
    );
    let decorations = &mut studio.decorations;
    ui.add(egui::TextEdit::singleline(&mut decorations.path).desired_width(f32::INFINITY));
    if ui.button("Reload library").clicked() {
        let path = std::path::Path::new(&decorations.path);
        studio.status = match DecorationLibrary::load(path) {
            Ok(library) => {
                decorations.library = library;
                format!("Loaded decorations from {}", decorations.path)
            }
            Err(error) => format!("Could not load decorations: {error:#}"),
        };
    }
}

/// Where the wardrobe saves garments, and reloading them from there.
fn wardrobe_library(ui: &mut egui::Ui, studio: &mut Studio) {
    ui.small("Garments draped and saved in the wardrobe, which the inventory adds.");
    let wardrobe = &mut studio.wardrobe;
    ui.add(egui::TextEdit::singleline(&mut wardrobe.path).desired_width(f32::INFINITY));
    if ui.button("Reload wardrobe").clicked() {
        let path = std::path::Path::new(&wardrobe.path);
        studio.status = match Wardrobe::load(path) {
            Ok(library) => {
                wardrobe.library = library;
                format!("Loaded the wardrobe from {}", wardrobe.path)
            }
            Err(error) => format!("Could not load the wardrobe: {error:#}"),
        };
    }
}

/// Write every catalog default design to the paths set on the Output tab.
pub(super) fn save_designs(studio: &mut Studio, catalog: &EquipmentCatalog) {
    let paths = &studio.design_paths;
    studio.status = match catalog.designs.save(&DesignPaths {
        catalog: std::path::Path::new(&paths.catalog),
        bracer: std::path::Path::new(&paths.vambrace),
        breastplate: std::path::Path::new(&paths.breastplate),
    }) {
        Ok(()) => format!(
            "Saved catalog designs to {}, {} and {}",
            paths.catalog, paths.vambrace, paths.breastplate
        ),
        Err(error) => format!("Could not save catalog designs: {error:#}"),
    };
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
