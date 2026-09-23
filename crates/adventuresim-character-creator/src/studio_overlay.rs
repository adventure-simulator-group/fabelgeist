//! What the studio draws over the viewport: the character's nameplate, the
//! camera and appearance actions, the status line and a vignette.
use super::*;
use studio_scene::Shot;
use studio_theme::{GOLD, GOLD_BRIGHT, MUTED, PANEL, PARCHMENT};
use studio_ui::StudioTab;

/// Space between the overlay and the viewport's edges.
const EDGE_MARGIN: f32 = 18.0;
/// Width of the character's name field.
const NAME_WIDTH: f32 = 420.0;
/// Segments around the vignette's ellipse.
const VIGNETTE_SEGMENTS: u32 = 64;
/// Opacity of the vignette where it meets the viewport's edge, and halfway in.
const VIGNETTE_ALPHA: (u8, u8) = (170, 55);
/// The vignette's clear centre, as a fraction of the viewport's half extents.
const VIGNETTE_CLEAR: egui::Vec2 = egui::vec2(0.62, 0.72);

pub(super) fn show(
    ctx: &egui::Context,
    studio: &mut Studio,
    shot: &mut Option<Shot>,
    viewport: egui::Rect,
) {
    // The armory frames its own wall and labels each piece in place.
    let dressed = studio.tab != StudioTab::Armory;
    if dressed {
        vignette(ctx, viewport);
        nameplate(ctx, studio, viewport);
    }
    let bar_top = dressed.then(|| action_bar(ctx, studio, shot, viewport));
    status(ctx, studio, viewport, bar_top);
}

/// Darken the viewport's edges so the character stands out from the room.
fn vignette(ctx: &egui::Context, viewport: egui::Rect) {
    let center = viewport.center();
    let half = viewport.size() * 0.5;
    let mut mesh = egui::Mesh::default();
    for segment in 0..VIGNETTE_SEGMENTS {
        let angle = std::f32::consts::TAU * segment as f32 / VIGNETTE_SEGMENTS as f32;
        let direction = egui::vec2(angle.cos(), angle.sin());
        let clear = center + direction * half * VIGNETTE_CLEAR;
        // Where the ray from the centre leaves the viewport.
        let reach = (half.x / direction.x.abs()).min(half.y / direction.y.abs());
        let edge = center + direction * reach;
        mesh.colored_vertex(clear, egui::Color32::TRANSPARENT);
        mesh.colored_vertex(
            clear.lerp(edge, 0.5),
            egui::Color32::from_black_alpha(VIGNETTE_ALPHA.1),
        );
        mesh.colored_vertex(edge, egui::Color32::from_black_alpha(VIGNETTE_ALPHA.0));
    }
    let rings = 3;
    for segment in 0..VIGNETTE_SEGMENTS {
        let [this, next] = [segment, (segment + 1) % VIGNETTE_SEGMENTS].map(|s| s * rings);
        for ring in 0..rings - 1 {
            let (a, b, c, d) = (this + ring, this + ring + 1, next + ring, next + ring + 1);
            mesh.add_triangle(a, b, c);
            mesh.add_triangle(b, d, c);
        }
    }
    ctx.layer_painter(egui::LayerId::background())
        .with_clip_rect(viewport)
        .add(mesh);
}

/// The character's name, centred above them and edited in place.
fn nameplate(ctx: &egui::Context, studio: &mut Studio, viewport: egui::Rect) {
    egui::Area::new("nameplate".into())
        .fixed_pos(egui::pos2(
            viewport.center().x,
            viewport.top() + EDGE_MARGIN,
        ))
        .pivot(egui::Align2::CENTER_TOP)
        .show(ctx, |ui| {
            ui.set_width(NAME_WIDTH);
            ui.vertical_centered(|ui| {
                ui.label(studio_theme::caps("Adventurer", 11.0, GOLD));
                let name = ui
                    .add(
                        egui::TextEdit::singleline(&mut studio.recipe.name)
                            .font(studio_theme::display_font())
                            .text_color(PARCHMENT)
                            .horizontal_align(egui::Align::Center)
                            .hint_text("Name your character")
                            .desired_width(NAME_WIDTH)
                            .frame(egui::Frame::NONE),
                    )
                    .on_hover_text("Click to rename");
                flourish(ui, name.rect.bottom() + 2.0, name.has_focus());
            });
        });
}

