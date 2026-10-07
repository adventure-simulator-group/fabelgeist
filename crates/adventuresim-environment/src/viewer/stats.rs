//! What the panel shows of the simulation, from its last snapshot.

use std::collections::VecDeque;

use eframe::egui::{self, Color32, Pos2, Sense, Shape, Stroke, Ui, Vec2};
use fabelgeist_botany::Biome;
use fabelgeist_botany::viewer::sim::{Snapshot, SpeciesInfo};

/// Population samples the sparkline keeps.
const HISTORY: usize = 240;
/// The sparkline's height, logical pixels.
const SPARKLINE_HEIGHT: f32 = 28.0;
/// A species swatch's side, logical pixels.
const SWATCH: f32 = 12.0;
/// The sparkline's stroke width, logical pixels.
const SPARKLINE_STROKE: f32 = 1.5;

#[derive(Default)]
pub struct Stats {
    pub time: f32,
    plants: usize,
    modules: usize,
    step_ms: f32,
    branches: usize,
    leaves: usize,
    species: Vec<SpeciesInfo>,
    pub biome: Option<Biome>,
    /// Plants alive, one sample a simulated step.
    history: VecDeque<f32>,
}

impl Stats {
    pub fn record(&mut self, snapshot: &Snapshot) {
        if snapshot.time != self.time || self.history.is_empty() {
            self.history.push_back(snapshot.plants as f32);
            while self.history.len() > HISTORY {
                self.history.pop_front();
            }
        }
        self.time = snapshot.time;
        self.plants = snapshot.plants;
        self.modules = snapshot.modules;
        self.step_ms = snapshot.step_ms;
        self.branches = snapshot.branches.len();
        self.leaves = snapshot.leaves.len();
        self.species = snapshot.species.clone();
        self.biome = Some(snapshot.biome);
    }

    /// The species growing, the climbers after the plants.
    pub fn species(&self) -> &[SpeciesInfo] {
        &self.species
    }

    /// A new scene starts a new population curve.
    pub fn forget_history(&mut self) {
        self.history.clear();
    }

    pub fn show(&self, ui: &mut Ui, frames_per_second: f32) {
        ui.strong(format!("Year {:.0}", self.time));
        ui.weak(format!("{} plants, {} modules", self.plants, self.modules));
        ui.weak(format!(
            "{} segments, {} leaf clusters",
            self.branches, self.leaves
        ));
        ui.weak(format!(
            "step {:.1} ms, {frames_per_second:.0} fps",
            self.step_ms
        ));
        if self.history.len() > 1 {
            sparkline(ui, &self.history);
        }
        for species in &self.species {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(SWATCH), Sense::hover());
                let [r, g, b] = species.leaf;
                ui.painter().circle_filled(
                    rect.center(),
                    SWATCH * 0.5,
                    Color32::from(egui::Rgba::from_rgb(r, g, b)),
                );
                ui.label(format!("{}  {}", species.name, species.plants));
            });
        }
    }
}

/// The population over time, scaled to fill its row.
fn sparkline(ui: &mut Ui, values: &VecDeque<f32>) {
    let size = Vec2::new(ui.available_width(), SPARKLINE_HEIGHT);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let (low, high) = values
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(low, high), &value| {
            (low.min(value), high.max(value))
        });
    let range = (high - low).max(1.0);
    let last = (values.len() - 1) as f32;
    let points: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(i, &value)| {
            Pos2::new(
                rect.left() + rect.width() * i as f32 / last,
                rect.bottom() - rect.height() * (value - low) / range,
            )
        })
        .collect();
    let stroke = Stroke::new(SPARKLINE_STROKE, ui.visuals().selection.bg_fill);
    ui.painter().add(Shape::line(points, stroke));
}
