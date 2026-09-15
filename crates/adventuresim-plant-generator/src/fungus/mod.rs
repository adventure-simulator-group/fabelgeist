//! Shared radial profiles, fertile surfaces, veil remnants and fruiting bodies.
mod geometry;
mod presets;
#[cfg(test)]
mod tests;
use crate::{GenerationError, Pigment, PlantMesh, Tessellation, parameters::bounded};
pub use presets::FungusSpecies;
use serde::{Deserialize, Serialize};

/// The spore-bearing surface is structural, rather than a painted underside.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FertileSurface {
    Gills,
    Pores,
    Ridges,
    Enclosed,
}

/// Metre-based profile controls. Species presets never select mesh algorithms.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FungusParameters {
    pub cap_elevation_m: f32,
    pub stipe_radius_m: f32,
    pub stipe_base_ratio: f32,
    pub stipe_bend: f32,
    pub cap_radius_m: f32,
    pub cap_rise_ratio: f32,
    pub cap_thickness_ratio: f32,
    pub cap_depression_ratio: f32,
    pub rim_wave: f32,
    pub rim_lobes: u8,
    pub asymmetry: f32,
    pub fertile_surface: FertileSurface,
    pub fold_count: u16,
    pub fold_depth_m: f32,
    pub decurrent_m: f32,
    pub ring_radius_ratio: f32,
    pub ring_height_fraction: f32,
    pub ornament_count: u16,
    pub ornament_radius_m: f32,
    pub ornament_height_m: f32,
    pub cap: Pigment,
    pub underside: Pigment,
    pub stipe: Pigment,
    pub ornament: Pigment,
}
impl FungusParameters {
    pub fn validate(&self) -> Result<(), GenerationError> {
        for (name, value, min, max) in [
            ("cap_elevation_m", self.cap_elevation_m, 0.008, 0.3),
            ("stipe_radius_m", self.stipe_radius_m, 0.002, 0.055),
            ("stipe_base_ratio", self.stipe_base_ratio, 0.5, 2.5),
            ("stipe_bend", self.stipe_bend, 0.0, 0.25),
            ("cap_radius_m", self.cap_radius_m, 0.012, 0.15),
            ("cap_rise_ratio", self.cap_rise_ratio, 0.0, 1.5),
            ("cap_thickness_ratio", self.cap_thickness_ratio, 0.02, 1.0),
            ("cap_depression_ratio", self.cap_depression_ratio, 0.0, 0.55),
            ("rim_wave", self.rim_wave, 0.0, 0.18),
            ("rim_lobes", f32::from(self.rim_lobes), 2.0, 12.0),
            ("asymmetry", self.asymmetry, 0.0, 0.2),
            ("fold_count", f32::from(self.fold_count), 12.0, 120.0),
            ("fold_depth_m", self.fold_depth_m, 0.0005, 0.008),
            (
                "decurrent_m",
                self.decurrent_m,
                0.0,
                self.cap_elevation_m * 0.6,
            ),
            ("ring_radius_ratio", self.ring_radius_ratio, 0.0, 3.0),
            (
                "ring_height_fraction",
                self.ring_height_fraction,
                0.25,
                0.85,
            ),
            ("ornament_count", f32::from(self.ornament_count), 0.0, 220.0),
            ("ornament_radius_m", self.ornament_radius_m, 0.0004, 0.008),
            ("ornament_height_m", self.ornament_height_m, 0.0003, 0.006),
        ] {
            bounded(name, value, min, max)?;
        }
        bounded(
            "stipe_radius_m",
            self.stipe_radius_m,
            0.002,
            self.cap_radius_m * 0.8,
        )?;
        bounded(
            "cap_elevation_m",
            self.cap_elevation_m,
            self.cap_radius_m * (self.cap_thickness_ratio + self.rim_wave)
                + self.decurrent_m
                + self.fold_depth_m,
            0.3,
        )?;
        // The depressed cap centre must stay above the bottom of its stem.
        bounded(
            "cap_depression_ratio",
            self.cap_depression_ratio,
            0.0,
            (self.cap_elevation_m / self.cap_radius_m + self.cap_rise_ratio) * 0.8,
        )?;
        bounded(
            "cap_depression_ratio",
            self.cap_depression_ratio,
            0.0,
            self.cap_rise_ratio + self.cap_thickness_ratio + self.decurrent_m / self.cap_radius_m
                - 0.005,
        )?;
        Ok(())
    }
    pub fn height_m(&self) -> f32 {
        self.cap_elevation_m + self.cap_radius_m * self.cap_rise_ratio
    }
    pub fn generate(&self, seed: u64, detail: Tessellation) -> Result<PlantMesh, GenerationError> {
        self.validate()?;
        Ok(geometry::generate(self, seed, detail))
    }
}
