//! The heads-up display: the year and the population in one corner, the
//! time controls and the menu button in the other, and, on F3, the figures
//! for developers.

use bevy_egui::egui::{self, Align, Align2, Context, Id, Layout, RichText, vec2};

use crate::live::Viewer;
use crate::world::{Pace, Request};
use crate::{icons, theme, widgets};

/// A time control's side, logical pixels.
const BUTTON: f32 = 38.0;
/// The population sparkline's size, logical pixels.
const SPARKLINE: egui::Vec2 = egui::vec2(170.0, 30.0);
/// The year's type size, logical pixels.
const YEAR_SIZE: f32 = 34.0;

pub fn show(ctx: &Context, viewer: &mut Viewer) {
    status(ctx, viewer);
    time(ctx, viewer);
    if viewer.debug {
        figures(ctx, viewer);
    }
}

/// The scene, the year, and how many plants live.
fn status(ctx: &Context, viewer: &Viewer) {
    let corner = vec2(theme::GUTTER, theme::GUTTER);
    egui::Area::new(Id::new("status"))
        .anchor(Align2::LEFT_TOP, corner)
        .interactable(false)
        .show(ctx, |ui| {
            theme::panel().show(ui, |ui| {
                ui.label(
                    RichText::new(viewer.world.scene.name().to_uppercase())
                        .small()
                        .strong()
                        .color(theme::ACCENT),
                );
                let row = vec2(SPARKLINE.x, YEAR_SIZE);
                ui.allocate_ui_with_layout(row, Layout::left_to_right(Align::Max), |ui| {
                    ui.label(RichText::new("Year").color(theme::MUTED));
                    ui.label(
                        RichText::new(format!("{:.0}", viewer.stats.time))
                            .size(YEAR_SIZE)
                            .strong(),
                    );
                });
                ui.label(format!("{} plants", grouped(viewer.stats.plants)));
                viewer.stats.sparkline(ui, SPARKLINE);
            });
        });
}

/// Pause, the paces, and the menu.
fn time(ctx: &Context, viewer: &mut Viewer) {
    let corner = vec2(-theme::GUTTER, theme::GUTTER);
    egui::Area::new(Id::new("time"))
        .anchor(Align2::RIGHT_TOP, corner)
        .show(ctx, |ui| {
            theme::panel().show(ui, |ui| {
                ui.horizontal(|ui| {
                    let paused = !viewer.world.playing;
                    let pause = widgets::icon_button(ui, BUTTON, paused, icons::pause)
                        .on_hover_text("Pause (Space)");
                    if pause.clicked() {
                        let play = viewer.world.toggle_play();
                        viewer.requests.push(Request::Send(play));
                    }
                    for pace in Pace::ALL {
                        let current = viewer.world.playing && viewer.world.pace == pace;
                        let button = widgets::icon_button(ui, BUTTON, current, |p, r, c| {
                            icons::play(p, r, pace.arrows(), c);
                        })
                        .on_hover_text(pace.describe());
                        if button.clicked() {
                            for command in viewer.world.play_at(pace) {
                                viewer.requests.push(Request::Send(command));
                            }
                        }
                    }
                    ui.separator();
                    let menu = widgets::icon_button(ui, BUTTON, false, icons::menu)
                        .on_hover_text("Menu (Esc)");
                    if menu.clicked() {
                        viewer.menu.open = true;
                    }
                });
            });
        });
}

/// What the simulation and the renderer are doing, for developers.
fn figures(ctx: &Context, viewer: &Viewer) {
    let corner = vec2(theme::GUTTER, -theme::GUTTER);
    let stats = &viewer.stats;
    egui::Area::new(Id::new("figures"))
        .anchor(Align2::LEFT_BOTTOM, corner)
        .interactable(false)
        .show(ctx, |ui| {
            theme::panel().show(ui, |ui| {
                for line in [
                    format!("{:.0} fps", viewer.frame_times.rate()),
                    format!("step {:.1} ms", stats.step_ms),
                    format!("{} modules", grouped(stats.modules)),
                    format!("{} segments", grouped(stats.branches)),
                    format!("{} leaf clusters", grouped(stats.leaves)),
                    format!(
                        "leaf coverage {}",
                        if viewer.leaf_coverage { "kept" } else { "plain" }
                    ),
                    impostor_line(viewer),
                ] {
                    ui.label(RichText::new(line).monospace().color(theme::MUTED));
                }
            });
        });
}

/// What the impostors are doing.
pub fn impostor_line(viewer: &Viewer) -> String {
    let status = &viewer.impostor_status;
    if !viewer.impostors.enabled {
        return "impostors off".to_owned();
    }
    if let Some((done, of)) = status.baking {
        return format!("impostors baking {done}/{of}");
    }
    let Some(year) = status.year else {
        return "impostors waiting for a snapshot".to_owned();
    };
    format!(
        "impostors: {} trees, {} saplings from year {year:.0}, {:.0} MB{}",
        status.trees,
        status.saplings,
        status.bytes as f64 / (1024.0 * 1024.0),
        if status.scene { "" } else { ", scene stale" }
    )
}

/// `count` with its thousands apart: 12,480.
pub fn grouped(count: usize) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}