/// A gold rule under the name, bright at the centre and fading out.
fn flourish(ui: &egui::Ui, y: f32, focused: bool) {
    let center = ui.max_rect().center().x;
    let reach = NAME_WIDTH * 0.5;
    let strength = if focused { 1.0 } else { 0.6 };
    let mut mesh = egui::Mesh::default();
    let color = GOLD.gamma_multiply(strength);
    for (x, color) in [
        (center - reach, egui::Color32::TRANSPARENT),
        (center, color),
        (center + reach, egui::Color32::TRANSPARENT),
    ] {
        mesh.colored_vertex(egui::pos2(x, y), color);
        mesh.colored_vertex(egui::pos2(x, y + 1.0), color);
    }
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    mesh.add_triangle(2, 3, 4);
    mesh.add_triangle(3, 5, 4);
    ui.painter().add(mesh);
    ui.painter().circle_filled(
        egui::pos2(center, y + 0.5),
        2.5,
        GOLD_BRIGHT.gamma_multiply(strength),
    );
}

/// Camera shots and the appearance actions, along the viewport's foot.
/// Returns the bar's top edge.
fn action_bar(
    ctx: &egui::Context,
    studio: &mut Studio,
    shot: &mut Option<Shot>,
    viewport: egui::Rect,
) -> f32 {
    egui::Area::new("action_bar".into())
        .fixed_pos(egui::pos2(
            viewport.center().x,
            viewport.bottom() - EDGE_MARGIN,
        ))
        .pivot(egui::Align2::CENTER_BOTTOM)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(PANEL)
                .stroke(egui::Stroke::new(
                    studio_theme::HAIRLINE,
                    studio_theme::BRONZE,
                ))
                .corner_radius(10)
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        camera_buttons(ui, shot);
                        ui.separator();
                        if ui
                            .button("Randomize")
                            .on_hover_text("Roll a new build and body shape.")
                            .clicked()
                        {
                            character_controls::randomize(studio);
                        }
                        if ui
                            .button("Reset")
                            .on_hover_text("Return the body and face to neutral.")
                            .clicked()
                        {
                            character_controls::neutral(studio);
                        }
                        ui.separator();
                        if ui
                            .add(studio_theme::primary_button("Save character"))
                            .on_hover_text(format!("Save the recipe to {}", studio.recipe_path))
                            .clicked()
                        {
                            studio.status = save_recipe(studio)
                                .unwrap_or_else(|error| format!("Save failed: {error:#}"));
                        }
                    });
                });
        })
        .response
        .rect
        .top()
}

fn camera_buttons(ui: &mut egui::Ui, shot: &mut Option<Shot>) {
    for (label, hover, request) in [
        ("◀", "Turn left", Shot::TurnLeft),
        ("Full body", "Frame the whole character", Shot::FullBody),
        ("Portrait", "Frame the head and face", Shot::Portrait),
        ("▶", "Turn right", Shot::TurnRight),
    ] {
        if ui.button(label).on_hover_text(hover).clicked() {
            *shot = Some(request);
        }
    }
}

/// The latest status and the navigation hint.
fn status(ctx: &egui::Context, studio: &Studio, viewport: egui::Rect, bar_top: Option<f32>) {
    let bottom = bar_top.map_or(viewport.bottom() - EDGE_MARGIN, |top| top - 8.0);
    egui::Area::new("status".into())
        .fixed_pos(egui::pos2(viewport.center().x, bottom))
        .pivot(egui::Align2::CENTER_BOTTOM)
        .interactable(false)
        .show(ctx, |ui| {
            ui.set_max_width(viewport.width() - 2.0 * EDGE_MARGIN);
            egui::Frame::new()
                .fill(PANEL.gamma_multiply(0.8))
                .corner_radius(10)
                .inner_margin(egui::Margin::symmetric(10, 3))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(&studio.status).small().color(MUTED));
                });
        });
    egui::Area::new("navigation_hint".into())
        .fixed_pos(egui::pos2(
            viewport.right() - EDGE_MARGIN,
            viewport.top() + EDGE_MARGIN,
        ))
        .pivot(egui::Align2::RIGHT_TOP)
        .interactable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Drag to orbit · right-drag to pan · wheel to zoom")
                    .small()
                    .color(MUTED),
            );
        });
}
