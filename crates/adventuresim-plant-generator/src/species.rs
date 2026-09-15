use crate::{
    GenerationError, PlantLod, PlantMesh, flower::FlowerSpecies, fungus::FungusSpecies,
    habitat::PlantHabitat,
};
use serde::{Deserialize, Serialize};

/// Botanical catalog identity, separate from continuous organ parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlantSpecies {
    Flower(FlowerSpecies),
    Fungus(FungusSpecies),
}
impl PlantSpecies {
    pub const ALL: [Self; 9] = [
        Self::Flower(FlowerSpecies::Daisy),
        Self::Flower(FlowerSpecies::MeadowButtercup),
        Self::Flower(FlowerSpecies::WoodAnemone),
        Self::Flower(FlowerSpecies::CornPoppy),
        Self::Flower(FlowerSpecies::NettleLeavedBellflower),
        Self::Fungus(FungusSpecies::FlyAgaric),
        Self::Fungus(FungusSpecies::Porcini),
        Self::Fungus(FungusSpecies::Chanterelle),
        Self::Fungus(FungusSpecies::CommonPuffball),
    ];
    pub fn generate(self, seed: u64, detail: PlantLod) -> Result<PlantMesh, GenerationError> {
        match self {
            Self::Flower(s) => s.parameters().generate(seed, detail),
            Self::Fungus(s) => s.parameters().generate(seed, detail),
        }
    }
    pub fn habitat_weight(self, habitat: PlantHabitat) -> f32 {
        match self {
            Self::Flower(s) => habitat.flower_weight(s),
            Self::Fungus(s) => habitat.fungus_weight(s),
        }
    }
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|s| *s == self)
            .expect("catalog species")
    }
}
