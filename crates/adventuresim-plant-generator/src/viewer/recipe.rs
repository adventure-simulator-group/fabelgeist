use super::*;
use adventuresim_plant_generator::{
    GenerationError, PlantMesh,
    fungus::{FungusParameters, FungusSpecies},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(super) enum Family {
    Flowers,
    Fungi,
}
impl Family {
    pub(super) fn count(self) -> usize {
        match self {
            Self::Flowers => FlowerSpecies::ALL.len(),
            Self::Fungi => FungusSpecies::ALL.len(),
        }
    }
    pub(super) fn name(self, index: usize) -> &'static str {
        match self {
            Self::Flowers => FlowerSpecies::ALL[index].name(),
            Self::Fungi => FungusSpecies::ALL[index].name(),
        }
    }
    pub(super) fn recipe(self, index: usize) -> Recipe {
        match self {
            Self::Flowers => Recipe::Flower(FlowerSpecies::ALL[index].parameters()),
            Self::Fungi => Recipe::Fungus(FungusSpecies::ALL[index].parameters()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "family", content = "parameters", deny_unknown_fields)]
pub(super) enum Recipe {
    Flower(FlowerParameters),
    Fungus(FungusParameters),
}
impl Recipe {
    pub(super) fn validate(&self) -> Result<(), GenerationError> {
        match self {
            Self::Flower(p) => p.validate(),
            Self::Fungus(p) => p.validate(),
        }
    }
    pub(super) fn family(&self) -> Family {
        match self {
            Self::Flower(_) => Family::Flowers,
            Self::Fungus(_) => Family::Fungi,
        }
    }
    pub(super) fn generate(
        &self,
        seed: u64,
        detail: Tessellation,
    ) -> Result<PlantMesh, GenerationError> {
        match self {
            Self::Flower(p) => p.generate(seed, detail),
            Self::Fungus(p) => p.generate(seed, detail),
        }
    }
    pub(super) fn controls(&self) -> serde_json::Value {
        match self {
            Self::Flower(p) => serde_json::to_value(p).unwrap(),
            Self::Fungus(p) => serde_json::to_value(p).unwrap(),
        }
    }
    pub(super) fn edited(&self, value: serde_json::Value) -> Result<Self, serde_json::Error> {
        match self {
            Self::Flower(_) => serde_json::from_value(value).map(Self::Flower),
            Self::Fungus(_) => serde_json::from_value(value).map(Self::Fungus),
        }
    }
    pub(super) fn framing(&self) -> (f32, Vec3, f32) {
        match self {
            Self::Flower(p) => (
                p.height_m,
                Vec3::new(p.height_m * p.stem_lean, p.height_m, 0.0),
                (p.petal_length_m + p.center_radius_m) * 4.0,
            ),
            Self::Fungus(p) => (
                p.height_m().max(p.cap_radius_m * 2.0),
                Vec3::new(p.cap_elevation_m * p.stipe_bend, p.cap_elevation_m, 0.0),
                p.cap_radius_m * 2.0,
            ),
        }
    }
}
