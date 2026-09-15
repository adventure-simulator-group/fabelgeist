//! Conservative habitat and phenology filters, separate from organ geometry.
use crate::flower::FlowerSpecies;

/// Local surface class supplied by the tactical scene's ground sampler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlantGround {
    Grass,
    WoodlandLitter,
    Soil,
    Unsuitable,
}

/// Normalized scene habitat and one-based calendar day supplied by the caller.
#[derive(Clone, Copy, Debug)]
pub struct PlantHabitat {
    pub canopy: f32,
    pub cultivation: f32,
    pub moisture: f32,
    pub snow: f32,
    pub day_of_year: u16,
    pub ground: PlantGround,
}
impl PlantHabitat {
    /// Relative occurrence weight; zero excludes an unsuitable site or season.
    pub fn flower_weight(self, species: FlowerSpecies) -> f32 {
        if self.ground == PlantGround::Unsuitable || self.snow > 0.05 {
            return 0.0;
        }
        let (start, end) = match species {
            FlowerSpecies::Daisy => (60, 320),
            FlowerSpecies::MeadowButtercup => (120, 245),
            FlowerSpecies::WoodAnemone => (60, 150),
            FlowerSpecies::CornPoppy => (135, 245),
            FlowerSpecies::NettleLeavedBellflower => (180, 260),
        };
        if !(start..=end).contains(&self.day_of_year) {
            return 0.0;
        }
        let woodland = self.ground == PlantGround::WoodlandLitter;
        match species {
            FlowerSpecies::WoodAnemone if woodland && self.canopy > 0.25 => 1.0,
            FlowerSpecies::NettleLeavedBellflower if self.canopy > 0.2 && self.canopy < 0.85 => {
                0.45
            }
            FlowerSpecies::Daisy if !woodland && self.canopy < 0.4 => 0.8,
            FlowerSpecies::MeadowButtercup
                if !woodland && self.canopy < 0.45 && self.moisture > 0.2 =>
            {
                0.7
            }
            FlowerSpecies::CornPoppy
                if !woodland && self.canopy < 0.3 && self.cultivation > 0.35 =>
            {
                0.65
            }
            _ => 0.0,
        }
    }
}
