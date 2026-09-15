//! Reusable corolla, whorl, inflorescence and leaf architecture controls.
mod geometry;
mod presets;
#[cfg(test)]
mod tests;
use crate::{GenerationError, Pigment, PlantMesh, Tessellation, parameters::bounded};
pub use presets::FlowerSpecies;
use serde::{Deserialize, Serialize};

/// Discrete organ topology. Each topology shares organ surfaces and axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Corolla {
    FreePetals,
    RayAndDisk,
    FusedBell,
}

/// Leaf arrangement along a flowering shoot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LeafArrangement {
    BasalRosette,
    Alternate,
    Whorl,
}

/// Serializable authoring boundary. `generate` validates every scalar first.
/// Dimensions are metres; curvature and width ratios are dimensionless.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowerParameters {
    pub height_m: f32,
    pub stem_radius_m: f32,
    pub stem_lean: f32,
    pub heads: u8,
    pub corolla: Corolla,
    pub petals: u8,
    pub petal_length_m: f32,
    pub petal_width_ratio: f32,
    pub petal_cup: f32,
    pub petal_notch: f32,
    pub petal_ripple: f32,
    pub center_radius_m: f32,
    pub center_height_ratio: f32,
    pub stamens: u8,
    pub head_tilt: f32,
    pub leaf_arrangement: LeafArrangement,
    pub leaves: u8,
    pub leaf_length_m: f32,
    pub leaf_width_ratio: f32,
    pub leaf_lobes: u8,
    pub leaf_lobe_depth: f32,
    pub leaflets: u8,
    pub leaf_fan_radians: f32,
    pub petal: Pigment,
    pub petal_base: Pigment,
    pub base_fraction: f32,
    pub center: Pigment,
    pub anther: Pigment,
    pub green: Pigment,
}
impl FlowerParameters {
    pub fn validate(&self) -> Result<(), GenerationError> {
        for (field, value, min, max) in [
            ("height_m", self.height_m, 0.03, 1.5),
            ("stem_radius_m", self.stem_radius_m, 0.0003, 0.008),
            ("stem_lean", self.stem_lean, 0.0, 0.3),
            ("heads", f32::from(self.heads), 1.0, 8.0),
            ("petals", f32::from(self.petals), 3.0, 40.0),
            ("petal_length_m", self.petal_length_m, 0.003, 0.06),
            ("petal_width_ratio", self.petal_width_ratio, 0.06, 1.5),
            ("petal_cup", self.petal_cup, -0.4, 1.5),
            ("petal_notch", self.petal_notch, 0.0, 0.35),
            ("petal_ripple", self.petal_ripple, 0.0, 0.15),
            ("center_radius_m", self.center_radius_m, 0.0005, 0.018),
            ("center_height_ratio", self.center_height_ratio, 0.1, 1.5),
            ("stamens", f32::from(self.stamens), 0.0, 48.0),
            ("head_tilt", self.head_tilt, 0.0, std::f32::consts::PI),
            ("leaves", f32::from(self.leaves), 1.0, 12.0),
            ("leaf_length_m", self.leaf_length_m, 0.008, 0.15),
            ("leaf_width_ratio", self.leaf_width_ratio, 0.08, 1.0),
            ("leaf_lobes", f32::from(self.leaf_lobes), 0.0, 8.0),
            ("leaf_lobe_depth", self.leaf_lobe_depth, 0.0, 0.85),
            ("leaflets", f32::from(self.leaflets), 1.0, 7.0),
            (
                "leaf_fan_radians",
                self.leaf_fan_radians,
                0.0,
                std::f32::consts::PI,
            ),
            ("base_fraction", self.base_fraction, 0.0, 0.6),
        ] {
            bounded(field, value, min, max)?;
        }
        Ok(())
    }

    pub fn generate(&self, seed: u64, detail: Tessellation) -> Result<PlantMesh, GenerationError> {
        self.validate()?;
        Ok(geometry::generate(self, seed, detail))
    }
}
