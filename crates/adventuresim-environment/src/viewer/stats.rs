//! What the HUD shows of the simulation, from its last snapshot.

use std::collections::VecDeque;

use bevy_egui::egui::{Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use fabelgeist_botany::viewer::sim::{Snapshot, SpeciesInfo};

use crate::theme;

/// Population samples the sparkline keeps.
const HISTORY: usize = 240;
/// The sparkline's stroke width, logical pixels.
const SPARKLINE_STROKE: f32 = 2.0;

#[derive(Default)]
pub struct Stats {
    pub time: f32,
    pub plants: usize,
    pub modules: usize,
    pub step_ms: f32,
    pub branches: usize,
    pub leaves: usize,
    species: Vec<SpeciesInfo>,
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
    }

    /// The species growing, the climbers after the plants.
    pub fn species(&self) -> &[SpeciesInfo] {
        &self.species
    }

    /// A new scene starts a new population curve.
    pub fn forget_history(&mut self) {
        self.history.clear();
    }

    /// The population over time, in a `size` box.
    pub fn sparkline(&self, ui: &mut Ui, size: Vec2) {
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        if self.history.len() > 1 {
            sparkline(ui, rect, &self.history);
        }
    }
}

/// `values`, scaled to fill `rect`.
fn sparkline(ui: &Ui, rect: Rect, values: &VecDeque<f32>) {
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
    let stroke = Stroke::new(SPARKLINE_STROKE, theme::ACCENT);
    ui.painter().add(Shape::line(points, stroke));
}
